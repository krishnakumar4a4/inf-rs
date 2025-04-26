use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum InfValue {
    // INF values can be complex, like comma-separated lists or numbers.
    // For simplicity, we'll treat most as strings initially.
    // You could extend this to handle specific value types if needed.
    CommaSeparated(Vec<String>),
    // Representing a raw, unparsed value for now
    Raw(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InfEntry {
    KeyValue(String, Option<InfValue>),
    OnlyValue(InfValue),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InfSection {
    pub name: String,
    pub entries: Vec<InfEntry>,
}

#[derive(Debug, PartialEq)]
pub struct InfFile {
    pub sections: HashMap<String, InfSection>,
    pub strings: HashMap<String, String>,
}