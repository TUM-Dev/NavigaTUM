use actix_web::{get, web, HttpResponse};
use rusqlite::{Connection, OpenFlags};

struct TypeAndKey {
    _type: String,
    key: String,
}

fn prepare_redirect(arch_name: String) -> Result<HttpResponse, rusqlite::Error> {
    let conn = Connection::open_with_flags(
        "data/api_data.db",
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .expect("Cannot open database");

    let mut stmt = conn
        .prepare("SELECT type,key FROM de WHERE arch_name = ?")
        .expect("Cannot prepare statement");
    let key_iter = stmt.query_map([arch_name], |row| {
        Ok(TypeAndKey {
            _type: row.get_unwrap(0),
            key: row.get_unwrap(1),
        })
    })?;

    let mut keys = Vec::new();
    // These redirects come from the frontend
    for key in key_iter {
        keys.push(extract_redirect_base(key?));
    }

    match keys.len() {
        0 => Ok(HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Not found")),
        1 => Ok(HttpResponse::MovedPermanently()
            .content_type("text/plain")
            .append_header(("Location", format!("https://nav.tum.de/{}", keys[0])))
            .body("")),
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
            Ok(HttpResponse::Ok()
                .content_type("text/plain")
                .body(format!("{}{}", base_msg, msg)))
        }
    }
}

fn extract_redirect_base(key: TypeAndKey) -> String {
    match key._type.as_str() {
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
    let arch_name = params.into_inner();
    let result = prepare_redirect(arch_name);

    match result {
        Ok(res) => res,
        Err(_) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body("Internal Server Error"),
    }
}
