use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Deserialize, Serialize, Copy, Clone, Debug, Eq, PartialEq, Default, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LanguageOptions {
    #[default]
    De,
    En,
}

impl Display for LanguageOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LanguageOptions::En => f.write_str("en"),
            LanguageOptions::De => f.write_str("de"),
        }
    }
}

#[derive(
    Deserialize, Serialize, Copy, Clone, Debug, Eq, PartialEq, Default, utoipa::IntoParams,
)]
pub struct LangQueryArgs {
    /// The language you want your preview to be in. If either this or the query parameter is set to en, this will be delivered.
    #[serde(default)]
    #[param(inline)]
    pub lang: LanguageOptions,
}
