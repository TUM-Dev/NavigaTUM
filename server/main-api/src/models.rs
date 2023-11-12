#[derive(Debug, Clone)]
pub struct DBRoomEntry {
    pub key: String,
    pub name: String,
    pub tumonline_room_nr: Option<i64>,
    pub r#type: String,
    pub type_common_name: String,
    pub lat: f64,
    pub lon: f64,
    pub data: String,
}

#[derive(Debug, Clone)]
pub struct DBRoomKeyAlias {
    pub key: String,
    pub visible_id: String,
    pub r#type: String,
}
