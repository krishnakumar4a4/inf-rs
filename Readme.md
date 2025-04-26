# INF parser in Rust
This is a Windows INF file parser library. Supports UTF-8 & UTF16-LE formats of INF files.

## Features
- Parse Windows INF files in both UTF-8 and UTF-16LE formats
- Support for section-based parsing
- Handle key-value pairs and standalone values
- Support for quoted values and line continuations
- Comprehensive error handling
- UTF-16LE BOM detection and handling
- Debug logging for detailed parsing information

## Usage

Add this to your `Cargo.toml`:
```toml
[dependencies]
inf-rs = "0.1.0"
log = "0.4.20"
env_logger = "0.10.1"  # Optional, for logging configuration
```

Basic usage example:
```rust
use inf_rs::WinInfFile;
use std::path::PathBuf;
use env_logger::Env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger (optional)
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    
    let mut inf_file = WinInfFile::default();
    inf_file.parse(PathBuf::from("path/to/file.inf"))?;
    
    // Access sections
    for (section_name, section) in inf_file.sections.iter() {
        println!("Section: {}", section_name);
        for entry in &section.entries {
            match entry {
                InfEntry::KeyValue(key, value) => {
                    println!("  {} = {:?}", key, value);
                }
                InfEntry::OnlyValue(value) => {
                    println!("  {:?}", value);
                }
            }
        }
    }
    
    Ok(())
}
```

## Logging
The library uses the `log` crate for debug logging. To see debug messages, you can:
1. Use `env_logger` as shown in the example above
2. Set the `RUST_LOG` environment variable to `debug` or `trace`
3. Or configure your own logger implementation

## About UTF16-LE
1. INF files are UTF16-LE (unicode 16 Little Endian) format. Ref https://learn.microsoft.com/en-us/windows-hardware/drivers/display/general-unicode-requirement
2. How UTF-16 LE works
   - Each Unicode code point is encoded using either one or two 16-bit code units
   - Code points less than 216 are encoded with a single 16-bit code unit
   - Code points greater than or equal to 216 are encoded using two 16-bit code units
   - The two 16-bit code units for code points greater than or equal to 216 are called a surrogate pair

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.