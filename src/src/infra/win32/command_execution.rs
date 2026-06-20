use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::ptr::null;

use windows_sys::Win32::Foundation::{GetLastError, HINSTANCE, HWND};
use windows_sys::Win32::Graphics::Gdi::HFONT;
use windows_sys::Win32::System::Diagnostics::Debug::OutputDebugStringW;
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW;

use crate::domain::{
    AppState, ArgumentReplacementError, ArgumentResolutionError, ArgumentToken, CommandButton,
    ExecutionType, UiLanguage, Workspace, arguments_require_workspace, replace_argument_tokens,
    resolve_argument_replacements,
};

use super::{
    browse_for_folder, browse_for_selected_file, is_accessible_folder,
    show_argument_text_input_dialog, show_error_message, show_warning_message, tr, wide_null,
};

#[derive(Clone, Copy)]
pub(super) struct CommandExecutionUi {
    instance: HINSTANCE,
    font: HFONT,
    font_size: u16,
    language: UiLanguage,
}

impl CommandExecutionUi {
    pub(super) fn new(
        instance: HINSTANCE,
        font: HFONT,
        font_size: u16,
        language: UiLanguage,
    ) -> Self {
        Self {
            instance,
            font,
            font_size,
            language,
        }
    }
}

#[derive(Debug)]
pub(super) enum CommandExecutionError {
    ArgumentResolution {
        source: ArgumentResolutionError,
    },
    ArgumentReplacement {
        source: ArgumentReplacementError,
    },
    MissingWorkspaceForExternalTerminal,
    InaccessibleWorkspacePath {
        path: String,
    },
    EmptyExecutablePath {
        executable_path: String,
    },
    InteriorNul {
        field: &'static str,
        value: String,
    },
    ShellExecuteFailed {
        result_code: isize,
        last_error_code: u32,
        executable_path: String,
        arguments: String,
        directory: Option<String>,
    },
}

impl CommandExecutionError {
    #[cfg(test)]
    pub(super) fn user_message(&self) -> String {
        self.user_message_for_language(UiLanguage::Korean)
    }

    fn user_message_for_language(&self, language: UiLanguage) -> String {
        match self {
            Self::ArgumentResolution { source } => source.user_message_for_language(language),
            Self::ArgumentReplacement { source } => source.user_message_for_language(language),
            Self::MissingWorkspaceForExternalTerminal => language
                .text("워크스페이스를 선택하세요.", "Select a workspace.")
                .to_owned(),
            Self::InaccessibleWorkspacePath { path } => {
                format!(
                    "{}: {path}",
                    language.text(
                        "워크스페이스 폴더를 열 수 없습니다",
                        "Could not open the workspace folder"
                    )
                )
            }
            Self::EmptyExecutablePath { .. } => language
                .text("실행 대상을 입력하세요.", "Enter an executable.")
                .to_owned(),
            Self::InteriorNul { field, .. } => {
                format!(
                    "{}: {field}",
                    language.text(
                        "실행 값에 사용할 수 없는 문자가 있습니다",
                        "The execution value contains an unsupported character"
                    )
                )
            }
            Self::ShellExecuteFailed {
                result_code,
                last_error_code,
                ..
            } => format!(
                "{}\n\n{}\n{}: {result_code}, Windows: {last_error_code}",
                language.text("명령 실행 실패", "Command failed"),
                shell_execute_error_message_for_language(language, *result_code),
                language.text("코드", "Code"),
            ),
        }
    }
}

impl Display for CommandExecutionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArgumentResolution { source } => {
                write!(formatter, "command argument resolution failed: {source}")
            }
            Self::ArgumentReplacement { source } => {
                write!(formatter, "command argument replacement failed: {source}")
            }
            Self::MissingWorkspaceForExternalTerminal => {
                write!(
                    formatter,
                    "external_terminal command requires a selected workspace"
                )
            }
            Self::InaccessibleWorkspacePath { path } => {
                write!(formatter, "workspace path is not accessible: {path}")
            }
            Self::EmptyExecutablePath { executable_path } => {
                write!(formatter, "empty executable_path: {executable_path:?}")
            }
            Self::InteriorNul { field, value } => {
                write!(
                    formatter,
                    "command execution value contains interior NUL: field={field}, value={value:?}"
                )
            }
            Self::ShellExecuteFailed {
                result_code,
                last_error_code,
                executable_path,
                arguments,
                directory,
            } => write!(
                formatter,
                "ShellExecuteW failed: return_code={result_code}, GetLastError={last_error_code}, executable_path={}, arguments={}, directory={}",
                shell_execute_failure_value_display(Some(executable_path)),
                shell_execute_failure_value_display(Some(arguments)),
                shell_execute_failure_value_display(directory.as_deref())
            ),
        }
    }
}

impl Error for CommandExecutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ArgumentResolution { source } => Some(source),
            Self::ArgumentReplacement { source } => Some(source),
            Self::MissingWorkspaceForExternalTerminal
            | Self::InaccessibleWorkspacePath { .. }
            | Self::EmptyExecutablePath { .. }
            | Self::InteriorNul { .. }
            | Self::ShellExecuteFailed { .. } => None,
        }
    }
}

/// Executes the selected command button without requiring the full main window context.
///
/// Returns false when the selected button could not be resolved and the caller should refresh
/// command menu state for a missing selection.
pub(super) fn execute_selected_command_button(
    owner: HWND,
    state: &mut AppState,
    ui: CommandExecutionUi,
) -> bool {
    let Some(button) = state.selected_command_button().cloned() else {
        state.select_command_button(None);
        show_warning_message(
            owner,
            tr(ui.language, "명령", "Command"),
            tr(
                ui.language,
                "선택한 명령을 찾을 수 없습니다.",
                "The selected command could not be found.",
            ),
        );
        return false;
    };
    let workspace = state.selected_workspace().cloned();

    if let Some(error) = command_workspace_error(&button, workspace.as_ref()) {
        let message = error.user_message_for_language(ui.language);
        state.set_status_message(message.clone());
        show_warning_message(owner, tr(ui.language, "명령 실행", "Run Command"), &message);
        return true;
    }

    let arguments = match prepare_command_arguments(
        owner,
        ui.instance,
        ui.font,
        ui.font_size,
        ui.language,
        &button,
        workspace.as_ref(),
    ) {
        Ok(Some(arguments)) => arguments,
        Ok(None) => {
            state.set_status_message(tr(ui.language, "실행 취소", "Run canceled"));
            return true;
        }
        Err(error) => {
            let message = error.user_message_for_language(ui.language);
            state.set_status_message(message.clone());
            show_error_message(owner, tr(ui.language, "명령 실행", "Run Command"), &message);
            return true;
        }
    };

    let result = match button.execution_type {
        ExecutionType::ShellApi => {
            execute_shell_api_command(owner, &button.executable_path, &arguments)
        }
        ExecutionType::ExternalTerminal => match workspace.as_ref() {
            Some(workspace) => execute_external_terminal_command(
                owner,
                &button.executable_path,
                &arguments,
                workspace,
            ),
            None => Err(CommandExecutionError::MissingWorkspaceForExternalTerminal),
        },
    };

    match result {
        Ok(()) => {
            state.set_status_message(match ui.language {
                UiLanguage::Korean => format!("실행 완료: {}", button.button_name),
                UiLanguage::English => format!("Run completed: {}", button.button_name),
            });
        }
        Err(error) => {
            let message = error.user_message_for_language(ui.language);
            state.set_status_message(message.clone());
            show_error_message(owner, tr(ui.language, "명령 실행", "Run Command"), &message);
        }
    }

    true
}

pub(super) fn command_workspace_error(
    button: &CommandButton,
    workspace: Option<&Workspace>,
) -> Option<CommandExecutionError> {
    let requires_workspace = button.execution_type == ExecutionType::ExternalTerminal
        || arguments_require_workspace(&button.arguments);
    if !requires_workspace {
        return None;
    }

    let Some(workspace) = workspace else {
        return Some(match button.execution_type {
            ExecutionType::ExternalTerminal => {
                CommandExecutionError::MissingWorkspaceForExternalTerminal
            }
            ExecutionType::ShellApi => CommandExecutionError::ArgumentResolution {
                source: ArgumentResolutionError::WorkspaceRequired,
            },
        });
    };

    if is_accessible_folder(&workspace.path) {
        None
    } else {
        Some(CommandExecutionError::InaccessibleWorkspacePath {
            path: workspace.path.clone(),
        })
    }
}

pub(super) fn prepare_command_arguments(
    owner: HWND,
    instance: HINSTANCE,
    font: HFONT,
    font_size: u16,
    language: UiLanguage,
    button: &CommandButton,
    workspace: Option<&Workspace>,
) -> Result<Option<String>, CommandExecutionError> {
    let Some(replacements) = resolve_command_argument_replacements(
        owner, instance, font, font_size, language, button, workspace,
    )?
    else {
        return Ok(None);
    };

    let replacements = replacements
        .into_iter()
        .map(|(token, value)| {
            (
                token,
                argument_token_execution_value(button.execution_type, token, value),
            )
        })
        .collect::<Vec<_>>();

    replace_argument_tokens(&button.arguments, &replacements)
        .map(Some)
        .map_err(|source| CommandExecutionError::ArgumentReplacement { source })
}

fn resolve_command_argument_replacements(
    owner: HWND,
    instance: HINSTANCE,
    font: HFONT,
    font_size: u16,
    language: UiLanguage,
    button: &CommandButton,
    workspace: Option<&Workspace>,
) -> Result<Option<Vec<(ArgumentToken, String)>>, CommandExecutionError> {
    resolve_argument_replacements(&button.arguments, workspace, |token| {
        let value = match token {
            ArgumentToken::SelectFile => {
                match browse_for_selected_file(owner, language)
                    .map(|path| path.display().to_string())
                {
                    Some(value) => value,
                    None => return Ok(None),
                }
            }
            ArgumentToken::SelectDir => {
                match browse_for_folder(owner, tr(language, "폴더 선택", "Select Folder"))
                    .map(|path| path.display().to_string())
                {
                    Some(value) => value,
                    None => return Ok(None),
                }
            }
            ArgumentToken::InputText => {
                match show_argument_text_input_dialog(owner, instance, font, font_size, language)
                    .map_err(|source| source.to_string())?
                {
                    Some(value) => value,
                    None => return Ok(None),
                }
            }
            ArgumentToken::Path | ArgumentToken::Name | ArgumentToken::Language => {
                return Err(format!("unexpected interactive token: {}", token.literal()));
            }
        };

        Ok(Some(value))
    })
    .map_err(|source| CommandExecutionError::ArgumentResolution { source })
}

pub(super) fn argument_token_execution_value(
    execution_type: ExecutionType,
    _token: ArgumentToken,
    value: String,
) -> String {
    match execution_type {
        ExecutionType::ShellApi => quote_windows_command_argument(&value),
        ExecutionType::ExternalTerminal => quote_cmd_command_argument(&value),
    }
}

fn execute_shell_api_command(
    owner: HWND,
    executable_path: &str,
    arguments: &str,
) -> Result<(), CommandExecutionError> {
    shell_execute_open(owner, executable_path, arguments, None)
}

pub(super) fn execute_external_terminal_command(
    owner: HWND,
    executable_path: &str,
    arguments: &str,
    workspace: &Workspace,
) -> Result<(), CommandExecutionError> {
    let command_line = command_line_from_executable_and_arguments(executable_path, arguments);
    let parameters = external_terminal_parameters_for_command(&command_line);
    let command_processor = command_processor_path();
    shell_execute_open(
        owner,
        &command_processor,
        &parameters,
        Some(&workspace.path),
    )
}

pub(super) fn command_line_from_executable_and_arguments(
    executable_path: &str,
    arguments: &str,
) -> String {
    let executable = quote_cmd_executable_path(executable_path);
    if arguments.is_empty() {
        executable
    } else {
        format!("{executable} {arguments}")
    }
}

pub(super) fn external_terminal_parameters_for_command(command_line: &str) -> String {
    format!("/D /V:OFF /S /K \"{command_line}\"")
}

fn command_processor_path() -> String {
    std::env::var("ComSpec")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "cmd.exe".to_owned())
}

pub(super) fn shell_execute_open(
    owner: HWND,
    executable_path: &str,
    arguments: &str,
    directory: Option<&str>,
) -> Result<(), CommandExecutionError> {
    let executable_path_input = executable_path.to_owned();
    let executable_path = executable_path.trim();
    if executable_path.is_empty() {
        return Err(CommandExecutionError::EmptyExecutablePath {
            executable_path: executable_path_input,
        });
    }
    reject_shell_execute_interior_nul("executable_path", executable_path)?;
    reject_shell_execute_interior_nul("arguments", arguments)?;

    let directory_value = directory
        .filter(|directory| !directory.trim().is_empty())
        .map(str::to_owned);
    if let Some(directory) = directory_value.as_deref() {
        reject_shell_execute_interior_nul("directory", directory)?;
    }
    let operation = wide_null("open");
    let executable_path = wide_null(executable_path);
    let parameters = (!arguments.is_empty()).then(|| wide_null(arguments));
    let directory = directory_value.as_deref().map(wide_null);
    let parameters_ptr = parameters
        .as_ref()
        .map_or(null(), |parameters| parameters.as_ptr());
    let directory_ptr = directory
        .as_ref()
        .map_or(null(), |directory| directory.as_ptr());

    // SAFETY: owner is the main window, strings are null-terminated and live for the call, and
    // ShellExecuteW is used at the platform boundary to ask Windows to execute the target.
    let result = unsafe {
        ShellExecuteW(
            owner,
            operation.as_ptr(),
            executable_path.as_ptr(),
            parameters_ptr,
            directory_ptr,
            SW_SHOW,
        )
    };
    let result_code = result as isize;
    if result_code > 32 {
        Ok(())
    } else {
        // SAFETY: GetLastError reads the calling thread's last-error code after ShellExecuteW.
        let last_error_code = unsafe { GetLastError() };
        let executable_path_display =
            redacted_shell_execute_failure_value(Some(executable_path_input.trim()));
        let arguments_display = redacted_shell_execute_failure_value(Some(arguments));
        let directory_display = directory_value
            .as_deref()
            .map(|directory| redacted_shell_execute_failure_value(Some(directory)));
        log_shell_execute_failure(
            result_code,
            last_error_code,
            &executable_path_display,
            &arguments_display,
            directory_display.as_deref(),
        );
        Err(CommandExecutionError::ShellExecuteFailed {
            result_code,
            last_error_code,
            executable_path: executable_path_display,
            arguments: arguments_display,
            directory: directory_display,
        })
    }
}

fn reject_shell_execute_interior_nul(
    field: &'static str,
    value: &str,
) -> Result<(), CommandExecutionError> {
    if value.contains('\0') {
        Err(CommandExecutionError::InteriorNul {
            field,
            value: value.to_owned(),
        })
    } else {
        Ok(())
    }
}

const SHELL_EXECUTE_FIELD_REDACTED: &str = "<redacted>";
const SHELL_EXECUTE_FIELD_EMPTY: &str = "<empty>";
const SHELL_EXECUTE_FIELD_NOT_PROVIDED: &str = "<not provided>";

fn redacted_shell_execute_failure_value(value: Option<&str>) -> String {
    shell_execute_failure_value_display(value).to_owned()
}

fn shell_execute_failure_value_display(value: Option<&str>) -> &'static str {
    match value {
        Some(SHELL_EXECUTE_FIELD_REDACTED) => SHELL_EXECUTE_FIELD_REDACTED,
        Some(SHELL_EXECUTE_FIELD_EMPTY) => SHELL_EXECUTE_FIELD_EMPTY,
        Some(SHELL_EXECUTE_FIELD_NOT_PROVIDED) => SHELL_EXECUTE_FIELD_NOT_PROVIDED,
        Some("") => SHELL_EXECUTE_FIELD_EMPTY,
        Some(_) => SHELL_EXECUTE_FIELD_REDACTED,
        None => SHELL_EXECUTE_FIELD_NOT_PROVIDED,
    }
}

fn log_shell_execute_failure(
    result_code: isize,
    last_error_code: u32,
    executable_path: &str,
    arguments: &str,
    directory: Option<&str>,
) {
    let executable_path = shell_execute_failure_value_display(Some(executable_path));
    let arguments = shell_execute_failure_value_display(Some(arguments));
    let directory = shell_execute_failure_value_display(directory);
    log_platform_message(&format!(
        "ShellExecuteW failed: return_code={result_code}, GetLastError={last_error_code}, executable_path={executable_path}, arguments={arguments}, directory={directory}"
    ));
}

fn log_platform_message(message: &str) {
    eprintln!("{message}");
    let message = wide_null(message);
    // SAFETY: message is a null-terminated UTF-16 string and lives for the duration of the call.
    unsafe {
        OutputDebugStringW(message.as_ptr());
    }
}

fn shell_execute_error_message_for_language(language: UiLanguage, code: isize) -> String {
    match code {
        0 => language
            .text(
                "실행할 시스템 리소스가 부족합니다.",
                "There are not enough system resources to run it.",
            )
            .to_owned(),
        2 => language
            .text(
                "지정한 파일을 찾을 수 없습니다.",
                "The specified file could not be found.",
            )
            .to_owned(),
        3 => language
            .text(
                "지정한 경로를 찾을 수 없습니다.",
                "The specified path could not be found.",
            )
            .to_owned(),
        5 => language
            .text("접근이 거부되었습니다.", "Access was denied.")
            .to_owned(),
        8 => language
            .text("메모리가 부족합니다.", "Out of memory.")
            .to_owned(),
        26 => language
            .text(
                "공유 위반 때문에 실행할 수 없습니다.",
                "Could not run because of a sharing violation.",
            )
            .to_owned(),
        27 => language
            .text(
                "파일 연결이 올바르지 않습니다.",
                "The file association is invalid.",
            )
            .to_owned(),
        28 => language
            .text(
                "DDE 트랜잭션 시간이 초과되었습니다.",
                "The DDE transaction timed out.",
            )
            .to_owned(),
        29 => language
            .text(
                "DDE 트랜잭션에 실패했습니다.",
                "The DDE transaction failed.",
            )
            .to_owned(),
        30 => language
            .text(
                "다른 DDE 트랜잭션이 처리 중입니다.",
                "Another DDE transaction is in progress.",
            )
            .to_owned(),
        31 => language
            .text(
                "이 파일 형식에 연결된 실행 프로그램이 없습니다.",
                "There is no app associated with this file type.",
            )
            .to_owned(),
        32 => language
            .text(
                "동적 연결 라이브러리를 찾을 수 없습니다.",
                "The dynamic-link library could not be found.",
            )
            .to_owned(),
        other => match language {
            UiLanguage::Korean => format!("ShellExecuteW 오류 코드 {other}"),
            UiLanguage::English => format!("ShellExecuteW error code {other}"),
        },
    }
}

pub(super) fn quote_windows_command_argument(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_owned();
    }

    if !value.chars().any(requires_windows_command_quote) {
        return value.to_owned();
    }

    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    let mut backslashes = 0;

    for character in value.chars() {
        match character {
            '\\' => backslashes += 1,
            '"' => {
                for _ in 0..(backslashes * 2 + 1) {
                    quoted.push('\\');
                }
                quoted.push('"');
                backslashes = 0;
            }
            _ => {
                for _ in 0..backslashes {
                    quoted.push('\\');
                }
                backslashes = 0;
                quoted.push(character);
            }
        }
    }

    for _ in 0..(backslashes * 2) {
        quoted.push('\\');
    }
    quoted.push('"');
    quoted
}

fn requires_windows_command_quote(character: char) -> bool {
    character.is_whitespace() || matches!(character, '"' | '&' | '|' | '<' | '>' | '(' | ')' | '^')
}

fn quote_cmd_command_argument(value: &str) -> String {
    let windows_argument = quote_windows_command_argument(value);
    escape_cmd_command_token(&windows_argument)
}

fn quote_cmd_executable_path(value: &str) -> String {
    let windows_argument = quote_windows_command_argument(value);
    escape_cmd_executable_path(&windows_argument)
}

fn escape_cmd_executable_path(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if character == '%' {
            escaped.push('^');
        }
        escaped.push(character);
    }
    escaped
}

fn escape_cmd_command_token(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if requires_cmd_escape(character) {
            escaped.push('^');
        }
        escaped.push(character);
    }
    escaped
}

fn requires_cmd_escape(character: char) -> bool {
    matches!(
        character,
        '"' | '%' | '!' | '&' | '|' | '<' | '>' | '(' | ')' | '^'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_execute_failed_display_redacts_command_values() {
        let error = CommandExecutionError::ShellExecuteFailed {
            result_code: 2,
            last_error_code: 5,
            executable_path: r"C:\secret\tool.exe".to_owned(),
            arguments: "--token super-secret".to_owned(),
            directory: Some(r"C:\secret\workspace".to_owned()),
        };

        let message = error.to_string();

        assert!(message.contains("executable_path=<redacted>"));
        assert!(message.contains("arguments=<redacted>"));
        assert!(message.contains("directory=<redacted>"));
        assert!(!message.contains("super-secret"));
        assert!(!message.contains("secret\\"));
    }

    #[test]
    fn command_execution_user_message_keeps_korean_default_and_uses_requested_language() {
        let error = CommandExecutionError::MissingWorkspaceForExternalTerminal;

        assert_eq!(error.user_message(), "워크스페이스를 선택하세요.");
        assert_eq!(
            error.user_message_for_language(UiLanguage::English),
            "Select a workspace."
        );
    }

    #[test]
    fn shell_execute_error_message_uses_requested_language() {
        assert_eq!(
            shell_execute_error_message_for_language(UiLanguage::Korean, 31),
            "이 파일 형식에 연결된 실행 프로그램이 없습니다."
        );
        assert_eq!(
            shell_execute_error_message_for_language(UiLanguage::English, 31),
            "There is no app associated with this file type."
        );
        assert_eq!(
            shell_execute_error_message_for_language(UiLanguage::Korean, 123),
            "ShellExecuteW 오류 코드 123"
        );
        assert_eq!(
            shell_execute_error_message_for_language(UiLanguage::English, 123),
            "ShellExecuteW error code 123"
        );
    }

    #[test]
    fn shell_execute_failure_value_display_keeps_only_safe_state() {
        assert_eq!(
            shell_execute_failure_value_display(Some("non-empty value")),
            SHELL_EXECUTE_FIELD_REDACTED
        );
        assert_eq!(
            shell_execute_failure_value_display(Some("")),
            SHELL_EXECUTE_FIELD_EMPTY
        );
        assert_eq!(
            shell_execute_failure_value_display(None),
            SHELL_EXECUTE_FIELD_NOT_PROVIDED
        );
    }

    #[test]
    fn external_terminal_command_line_escapes_percent_in_executable_path() {
        let command_line =
            command_line_from_executable_and_arguments(r"C:\tools\%TEMP%\runner.exe", "--flag");

        assert_eq!(command_line, r"C:\tools\^%TEMP^%\runner.exe --flag");
    }

    #[test]
    fn external_terminal_command_line_quotes_and_escapes_percent_executable_path() {
        let command_line = command_line_from_executable_and_arguments(
            r"C:\Program Files\%TEMP%\runner.exe",
            "--flag",
        );
        let parameters = external_terminal_parameters_for_command(&command_line);

        assert_eq!(
            command_line,
            r#""C:\Program Files\^%TEMP^%\runner.exe" --flag"#
        );
        assert_eq!(
            parameters,
            r#"/D /V:OFF /S /K ""C:\Program Files\^%TEMP^%\runner.exe" --flag""#
        );
    }

    #[test]
    fn external_terminal_command_line_keeps_simple_command_name_unquoted() {
        assert_eq!(
            command_line_from_executable_and_arguments("git", "status"),
            "git status"
        );
    }
}
