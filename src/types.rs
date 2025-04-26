use std::collections::HashMap;

/// A value in a Windows INF file
/// 
/// Values can be either raw strings or lists of strings.
/// Raw strings are used for simple values, while lists are used for
/// multi-line values or arrays.
#[derive(Debug, Clone, PartialEq)]
pub enum InfValue {
    // INF values can be complex, like comma-separated lists or numbers.
    // For simplicity, we'll treat most as strings initially.
    // You could extend this to handle specific value types if needed.
    CommaSeparated(Vec<String>),
    /// A raw string value
    /// 
    /// This variant is used for simple string values in the INF file.
    Raw(String),
    /// A list of string values
    /// 
    /// This variant is used for multi-line values or arrays in the INF file.
    List(Vec<String>),
}

/// An entry in a Windows INF file section
/// 
/// Entries can be either key-value pairs or standalone values.
/// Key-value pairs are used for configuration settings, while standalone values
/// are often used for lists or simple values.
#[derive(Debug, Clone, PartialEq)]
pub enum InfEntry {
    /// A key-value pair entry
    /// 
    /// The first field is the key, and the second field is an optional value.
    /// The value is optional because some INF files may have keys without values.
    KeyValue(String, Option<InfValue>),
    /// A standalone value entry
    /// 
    /// This variant is used for entries that don't have a key, such as
    /// list items or simple values.
    OnlyValue(InfValue),
}

/// A section in a Windows INF file
/// 
/// Each section in an INF file has a name and contains a list of entries.
/// Entries can be either key-value pairs or standalone values.
#[derive(Debug, Clone, PartialEq)]
pub struct InfSection {
    /// The name of the section
    pub name: String,
    /// The entries contained in this section
    pub entries: Vec<InfEntry>,
}

#[derive(Debug, PartialEq)]
pub struct InfFile {
    pub sections: HashMap<String, InfSection>,
    pub strings: HashMap<String, String>,
}