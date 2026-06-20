pub const DEFAULT_UI_LANGUAGE: &str = "en";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UiLanguage {
    Korean,
    #[default]
    English,
}

const UI_LANGUAGE_OPTIONS: [UiLanguage; 2] = [UiLanguage::Korean, UiLanguage::English];

impl UiLanguage {
    pub fn text(self, korean: &'static str, english: &'static str) -> &'static str {
        match self {
            Self::Korean => korean,
            Self::English => english,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Korean => "한국어",
            Self::English => "English",
        }
    }

    pub fn display_name_for(self, current_language: UiLanguage) -> &'static str {
        match current_language {
            UiLanguage::Korean => match self {
                Self::Korean => "한국어",
                Self::English => "영어",
            },
            UiLanguage::English => match self {
                Self::Korean => "Korean",
                Self::English => "English",
            },
        }
    }

    pub fn as_config_value(self) -> &'static str {
        match self {
            Self::Korean => "ko",
            Self::English => "en",
        }
    }

    pub fn from_config_value(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "ko" | "kr" | "kor" | "korean" | "한국어" | "한글" => Some(Self::Korean),
            "en" | "eng" | "english" => Some(Self::English),
            "default" => Some(Self::default()),
            _ => None,
        }
    }

    pub fn options() -> &'static [Self] {
        &UI_LANGUAGE_OPTIONS
    }
}
