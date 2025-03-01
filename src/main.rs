use std::env::current_dir;
use std::path::{Path, PathBuf};
use inf_rs::{WinInfFile, WinInfFileParseError};

fn main() {
    let cwd = current_dir().unwrap();
    let inf_path = cwd.join("sampledisplay.inf"); //UTF-8
    // let inf_path = cwd.join("AudioCodec.inf"); //UTF16
    println!("inf: {:?}", inf_path);
    WinInfFile::parse(inf_path).unwrap();
}