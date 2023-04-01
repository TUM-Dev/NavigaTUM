use log::error;

fn api_url_from_env() -> Option<String> {
    let main_api_addr = std::env::var("API_SVC_SERVICE_HOST").ok()?;
    let main_api_port = std::env::var("API_SVC_SERVICE_PORT_API").ok()?;

    Some(format!(
        "http://{main_api_addr}:{main_api_port}/internal/list/ids_with_calendar"
    ))
}

pub struct ReducedRoom {
    pub key: String,
    pub tumonline_room_nr: i32,
}

pub async fn get_all_ids() -> Vec<ReducedRoom> {
    // returns all (key, tumonline_room_nr) from the main-api
    let url = api_url_from_env()
        .unwrap_or_else(|| "https://nav.tum.de/internal/list/ids_with_calendar".to_string());
    let res = reqwest::get(url).await;
    let text = match res {
        Ok(res) => res.text().await,
        Err(e) => {
            error!("Failed to contact main-api: {e:#?}");
            return vec![];
        }
    };
    match text {
        Ok(ids) => serde_json::from_slice::<Vec<(String, i32)>>(ids.as_bytes())
            .expect("Failed to parse json, make sure to pass the correct schema on both sides"),
        Err(e) => {
            error!("Failed to process text get all ids from api: {e:#?}");
            vec![]
        }
    }
    .into_iter()
    .map(|(key, tumonline_room_nr)| ReducedRoom {
        key,
        tumonline_room_nr,
    })
    .collect()
}
