use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

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
    #[param(inline)]
    lang: LanguageOptions,
}

impl LangQueryArgs {
    pub fn should_use_english(self) -> bool {
        self.lang == LanguageOptions::En
    }
}
impl Display for LangQueryArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.lang {
            LanguageOptions::En => f.write_str("en"),
            LanguageOptions::De => f.write_str("de"),
        }
    }
}
