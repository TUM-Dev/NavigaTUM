use serde::Deserialize;

pub fn should_use_english(args: DetailsQueryArgs) -> bool {
    args.lang.map_or(false, |c| c == "en")
}

#[derive(Deserialize)]
pub struct DetailsQueryArgs {
    lang: Option<String>,
}
