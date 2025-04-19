#![feature(str_from_utf16_endian)]

mod types;

use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, Read};
use std::ops::Index;
use std::path::PathBuf;
use encoding_rs::{Encoding, UTF_16LE};
use nom::character::complete::{char, line_ending, not_line_ending};
use nom::Parser;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::preceded;
use log::{debug, info};
use crate::types::{InfEntry, InfSection, InfValue};
use crate::WinInfFileParseError::{FileDoNotExist, FileOpenError, FileReadError};

#[derive(Default)]
pub struct WinInfFile {
    sections: HashMap<String, InfSection>,
    last_section_name: String,
    last_entry_value_contd: String
}

#[derive(Default)]
struct LineReader {
    pub remaining_string: String,
    pub lines: Vec<String>
}

impl LineReader {
    fn read_to_line(&mut self, line_part: &str) -> Result<(), &str>{
        self.remaining_string.push_str(line_part);
        let mut found_cr = false;
        let mut new_line = String::from("");
        for c in self.remaining_string.clone().chars() {
            // If LF did not follow CR, fail
            if found_cr && c != '\n' {
                return Err("invalid crlf char, found \r but not \n immediately")
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

            if c == '\r' {
                found_cr = true;
                continue
            }

            if c == '\n' {
                if new_line.len() != 0 {
                    self.lines.push(new_line.clone());
                    new_line.clear();
                }
                continue
            }

            new_line.push(c);
        }
        self.remaining_string = new_line;
        Ok(())
    }

    fn take_lines(&mut self) -> Vec<String> {
        let lines: Vec<String> = self.lines.drain(0..).collect();
        lines
    }

    fn finalize(&mut self) {
        if !self.remaining_string.is_empty() {
            self.lines.push(self.remaining_string.clone());
        }
    }
}

#[derive(Debug)]
pub enum WinInfFileParseError {
    FileDoNotExist(),
    FileOpenError(Error),
    FileReadError(),
    ReadLineError(String),
    SectionParseError(String)
}

impl WinInfFile {
    pub fn parse(&mut self, file_path: PathBuf) -> Result<(), WinInfFileParseError> {
        // check if file exists
        if !file_path.exists() {
            return Err(FileDoNotExist());
        }

        let mut f = File::open(file_path);
        if f.as_ref().is_err() {
            return Err(FileOpenError(f.err().unwrap()));
        }
        let mut f = f.unwrap();
        let mut decoder = UTF_16LE.new_decoder();

        let mut line_reader = LineReader::default();

        let buf_size = 1024;
        let mut bom_detected = false;
        loop {
            let mut buf: Vec<u8> = vec![0; buf_size];
            let read_count = f.read(&mut buf);
            if read_count.is_err() {
                return Err(FileReadError())
            }
            let read_count = read_count.unwrap();
            if read_count <= 0 {
                println!("bytes read: {}", read_count);
                break;
            }

            // This properly prints the BOM value for the first buf
            // Bom data: (Encoding { UTF-16LE }, 2)
            let bom = Encoding::for_bom(&buf[..read_count]);
            if let Some(b) = bom {
                println!("Bom data: {b:?}");
                bom_detected = true;
            }

            if bom_detected {
                // This works perfectly for UTF16 LE
                // Ref: https://learn.microsoft.com/en-us/windows-hardware/drivers/display/general-unicode-requirement
                let mut utf16_buf = vec![0; buf_size/2];
                let res = decoder.decode_to_utf16(&buf[..read_count], &mut utf16_buf, read_count != buf_size);
                println!("result: {res:?}");
                println!("decoded chars: {:?}", String::from_utf16_lossy(&utf16_buf[..res.2]));
                if let Err(e) = line_reader.read_to_line(&String::from_utf16_lossy(&utf16_buf[..res.2])) {
                    return Err(WinInfFileParseError::ReadLineError(e.to_string()));
                }
            } else {
                // This works if the file is UTF-8
                println!("decoded chars: {:?}", String::from_utf8_lossy(&buf[..read_count]));
                if let Err(e) = line_reader.read_to_line(&String::from_utf8_lossy(&buf[..read_count]).to_string()) {
                    return Err(WinInfFileParseError::ReadLineError(e.to_string()));
                }
            }

            for line in line_reader.take_lines() {
                if let Err(e) = self.read_section(line) {
                    return Err(WinInfFileParseError::SectionParseError(e.to_string()))
                }
            }
        }

        line_reader.finalize();
        for line in line_reader.take_lines() {
            if let Err(e) = self.read_section(line) {
                return Err(WinInfFileParseError::SectionParseError(e.to_string()))
            }
        }

        println!("total lines: {}", line_reader.lines.len());
        for line in line_reader.lines.iter() {
            println!(">> line: {}", line);
        }

        for (section_name, section) in self.sections.iter() {
            println!(">> section name: {}, section: {:?}", section_name, section);
        }

        Ok(())
    }
    fn read_section(&mut self, line: String) -> Result<(), &str> {
        // trim spaces both sides
        let mut line = line.trim_start().trim_end();

        // exclude comments
        if line.starts_with(';') {
            return Ok(());
        }

        // section name
        if line.starts_with('[') && line.ends_with(']') {
            let section_name = line[1..line.len()-1].to_string();
            if let Err(e) = validate_section_name(section_name.clone()) {
                return Err(e.clone());
            }

            // TODO: if there are multiple sections with same name, we have to merge them
            self.sections.insert(section_name.clone(), InfSection{ name: section_name.clone(), entries: vec![] });
            self.last_section_name = section_name.clone();
            return Ok(());
        }

        // entries
        if !self.last_section_name.is_empty() {
            println!("processing entries for section name: {}", self.last_section_name);
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim_start().trim_end();
                let mut value = value.trim_start().trim_end();

                if value.starts_with('"') {
                    println!("processing quoted value: {}", value);
                    let end_double_quote_idx = value[1..].find('"');
                    if end_double_quote_idx.is_none() {
                        return Err("Invalid INF entry value, key:{}, section_name: {}");
                    }
                    let end_double_quote_idx = end_double_quote_idx.unwrap()+1;
                    // check for continuation char
                    if value.len() > end_double_quote_idx {
                        if let Some(c) = value.chars().nth(end_double_quote_idx + 1usize) {
                            if c == '\\' {
                                self.last_entry_value_contd = value[..end_double_quote_idx].to_string();
                                return Ok(());
                            }
                        } else {
                            // TODO: is this error?
                        }
                    }

                    self.last_entry_value_contd.clear();
                    value = &value[..end_double_quote_idx];
                    if let Some(section) = self.sections.get_mut(&self.last_section_name) {
                        let new_entry = InfEntry{ key: key.to_string(), value: Some(InfValue::Raw(value.to_string())) };
                        section.entries.push(new_entry);
                    }

                } else {
                    println!("processing unquoted value: {}", value);

                    // value containing comments at the end
                    if let Some((first, _)) = value.split_once(';') {
                        value = first.trim_start().trim_end();
                    }

                    // multiple backslashes at the end, windows treat only the last one as line continuator and ignores rest
                    if value.ends_with('\\') {
                        println!("processing unquoted contd value: {}", value);
                        if let Some(first_backslash_idx) = value.find('\\') {
                            if first_backslash_idx > 0 {
                                self.last_entry_value_contd = value[..first_backslash_idx].to_string();
                                return Ok(())
                            } else {
                                self.last_entry_value_contd.clear();
                                return Ok(())
                            }
                        }
                    } else {
                        println!("processing unquoted non contd value: {}", value);
                        self.last_entry_value_contd.clear();

                        if let Some(entry) = self.sections.get_mut(&self.last_section_name) {
                            entry.entries.push(InfEntry{ key: key.to_string(), value: Some(InfValue::Raw(value.to_string())) })
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn validate_section_name<'a>(name: String) -> Result<(), &'a str>{
    println!("validate section name: {}", name);
    if name.starts_with('\"') {
        // quoted section name
        // double quotes within are also allowed when escaped
        if !name.ends_with('\"') {
            return Err("invalid quoted section name");
        }

        if name.contains(']') {
            return Err("invalid ] in the quoted section name");
        }

        return Ok(())
    } else {
        // unquoted section name
        if name.ends_with('\\') {
            return Err("invalid \\ at the end of section name");
        }

        if name.matches('%').count() % 2 == 1 {
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



