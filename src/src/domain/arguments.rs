use std::error::Error;
use std::fmt::{self, Display, Formatter};

use super::core::Workspace;
use super::localization::UiLanguage;

pub const ARGUMENT_TOKENS: &[&str] = &[
    "{path}",
    "{name}",
    "{selectfile}",
    "{selectdir}",
    "{inputtext}",
    "{Language}",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArgumentToken {
    Path,
    Name,
    SelectFile,
    SelectDir,
    InputText,
    Language,
}

impl ArgumentToken {
    pub fn from_literal(value: &str) -> Option<Self> {
        match value {
            "{path}" => Some(Self::Path),
            "{name}" => Some(Self::Name),
            "{selectfile}" => Some(Self::SelectFile),
            "{selectdir}" => Some(Self::SelectDir),
            "{inputtext}" => Some(Self::InputText),
            "{Language}" => Some(Self::Language),
            _ => None,
        }
    }

    pub fn literal(self) -> &'static str {
        match self {
            Self::Path => "{path}",
            Self::Name => "{name}",
            Self::SelectFile => "{selectfile}",
            Self::SelectDir => "{selectdir}",
            Self::InputText => "{inputtext}",
            Self::Language => "{Language}",
        }
    }

    pub fn requires_workspace(self) -> bool {
        matches!(self, Self::Path | Self::Name | Self::Language)
    }

    pub fn is_interactive(self) -> bool {
        matches!(self, Self::SelectFile | Self::SelectDir | Self::InputText)
    }
}

pub fn argument_tokens_in_first_appearance_order(arguments: &str) -> Vec<ArgumentToken> {
    let mut tokens = Vec::new();
    let mut search_start = 0;

    while let Some(relative_start) = arguments[search_start..].find('{') {
        let start = search_start + relative_start;
        let token_body_start = start + '{'.len_utf8();
        let Some(relative_end) = arguments[token_body_start..].find('}') else {
            break;
        };
        let end = token_body_start + relative_end + '}'.len_utf8();
        let token = &arguments[start..end];

        if let Some(token) = ArgumentToken::from_literal(token)
            && !tokens.contains(&token)
        {
            tokens.push(token);
        }

        search_start = end;
    }

    tokens
}

pub fn interactive_argument_tokens_in_first_appearance_order(
    arguments: &str,
) -> Vec<ArgumentToken> {
    argument_tokens_in_first_appearance_order(arguments)
        .into_iter()
        .filter(|token| token.is_interactive())
        .collect()
}

pub fn arguments_require_workspace(arguments: &str) -> bool {
    argument_tokens_in_first_appearance_order(arguments)
        .into_iter()
        .any(|token| token.requires_workspace())
}

pub fn resolve_argument_replacements(
    arguments: &str,
    workspace: Option<&Workspace>,
    mut interactive_value: impl FnMut(ArgumentToken) -> Result<Option<String>, String>,
) -> Result<Option<Vec<(ArgumentToken, String)>>, ArgumentResolutionError> {
    let unknown_tokens = unknown_argument_tokens(arguments);
    if !unknown_tokens.is_empty() {
        return Err(ArgumentResolutionError::UnknownTokens(unknown_tokens));
    }

    let tokens = argument_tokens_in_first_appearance_order(arguments);
    if tokens.iter().any(|token| token.requires_workspace()) && workspace.is_none() {
        return Err(ArgumentResolutionError::WorkspaceRequired);
    }

    let mut replacements = Vec::new();
    for token in tokens {
        let value = match token {
            ArgumentToken::Path => workspace
                .map(|workspace| workspace.path.clone())
                .ok_or(ArgumentResolutionError::WorkspaceRequired)?,
            ArgumentToken::Name => workspace
                .map(|workspace| workspace.name.clone())
                .ok_or(ArgumentResolutionError::WorkspaceRequired)?,
            ArgumentToken::Language => workspace
                .map(|workspace| workspace.language.clone())
                .ok_or(ArgumentResolutionError::WorkspaceRequired)?,
            ArgumentToken::SelectFile | ArgumentToken::SelectDir | ArgumentToken::InputText => {
                match interactive_value(token).map_err(ArgumentResolutionError::InteractiveValue)? {
                    Some(value) => value,
                    None => return Ok(None),
                }
            }
        };

        replacements.push((token, value));
    }

    Ok(Some(replacements))
}

pub fn replace_argument_tokens(
    arguments: &str,
    replacements: &[(ArgumentToken, String)],
) -> Result<String, ArgumentReplacementError> {
    let mut replaced = String::with_capacity(arguments.len());
    let mut search_start = 0;

    while let Some(relative_start) = arguments[search_start..].find('{') {
        let start = search_start + relative_start;
        let token_body_start = start + '{'.len_utf8();
        let Some(relative_end) = arguments[token_body_start..].find('}') else {
            break;
        };
        let end = token_body_start + relative_end + '}'.len_utf8();
        let token_literal = &arguments[start..end];

        replaced.push_str(&arguments[search_start..start]);

        let Some(token) = ArgumentToken::from_literal(token_literal) else {
            return Err(ArgumentReplacementError::UnknownToken(
                token_literal.to_owned(),
            ));
        };
        let Some(value) = replacement_value(replacements, token) else {
            return Err(ArgumentReplacementError::MissingTokenValue(token));
        };

        replaced.push_str(value);
        search_start = end;
    }

    replaced.push_str(&arguments[search_start..]);
    Ok(replaced)
}

fn replacement_value(
    replacements: &[(ArgumentToken, String)],
    token: ArgumentToken,
) -> Option<&str> {
    replacements
        .iter()
        .find_map(|(candidate, value)| (*candidate == token).then_some(value.as_str()))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArgumentReplacementError {
    MissingTokenValue(ArgumentToken),
    UnknownToken(String),
}

impl ArgumentReplacementError {
    pub fn user_message(&self) -> String {
        self.user_message_for_language(UiLanguage::Korean)
    }

    pub fn user_message_for_language(&self, language: UiLanguage) -> String {
        match self {
            Self::MissingTokenValue(token) => {
                format!(
                    "{}: {}",
                    language.text(
                        "토큰 값을 준비할 수 없습니다",
                        "Could not prepare token value"
                    ),
                    token.literal()
                )
            }
            Self::UnknownToken(token) => {
                format!(
                    "{}: {token}",
                    language.text("알 수 없는 토큰입니다", "Unknown token")
                )
            }
        }
    }
}

impl Display for ArgumentReplacementError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTokenValue(token) => {
                write!(
                    formatter,
                    "missing argument token value: {}",
                    token.literal()
                )
            }
            Self::UnknownToken(token) => write!(formatter, "unknown argument token: {token}"),
        }
    }
}

impl Error for ArgumentReplacementError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArgumentResolutionError {
    UnknownTokens(Vec<String>),
    WorkspaceRequired,
    InteractiveValue(String),
}

impl ArgumentResolutionError {
    pub fn user_message(&self) -> String {
        self.user_message_for_language(UiLanguage::Korean)
    }

    pub fn user_message_for_language(&self, language: UiLanguage) -> String {
        match self {
            Self::UnknownTokens(tokens) => {
                format!(
                    "{}.\n\n{}",
                    language.text("알 수 없는 토큰입니다", "Unknown token"),
                    tokens.join(", ")
                )
            }
            Self::WorkspaceRequired => language
                .text("워크스페이스를 선택하세요.", "Select a workspace.")
                .to_owned(),
            Self::InteractiveValue(message) => message.clone(),
        }
    }
}

impl Display for ArgumentResolutionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownTokens(tokens) => {
                write!(formatter, "unknown argument tokens: {}", tokens.join(", "))
            }
            Self::WorkspaceRequired => write!(formatter, "workspace is required for arguments"),
            Self::InteractiveValue(message) => {
                write!(formatter, "interactive argument value failed: {message}")
            }
        }
    }
}

impl Error for ArgumentResolutionError {}

pub fn unknown_argument_tokens(arguments: &str) -> Vec<String> {
    let mut unknown = Vec::new();
    let mut search_start = 0;

    while let Some(relative_start) = arguments[search_start..].find('{') {
        let start = search_start + relative_start;
        let token_body_start = start + '{'.len_utf8();
        let Some(relative_end) = arguments[token_body_start..].find('}') else {
            break;
        };
        let end = token_body_start + relative_end + '}'.len_utf8();
        let token = &arguments[start..end];

        if ArgumentToken::from_literal(token).is_none() && !unknown.iter().any(|item| item == token)
        {
            unknown.push(token.to_owned());
        }

        search_start = end;
    }

    unknown
}
