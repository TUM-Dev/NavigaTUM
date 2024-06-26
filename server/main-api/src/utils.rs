use serde::Deserialize;

#[derive(Deserialize, Copy, Clone, Debug, Eq, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
enum LanguageOptions {
    #[default]
    De,
    En,
}

#[derive(Deserialize, Copy, Clone, Debug, Eq, PartialEq, Default)]
#[serde(default)]
pub struct LangQueryArgs {
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
