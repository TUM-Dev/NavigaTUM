use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Copy, Clone, Debug, Eq, PartialEq, Default, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum LanguageOptions {
    #[default]
    De,
    En,
}

#[derive(
    Deserialize,
    Serialize,
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Default,
    utoipa::IntoParams,
    utoipa::ToSchema,
)]
#[serde(default)]
pub struct LangQueryArgs {
    /// The language you want your preview to be in. If either this or the query parameter is set to en, this will be delivered.
    lang: LanguageOptions,
}

impl LangQueryArgs {
    pub fn should_use_english(self) -> bool {
        self.lang == LanguageOptions::En
    }
    pub fn serialise(self) -> String {
        match self.lang {
            LanguageOptions::En => "en".to_string(),
            LanguageOptions::De => "de".to_string(),
        }
    }
}
