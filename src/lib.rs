#![feature(str_from_utf16_endian)]

mod types;

use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, Read};
use std::path::PathBuf;
use encoding_rs::{Encoding, UTF_16LE};
use nom::character::complete::{char, line_ending, not_line_ending};
use nom::Parser;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::preceded;
use crate::types::InfSection;
use crate::WinInfFileParseError::{FileDoNotExist, FileOpenError, FileReadError};

pub struct WinInfFile {
    pub sections: HashMap<String, InfSection>,
    pub remaining_string: String,
    pub lines: Vec<String>
}

#[derive(Debug)]
pub enum WinInfFileParseError {
    FileDoNotExist(),
    FileOpenError(Error),
    FileReadError(),
    ReadLineError(String),
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
                if let Err(e) = self.read_line(&String::from_utf16_lossy(&utf16_buf[..res.2])) {
                    return Err(WinInfFileParseError::ReadLineError(e.to_string()));
                }
            } else {
                // This works if the file is UTF-8
                println!("decoded chars: {:?}", String::from_utf8_lossy(&buf[..read_count]));
                if let Err(e) = self.read_line(&String::from_utf8_lossy(&buf[..read_count]).to_string()) {
                    return Err(WinInfFileParseError::ReadLineError(e.to_string()));
                }
            }
        }

        if !self.remaining_string.is_empty() {
            self.lines.push(self.remaining_string.clone());
        }

        println!("total lines: {}", self.lines.len());
        for line in self.lines.iter() {
            println!(">> line: {}", line);
        }

        Ok(())
    }

    fn read_line(&mut self, input: &str) -> Result<(), &str>{
        self.remaining_string.push_str(input);
        let mut found_cr = false;
        let mut new_line = String::from("");
        for (i, c) in self.remaining_string.clone().chars().enumerate() {
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
}



