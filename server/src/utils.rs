use actix_web::HttpRequest;
use serde::Deserialize;

pub fn should_use_english(args: DetailsQuerryArgs, req: HttpRequest) -> bool {
    // we calculate the language from the request by checking if either the query or the cookie are set to en
    let cookie_en = req.cookie("lang").map_or(false, |c| c.value() == "en");
    let arg_en = args.lang.map_or(false, |c| c == "en");
    arg_en || cookie_en
}

#[derive(Deserialize)]
pub struct DetailsQuerryArgs {
    lang: Option<String>,
}
