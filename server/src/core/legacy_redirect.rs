use crate::models::DBRoomEntry;
use crate::utils;
use actix_web::{get, web, HttpResponse};
use diesel::prelude::*;
use log::error;

fn extract_redirect_base(key: &DBRoomEntry) -> String {
    match key.type_.as_str() {
        "root" => "".to_string(),
        "campus" => format!("campus/{}", key.key),
        "site" => format!("site/{}", key.key),
        "area" => format!("site/{}", key.key), // Currently also "site", maybe "group"? TODO
        "building" => format!("building/{}", key.key),
        "joined_building" => format!("building/{}", key.key),
        "room" => format!("room/{}", key.key),
        "virtual_room" => format!("room/{}", key.key),
        _ => format!("view/{}", key.key), // can be triggered if we add a type but don't add it here
    }
}

/// the old roomfinder still exists and adoption of our new system is not great.
/// This is a redirect route which can be a direct redirect for the old room-finder.
/// After 1-2 years, we will introduce some text to nudging those,
/// who still have not changed their links, as otherwise we assume this transition will never be done...
/// Said nudge will include information on who to contact if updating the website is not possible and
/// tell the users what link to exchange with what other link.
/// Redirecting to y after a button click or something similar is probably good.
///
/// THIS IS NOT A PERMANENT SOLUTION, AND WILL BE REMOVED IN THE FUTURE
#[get("/legacy_redirect/{arch_name}")]
pub async fn legacy_redirect_handler(params: web::Path<String>) -> HttpResponse {
    let a_name = params.into_inner(); // named like this to not clash with the shema::arch_name

    // collect relevant rooms
    let conn = &mut utils::establish_connection();
    use crate::schema::de::dsl::*;
    let responses = de.filter(arch_name.eq(&a_name)).load::<DBRoomEntry>(conn);

    // map them to the actual redirect_base
    let keys = match responses {
        Ok(r) => r.iter().map(extract_redirect_base).collect::<Vec<String>>(),
        Err(e) => {
            error!("Error requesting details for {}: {:?}", a_name, e);
            return HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Internal Server Error");
        }
    };

    // formulate response
    match keys.len() {
        0 => HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Not found"),
        1 => HttpResponse::MovedPermanently()
            .content_type("text/plain")
            .append_header(("Location", format!("https://nav.tum.de/{}", keys[0])))
            .body(""),
        _ => {
            let msg: String = keys
                .iter()
                .map(|res| format!("- https://nav.tum.de/{}\n", res))
                .collect();
            let base_msg = concat!(
                "Multiple entries found, cannot automatically redirect.\n",
                "If you are this sites admin, please exchange the link with the one you meant.\n",
                "\n",
                "Please choose one of the following:\n"
            );
            HttpResponse::Ok()
                .content_type("text/plain")
                .body(format!("{}{}", base_msg, msg))
        }
    }
}
