#![feature(str_from_utf16_endian)]

use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, Read};
use std::path::PathBuf;

use encoding_rs::{Encoding, UTF_16LE};
use log::{debug, trace};

pub use crate::types::{InfEntry, InfSection, InfValue};

mod types;

/// Errors that can occur while parsing a Windows INF file
#[derive(Debug, thiserror::Error)]
pub enum WinInfFileError {
    /// The specified file does not exist
    #[error("File does not exist")]
    FileDoNotExist,
    /// Failed to open the file
    #[error("Failed to open file: {0}")]
    FileOpenError(#[from] Error),
    /// Failed to read the file contents
    #[error("Failed to read file")]
    FileReadError,
    /// Failed to read a line from the file
    #[error("Failed to read line: {0}")]
    ReadLineError(#[from] LineReaderError),
    /// Failed to parse a section in the file
    #[error("Failed to parse section: {0}")]
    SectionParseError(#[from] SectionReaderError),
}

/// Errors that can occur while reading lines from a file
#[derive(Debug, thiserror::Error)]
pub enum LineReaderError {
    /// Invalid CRLF sequence found in the file
    #[error("Invalid CRLF sequence: {0}")]
    InvalidCrlf(String),
}

/// Errors that can occur while parsing sections in a file
#[derive(Debug, thiserror::Error)]
pub enum SectionReaderError {
    /// Invalid section name found in the file
    #[error("Invalid section name: {0}")]
    InvalidSectionName(String),
    /// Invalid quoted value found in a section
    #[error("Invalid quoted value: {0}")]
    InvalidQuotedValue(String),
    /// Invalid line continuation found in a section
    #[error("Invalid continuation: {0}")]
    InvalidContinuation(String),
}

/// A Windows INF file parser
/// 
/// This struct provides functionality to parse Windows INF files and access their contents.
/// INF files are used for device driver installation and configuration in Windows.
#[derive(Default)]
pub struct WinInfFile {
    /// The sections contained in the INF file
    pub sections: HashMap<String, InfSection>,
    section_reader: SectionReader
}

#[derive(Default)]
struct LineReader {
    pub remaining_string: String,
    pub lines: Vec<String>
}

impl LineReader {
    fn read_to_line(&mut self, line_part: &str) -> Result<(), LineReaderError> {
        let mut found_cr = false;
        // pre-allocate memory for the new line at one shot and reuse it
        let mut new_line = String::with_capacity(self.remaining_string.len() + line_part.len());
        for c in self.remaining_string.chars().chain(line_part.chars()) {
            // If LF did not follow CR, fail
            if found_cr && c != '\n' {
                return Err(LineReaderError::InvalidCrlf(format!("found \\r but not \\n immediately")));
            }
            // If CRLF encountered, read to line
            if found_cr && c == '\n' {
                if new_line.len() != 0 {
                    self.lines.push(new_line.clone());
                    new_line.clear();
                }
                found_cr = false;
                continue
            }

            // If CR encountered, set flag
            if c == '\r' {
                found_cr = true;
                continue
            }

            // If \n encountered, read to line
            if c == '\n' {
                if new_line.len() != 0 {
                    self.lines.push(new_line.clone());
                    new_line.clear();
                }
                continue
            }

            // Add each char to the new line
            new_line.push(c);
        }

        // Add remaining line part without line ending back to the reader
        self.remaining_string = new_line;
        Ok(())
    }

    fn take_lines(&mut self) -> Vec<String> {
        self.lines.drain(0..).collect()
    }

    fn finalize(&mut self) {
        if !self.remaining_string.is_empty() {
            self.lines.push(self.remaining_string.clone());
        }
    }
}

#[derive(Default)]
struct SectionReader {
    last_section_name: String,
    last_entry_key: String,
    last_entry_value_contd: String
}

impl SectionReader {
    fn read_section(&mut self, line: String, sections: &mut HashMap<String, InfSection>) -> Result<(), SectionReaderError> {
        // trim spaces and tabs
        let line = line.trim();

        // exclude comments
        if line.starts_with(';') {
            return Ok(());
        }

        // section name
        if line.starts_with('[') && line.ends_with(']') {
            let section_name = line[1..line.len()-1].to_string();
            if let Err(e) = validate_section_name(section_name.clone()) {
                return Err(SectionReaderError::InvalidSectionName(e.to_string()));
            }

            // TODO: if there are multiple sections with same name, we have to merge them
            sections.insert(section_name.clone(), InfSection{ name: section_name.clone(), entries: vec![] });
            self.last_section_name = section_name.clone();
            return Ok(());
        }

        // entries
        if !self.last_section_name.is_empty() {
            debug!("processing entries for section name: {}", self.last_section_name);
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let mut value = value.trim();

                if value.starts_with('"') {
                    debug!("processing quoted value: {}", value);
                    let end_double_quote_idx = value[1..].find('"');
                    if end_double_quote_idx.is_none() {
                        return Err(SectionReaderError::InvalidQuotedValue(format!(
                            "no ending double quote found, key: {}, section: {}", 
                            key, self.last_section_name
                        )));
                    }
                    // +1 to include the first double quote
                    let end_double_quote_idx = end_double_quote_idx.unwrap()+1usize;
                    // check for continuation char, -1 of length since its zero based
                    if value.len()-1usize > end_double_quote_idx {
                        if let Some(c) = value.chars().nth(end_double_quote_idx + 1usize) {
                        // check if the char after ending double quote is continuation char
                            if c == '\\' {
                                self.last_entry_key = key.to_string();
                                self.last_entry_value_contd = value[1..end_double_quote_idx].to_string();
                                return Ok(());
                            } else if c != ';' {
                                // TODO: if there are any chars after ending double quote apart from continuation char or comment, should we consider this as malformed value?
                                return Err(SectionReaderError::InvalidContinuation(format!(
                                    "Invalid INF entry value: {}, no continuation char found after ending double quote, key: {}, section_name: {}", 
                                    value, key, self.last_section_name
                                )));
                            }
                        }
                    }

                    self.last_entry_value_contd.clear();
                    // exclude double quotes
                    value = &value[1..end_double_quote_idx];
                    if let Some(section) = sections.get_mut(&self.last_section_name) {
                        let new_entry = InfEntry::KeyValue(key.to_string(), Some(InfValue::Raw(value.to_string())) );
                        section.entries.push(new_entry);
                    }

                } else {
                    debug!("processing unquoted value: {}", value);

                    // value containing comments at the end
                    if let Some((first, _)) = value.split_once(';') {
                        value = first.trim();
                    }

                    // multiple backslashes at the end, windows treat only the last one as line continuator and ignores rest
                    if value.ends_with('\\') {
                        debug!("processing unquoted contd value: {}", value);
                        if let Some(first_backslash_idx) = value.find('\\') {
                            if first_backslash_idx > 0 {
                                self.last_entry_value_contd = value[..first_backslash_idx].to_string();
                                self.last_entry_key = key.to_string();
                                return Ok(())
                            } else {
                                self.last_entry_value_contd.clear();
                                self.last_entry_key.clear();
                                return Ok(())
                            }
                        }
                    } else {
                        debug!("processing unquoted non contd value: {}", value);
                        self.last_entry_value_contd.clear();
                        self.last_entry_key.clear();

                        if let Some(entry) = sections.get_mut(&self.last_section_name) {
                            entry.entries.push(InfEntry::KeyValue(key.to_string(), Some(InfValue::Raw(value.to_string())) ))
                        }
                    }
                }
            }
            else {
                let value = line.trim();

                // TODO: what if there are multiple continuation lines?
                if !self.last_entry_value_contd.is_empty() {
                    if let Some(entry) = sections.get_mut(&self.last_section_name) {
                        self.last_entry_value_contd.push_str(value);
                        entry.entries.push(InfEntry::KeyValue(
                            self.last_entry_key.to_string(), 
                            Some(InfValue::Raw(self.last_entry_value_contd.clone()))
                        ));
                    }
                    self.last_entry_value_contd.clear();
                    self.last_entry_key.clear();
                } else {
                    if let Some(entry) = sections.get_mut(&self.last_section_name) {
                        entry.entries.push(InfEntry::OnlyValue(InfValue::Raw(value.to_string())));
                    }
                }
            }
        }
        Ok(())
    }
}

impl WinInfFile {
    /// Parse a Windows INF file from the given path
    /// 
    /// # Arguments
    /// 
    /// * `file_path` - The path to the INF file to parse
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` if the file was parsed successfully
    /// * `Err(WinInfFileError)` if an error occurred during parsing
    /// 
    /// # Examples
    /// 
    /// ```
    /// use inf_rs::WinInfFile;
    /// use std::path::PathBuf;
    /// 
    /// let mut inf_file = WinInfFile::default();
    /// let result = inf_file.parse(PathBuf::from("tests/fixtures/Intel.inf"));
    /// assert!(result.is_ok());
    /// ```
    pub fn parse(&mut self, file_path: PathBuf) -> Result<(), WinInfFileError> {
        if !file_path.exists() {
            return Err(WinInfFileError::FileDoNotExist);
        }

        let mut f = File::open(file_path)?;
        let mut decoder = UTF_16LE.new_decoder();

        let mut line_reader = LineReader::default();

        let buf_size = 1024;
        let mut bom_detected = false;
        loop {
            let mut buf: Vec<u8> = vec![0; buf_size];
            let read_count = f.read(&mut buf);
            if read_count.is_err() {
                return Err(WinInfFileError::FileReadError);
            }
            let read_count = read_count.unwrap();
            if read_count <= 0 {
                trace!("bytes read: {}", read_count);
                break;
            }

            // This properly prints the BOM value for the first buf
            // Bom data: (Encoding { UTF-16LE }, 2)
            let bom = Encoding::for_bom(&buf[..read_count]);
            if let Some(b) = bom {
                debug!("Bom data: {b:?}");
                bom_detected = true;
            }

            if bom_detected {
                // This works perfectly for UTF16 LE
                // Ref: https://learn.microsoft.com/en-us/windows-hardware/drivers/display/general-unicode-requirement
                let mut utf16_buf = vec![0; buf_size/2];
                let res = decoder.decode_to_utf16(&buf[..read_count], &mut utf16_buf, read_count != buf_size);
                debug!("decoded chars: {:?}", String::from_utf16_lossy(&utf16_buf[..res.2]));
                if let Err(e) = line_reader.read_to_line(&String::from_utf16_lossy(&utf16_buf[..res.2])) {
                    return Err(WinInfFileError::ReadLineError(e));
                }
            } else {
                debug!("decoded chars: {:?}", String::from_utf8_lossy(&buf[..read_count]));
                if let Err(e) = line_reader.read_to_line(&String::from_utf8_lossy(&buf[..read_count]).to_string()) {
                    return Err(WinInfFileError::ReadLineError(e));
                }
            }

            for line in line_reader.take_lines() {
                if let Err(e) = self.section_reader.read_section(line, &mut self.sections) {
                    return Err(WinInfFileError::SectionParseError(e));
                }
            }
        }

        line_reader.finalize();
        for line in line_reader.take_lines() {
            if let Err(e) = self.section_reader.read_section(line, &mut self.sections) {
                return Err(WinInfFileError::SectionParseError(e));
            }
        }

        debug!("total lines: {}", line_reader.lines.len());
        for line in line_reader.lines.iter() {
            debug!(">> line: {}", line);
        }

        for (section_name, section) in self.sections.iter() {
            debug!(">> section name: {}, section: {:?}", section_name, section);
        }

        Ok(())
    }
}

fn validate_section_name<'a>(name: String) -> Result<(), &'a str> {
    debug!("validate section name: {}", name);
    if name.starts_with('\"') {
        // quoted section name
        // double quotes within are also allowed when escaped
        if !name.ends_with('\"') {
            return Err("invalid quoted section name");
        }

        if name.contains(']') {
            return Err("invalid ] in the quoted section name");
        }
        Ok(())
    } else {
        // unquoted section name
        if name.ends_with('\\') {
            return Err("invalid \\ at the end of section name");
        }

        // count the number of % in the section name
        if name.chars().filter(|c| *c == '%').count() % 2 != 0 {
            return Err("odd number of % in the section name, expected pairs");
        }

        name.find(|c| {
            // TODO: Also prevent invisible control characters here
            if c == '\r' || c == '\n' || c == '\"' || c == ' '
                || c == '\t' || c == '[' || c == ']' || c == ';' {
                return true;
            }
            return false;
        }).map_or_else(|| Ok(()), |_| Err("contains invalid chars in unquoted section name"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_reader_basic() {
        let mut reader = LineReader::default();
        assert!(reader.read_to_line("Hello\r\nWorld\r\n").is_ok());
        let lines = reader.take_lines();
        assert_eq!(lines, vec!["Hello", "World"]);
    }

    #[test]
    fn test_line_reader_incomplete_line() {
        let mut reader = LineReader::default();
        assert!(reader.read_to_line("Hello\r\nWor").is_ok());
        let lines = reader.take_lines();
        assert_eq!(lines, vec!["Hello"]);
        assert_eq!(reader.remaining_string, "Wor");
    }

    #[test]
    fn test_line_reader_invalid_crlf() {
        let mut reader = LineReader::default();
        assert!(reader.read_to_line("Hello\rWorld").is_err());
    }

    #[test]
    fn test_line_reader_finalize() {
        let mut reader = LineReader::default();
        assert!(reader.read_to_line("Hello\r\nWorld").is_ok());
        reader.finalize();
        let lines = reader.take_lines();
        assert_eq!(lines, vec!["Hello", "World"]);
    }

    #[test]
    fn test_section_reader_basic() {
        let mut reader = SectionReader::default();
        let mut sections = HashMap::new();

        // Test section header
        assert!(reader.read_section("[TestSection]".to_string(), &mut sections).is_ok());
        assert_eq!(reader.last_section_name, "TestSection");
        assert!(sections.contains_key("TestSection"));

        // Test key-value pair
        assert!(reader.read_section("key=value".to_string(), &mut sections).is_ok());
        let section = sections.get("TestSection").unwrap();
        assert_eq!(section.entries.len(), 1);
        if let InfEntry::KeyValue(key, value) = &section.entries[0] {
            assert_eq!(key, "key");
            assert_eq!(value.as_ref().unwrap(), &InfValue::Raw("value".to_string()));
        } else {
            panic!("Expected KeyValue entry");
        }
    }

    #[test]
    fn test_section_reader_quoted_value() {
        let mut reader = SectionReader::default();
        let mut sections = HashMap::new();

        assert!(reader.read_section("[TestSection]".to_string(), &mut sections).is_ok());
        assert!(reader.read_section("key=\"quoted value\"".to_string(), &mut sections).is_ok());
        
        let section = sections.get("TestSection").unwrap();
        assert_eq!(section.entries.len(), 1);
        if let InfEntry::KeyValue(key, value) = &section.entries[0] {
            assert_eq!(key, "key");
            assert_eq!(value.as_ref().unwrap(), &InfValue::Raw("quoted value".to_string()));
        } else {
            panic!("Expected KeyValue entry");
        }
    }

    #[test]
    fn test_section_reader_quoted_value_with_continuation() {
        let mut reader = SectionReader::default();
        let mut sections = HashMap::new();

        assert!(reader.read_section("[TestSection]".to_string(), &mut sections).is_ok());
        assert!(reader.read_section("key=\"quoted value\\\"\\".to_string(), &mut sections).is_ok());
        assert_eq!(reader.last_entry_value_contd, "quoted value\\");
        assert_eq!(reader.last_entry_key, "key");

        assert!(reader.read_section("  continued value  ".to_string(), &mut sections).is_ok());
        let section = sections.get("TestSection").unwrap();
        assert_eq!(section.entries.len(), 1);
        if let InfEntry::KeyValue(key, value) = &section.entries[0] {
            assert_eq!(key, "key");
            assert_eq!(value.as_ref().unwrap(), &InfValue::Raw("quoted value\\continued value".to_string()));
        } else {
            panic!("Expected KeyValue entry");
        }
    }

    #[test]
    fn test_section_reader_continued_value() {
        let mut reader = SectionReader::default();
        let mut sections = HashMap::new();

        assert!(reader.read_section("[TestSection]".to_string(), &mut sections).is_ok());
        assert!(reader.read_section("key=value\\".to_string(), &mut sections).is_ok());
        assert_eq!(reader.last_entry_value_contd, "value");
        
        assert!(reader.read_section("continued".to_string(), &mut sections).is_ok());
        let section = sections.get("TestSection").unwrap();
        assert_eq!(section.entries.len(), 1);
        if let InfEntry::KeyValue(key, value) = &section.entries[0] {
            assert_eq!(key, "key");
            assert_eq!(value.as_ref().unwrap(), &InfValue::Raw("valuecontinued".to_string()));
        } else {
            panic!("Expected KeyValue entry");
        }
    }

    #[test]
    fn test_section_reader_comments() {
        let mut reader = SectionReader::default();
        let mut sections = HashMap::new();

        assert!(reader.read_section("[TestSection]".to_string(), &mut sections).is_ok());
        assert!(reader.read_section("; This is a comment".to_string(), &mut sections).is_ok());
        assert!(reader.read_section("key=value ; This is a comment".to_string(), &mut sections).is_ok());
        
        let section = sections.get("TestSection").unwrap();
        assert_eq!(section.entries.len(), 1);
        if let InfEntry::KeyValue(key, value) = &section.entries[0] {
            assert_eq!(key, "key");
            assert_eq!(value.as_ref().unwrap(), &InfValue::Raw("value".to_string()));
        } else {
            panic!("Expected KeyValue entry");
        }
    }

    #[test]
    fn test_section_reader_invalid_section() {
        let mut reader = SectionReader::default();
        let mut sections = HashMap::new();

        assert!(reader.read_section("[Invalid Section]".to_string(), &mut sections).is_err());
        assert!(reader.read_section("[Section with \\]".to_string(), &mut sections).is_err());
    }
}