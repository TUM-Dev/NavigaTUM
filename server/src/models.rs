use diesel::prelude::*;

#[derive(Queryable)]
pub struct De {
    pub key: String,
    pub name: String,
    pub arch_name: Option<String>,
    pub type_: String,
    pub type_common_name: String,
    pub lat: f32,
    pub lon: f32,
    pub data: String,
}

#[derive(Queryable)]
pub struct En {
    pub key: String,
    pub name: String,
    pub arch_name: String,
    pub type_: String,
    pub type_common_name: String,
    pub lat: f32,
    pub lon: f32,
    pub data: String,
}
