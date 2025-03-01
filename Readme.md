# INF parser in Rust
This is a Windows INF file parser library.

## Details
1. INF files are UTF16-lE (unicode 16 Little Endian) format. Ref https://learn.microsoft.com/en-us/windows-hardware/drivers/display/general-unicode-requirement
2. How UTF-16 LE works
   - Each Unicode code point is encoded using either one or two 16-bit code units
   - Code points less than 216 are encoded with a single 16-bit code unit
   - Code points greater than or equal to 216 are encoded using two 16-bit code units
   - The two 16-bit code units for code points greater than or equal to 216 are called a surrogate pair
- 