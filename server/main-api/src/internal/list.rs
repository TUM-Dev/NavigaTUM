use crate::utils;
use actix_web::{get, web};
use diesel::prelude::*;
use log::error;

#[get("/list/ids_with_calendar")]
pub async fn ids_with_calendar() -> web::Json<Vec<(String, i32)>> {
    let conn = &mut utils::establish_connection();

    use crate::schema::de::dsl::*;
    // order is just here, to make debugging more reproducible.
    // Performance impact is negligible
    let res = de
        .select((key, tumonline_room_nr))
        .filter(tumonline_room_nr.is_not_null())
        .order_by((key, tumonline_room_nr))
        .load::<(String, Option<i32>)>(conn);
    web::Json(match res {
        Ok(d) => d
            .iter()
            .map(|(k, t)| (k.clone(), t.unwrap()))
            .collect::<Vec<(String, i32)>>(),
        Err(e) => {
            error!("Error requesting all ids: {e:?}");
            vec![]
        }
    })
}
