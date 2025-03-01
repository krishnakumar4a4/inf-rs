#![feature(str_from_utf16_endian)]

use std::fs::File;
use std::io::{Error, Read};
use std::path::PathBuf;
use encoding_rs::{Encoding, UTF_16LE};
use crate::WinInfFileParseError::{FileDoNotExist, FileOpenError, FileReadError};

pub struct WinInfFile {}

#[derive(Debug)]
pub enum WinInfFileParseError {
    FileDoNotExist(),
    FileOpenError(Error),
    FileReadError()
}

impl WinInfFile {
    pub fn parse(file_path: PathBuf) -> Result<Self, WinInfFileParseError> {
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
            } else {
                // This works if the file is UTF-8
                println!("decoded chars: {:?}", String::from_utf8_lossy(&buf[..read_count]));
            }
        }

        Ok(WinInfFile{})
    }
}

