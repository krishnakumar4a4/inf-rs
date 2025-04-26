use std::env::current_dir;
use inf_rs::WinInfFile;

fn main() {
    let cwd = current_dir().unwrap();
    let inf_path = cwd.join("sampledisplay.inf"); //UTF-8
    // let inf_path = cwd.join("AudioCodec.inf"); //UTF16
    println!("inf: {:?}", inf_path);
    let mut inf_file = WinInfFile::default();
    inf_file.parse(inf_path).unwrap();
}