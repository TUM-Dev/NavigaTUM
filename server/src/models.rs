#[derive(Debug, Clone)]
#[allow(dead_code)] // false positive. Clippy can't detect this due to macros
pub struct LocationKeyAlias {
    pub key: String,
    pub visible_id: String,
    pub r#type: String,
}
