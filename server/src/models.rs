use diesel::prelude::*;

#[derive(Queryable, Debug, Clone)]
pub struct DBRoomEntry {
    pub key: String,
    pub name: String,
    pub arch_name: Option<String>,
    pub type_: String,
    pub type_common_name: String,
    pub lat: f32,
    pub lon: f32,
    pub data: String,
}
