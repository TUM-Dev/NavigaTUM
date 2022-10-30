use serde::Deserialize;

#[derive(Deserialize)]
pub struct LangQueryArgs {
    lang: Option<String>,
}

impl LangQueryArgs {
    pub fn should_use_english(&self) -> bool {
        self.lang.as_ref().map_or(false, |c| c == "en")
    }
}
