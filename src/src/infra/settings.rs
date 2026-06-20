use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::domain::AppSettings;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettingsLoadOutcome {
    pub settings: AppSettings,
    pub warnings: Vec<String>,
}

pub fn load_user_settings() -> SettingsLoadOutcome {
    let path = match user_settings_file_path() {
        Ok(path) => path,
        Err(error) => {
            return SettingsLoadOutcome {
                settings: AppSettings::default(),
                warnings: vec![format!(
                    "Settings path unavailable. Using defaults: {error}"
                )],
            };
        }
    };

    load_settings_from_path(&path)
}

pub fn save_user_settings(settings: &AppSettings) -> Result<PathBuf, SettingsSaveError> {
    save_user_settings_with_change_check(settings, None)
}

pub fn save_user_settings_if_changed(
    settings: &AppSettings,
    previous_settings: &AppSettings,
) -> Result<PathBuf, SettingsSaveError> {
    save_user_settings_with_change_check(settings, Some(previous_settings))
}

fn save_user_settings_with_change_check(
    settings: &AppSettings,
    previous_settings: Option<&AppSettings>,
) -> Result<PathBuf, SettingsSaveError> {
    let path = user_settings_file_path().map_err(SettingsSaveError::Path)?;
    save_settings_to_path_with_change_check(settings, previous_settings, &path)?;
    Ok(path)
}

static USER_SETTINGS_FILE_PATH: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

fn configured_user_settings_file_path() -> &'static Mutex<Option<PathBuf>> {
    USER_SETTINGS_FILE_PATH.get_or_init(|| Mutex::new(None))
}

pub fn set_user_settings_file_path(path: PathBuf) {
    let Ok(mut configured_path) = configured_user_settings_file_path().lock() else {
        return;
    };

    *configured_path = Some(path);
}

pub fn clear_user_settings_file_path() {
    let Ok(mut configured_path) = configured_user_settings_file_path().lock() else {
        return;
    };

    *configured_path = None;
}

pub fn user_settings_file_path() -> Result<PathBuf, SettingsPathError> {
    if let Ok(configured_path) = configured_user_settings_file_path().lock()
        && let Some(path) = configured_path.as_ref()
    {
        return Ok(path.clone());
    }

    path_policy::current_settings_file_path()
}

pub fn load_settings_from_path(path: &Path) -> SettingsLoadOutcome {
    match persistence::read_settings_file(path) {
        Ok(content) => match load_settings_from_str(&content) {
            Ok(outcome) => outcome,
            Err(error) => SettingsLoadOutcome {
                settings: AppSettings::default(),
                warnings: vec![format!(
                    "Settings file format error. Using defaults: {} ({error})",
                    path.display()
                )],
            },
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => SettingsLoadOutcome {
            settings: AppSettings::default(),
            warnings: vec![format!(
                "Settings file not found. Using defaults: {}",
                path.display()
            )],
        },
        Err(error) => SettingsLoadOutcome {
            settings: AppSettings::default(),
            warnings: vec![format!(
                "Settings file read failed. Using defaults: {} ({error})",
                path.display()
            )],
        },
    }
}

pub fn save_settings_to_path(settings: &AppSettings, path: &Path) -> Result<(), SettingsSaveError> {
    save_settings_to_path_with_change_check(settings, None, path)
}

fn save_settings_to_path_with_change_check(
    settings: &AppSettings,
    previous_settings: Option<&AppSettings>,
    path: &Path,
) -> Result<(), SettingsSaveError> {
    if let Some(previous_settings) = previous_settings
        && previous_settings == settings
    {
        return Ok(());
    }

    ensure_existing_settings_file_is_loadable(path)?;

    let content =
        stored_document::serialize_settings(settings).map_err(SettingsSaveError::Serialize)?;
    persistence::write_settings_atomically(path, content.as_bytes())?;
    remember_loadable_settings_file(path);
    Ok(())
}

fn ensure_existing_settings_file_is_loadable(path: &Path) -> Result<(), SettingsSaveError> {
    if cached_loadable_settings_file_matches(path) {
        return Ok(());
    }

    let content = match persistence::read_settings_file(path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Ok(()),
    };

    stored_document::parse_settings(&content).map_err(|source| {
        SettingsSaveError::InvalidExistingFile {
            path: path.to_path_buf(),
            source,
        }
    })?;
    remember_loadable_settings_file(path);
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LoadableSettingsFileCache {
    path: PathBuf,
    fingerprint: persistence::SettingsFileFingerprint,
}

static LOADABLE_SETTINGS_FILE_CACHE: OnceLock<Mutex<Option<LoadableSettingsFileCache>>> =
    OnceLock::new();

fn loadable_settings_file_cache() -> &'static Mutex<Option<LoadableSettingsFileCache>> {
    LOADABLE_SETTINGS_FILE_CACHE.get_or_init(|| Mutex::new(None))
}

fn cached_loadable_settings_file_matches(path: &Path) -> bool {
    let Ok(Some(fingerprint)) = persistence::settings_file_fingerprint(path) else {
        return false;
    };
    let Ok(cache) = loadable_settings_file_cache().lock() else {
        return false;
    };

    cache
        .as_ref()
        .is_some_and(|cache| cache.path == path && cache.fingerprint == fingerprint)
}

fn remember_loadable_settings_file(path: &Path) {
    let Ok(Some(fingerprint)) = persistence::settings_file_fingerprint(path) else {
        return;
    };
    let Ok(mut cache) = loadable_settings_file_cache().lock() else {
        return;
    };

    *cache = Some(LoadableSettingsFileCache {
        path: path.to_path_buf(),
        fingerprint,
    });
}

pub fn settings_path_for_executable(executable_path: &Path) -> Result<PathBuf, SettingsPathError> {
    path_policy::settings_path_for_executable(executable_path)
}

fn load_settings_from_str(content: &str) -> Result<SettingsLoadOutcome, toml::de::Error> {
    let document = stored_document::parse_settings(content)?;
    let (settings, warnings) = restore_policy::restore_settings(document);
    Ok(SettingsLoadOutcome { settings, warnings })
}

#[derive(Debug)]
pub enum SettingsPathError {
    CurrentExe(io::Error),
    MissingParent(PathBuf),
    MissingFileStem(PathBuf),
}

impl Display for SettingsPathError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentExe(error) => {
                write!(formatter, "failed to resolve executable path ({error})")
            }
            Self::MissingParent(path) => {
                write!(
                    formatter,
                    "could not resolve the executable folder: {}",
                    path.display()
                )
            }
            Self::MissingFileStem(path) => {
                write!(
                    formatter,
                    "could not resolve the executable name: {}",
                    path.display()
                )
            }
        }
    }
}

impl Error for SettingsPathError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CurrentExe(error) => Some(error),
            Self::MissingParent(_) | Self::MissingFileStem(_) => None,
        }
    }
}

#[derive(Debug)]
pub enum SettingsSaveError {
    Path(SettingsPathError),
    Serialize(toml::ser::Error),
    InvalidExistingFile {
        path: PathBuf,
        source: toml::de::Error,
    },
    Write {
        path: PathBuf,
        source: io::Error,
    },
    Cleanup {
        path: PathBuf,
        source: io::Error,
        cause: Box<SettingsSaveError>,
    },
}

impl Display for SettingsSaveError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(error) => write!(formatter, "settings path failed: {error}"),
            Self::Serialize(error) => write!(formatter, "settings file generation failed: {error}"),
            Self::InvalidExistingFile { path, source } => write!(
                formatter,
                "existing settings file has format errors and was not overwritten: {} ({source})",
                path.display()
            ),
            Self::Write { path, source } => {
                write!(
                    formatter,
                    "settings save failed: {} ({source})",
                    path.display()
                )
            }
            Self::Cleanup {
                path,
                source,
                cause,
            } => write!(
                formatter,
                "could not clean up a temporary file after settings save: {} ({source}); original error: {cause}",
                path.display()
            ),
        }
    }
}

impl Error for SettingsSaveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Path(error) => Some(error),
            Self::Serialize(error) => Some(error),
            Self::InvalidExistingFile { source, .. } => Some(source),
            Self::Write { source, .. } => Some(source),
            Self::Cleanup { source, .. } => Some(source),
        }
    }
}

mod path_policy;
mod persistence;
mod restore_policy;
mod stored_document;

#[cfg(test)]
mod tests {
    use std::fs;
    #[cfg(windows)]
    use std::os::windows::fs::OpenOptionsExt;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::domain::{
        AppState, Category, CommandButton, CommandTab, ExecutionType, INITIAL_STATUS_MESSAGE,
        TreeRootItem, ViewSettings, Workspace, default_workspace_language_options,
    };

    #[test]
    fn settings_path_uses_executable_folder_and_file_stem() {
        #[cfg(windows)]
        let path = settings_path_for_executable(Path::new(r"C:\Tools\j3DevHelper.exe"))
            .expect("settings path should be derived from executable path");
        #[cfg(not(windows))]
        let path = settings_path_for_executable(Path::new("/opt/j3DevHelper"))
            .expect("settings path should be derived from executable path");

        #[cfg(windows)]
        assert_eq!(path, PathBuf::from(r"C:\Tools\j3DevHelper.toml"));
        #[cfg(not(windows))]
        assert_eq!(path, PathBuf::from("/opt/j3DevHelper.toml"));
    }

    #[test]
    fn user_settings_file_path_prefers_configured_path() {
        struct UserSettingsFilePathReset;

        impl Drop for UserSettingsFilePathReset {
            fn drop(&mut self) {
                clear_user_settings_file_path();
            }
        }

        let _reset = UserSettingsFilePathReset;
        clear_user_settings_file_path();
        let path = PathBuf::from("./custom-settings.toml");

        set_user_settings_file_path(path.clone());

        assert_eq!(
            user_settings_file_path().expect("configured path should be available"),
            path
        );
    }

    #[test]
    fn save_user_settings_uses_configured_path() {
        struct UserSettingsFilePathReset;

        impl Drop for UserSettingsFilePathReset {
            fn drop(&mut self) {
                clear_user_settings_file_path();
            }
        }

        let _reset = UserSettingsFilePathReset;
        let dir = unique_temp_dir("configured-user-settings-path");
        fs::create_dir_all(&dir).expect("temp directory should be created");
        let path = dir.join("custom-settings.toml");
        set_user_settings_file_path(path.clone());

        let saved_path =
            save_user_settings(&AppSettings::default()).expect("settings should be saved");

        assert_eq!(saved_path, path);
        assert!(saved_path.exists());

        fs::remove_dir_all(&dir).expect("temp directory should be removed");
    }

    #[test]
    fn load_valid_settings_preserves_config_field_names() {
        let loaded = load_settings_from_str(
            r#"
[view]
font_family = "Consolas"
font_size = 14
theme = "dark"

[[categories]]
name = "Backend"

[[workspaces]]
path = 'C:\dev\j3DevHelper'
name = "j3DevHelper"
Language = "Rust"
category = "Backend"

[[command_tabs]]
name = "common"

[[command_tabs.buttons]]
button_name = "Cargo Check"
executable_path = "cargo"
arguments = "check"
execution_type = "external_terminal"
"#,
        )
        .expect("valid TOML should load");

        assert!(loaded.warnings.is_empty());
        assert_eq!(loaded.settings.view.font_family, "Consolas");
        assert_eq!(loaded.settings.view.ui_language, "en");
        assert_eq!(
            loaded.settings.languages,
            default_workspace_language_options()
        );
        assert_eq!(loaded.settings.categories[0].name, "Backend");
        assert_eq!(loaded.settings.workspaces[0].language, "Rust");
        assert_eq!(
            loaded.settings.workspaces[0].category.as_deref(),
            Some("Backend")
        );
        assert_eq!(
            loaded.settings.command_tabs[0].buttons[0].execution_type,
            ExecutionType::ExternalTerminal
        );
    }

    #[test]
    fn linux_sample_settings_loads_without_warnings() {
        let loaded = load_settings_from_str(include_str!("../../j3devhelper-linux.toml"))
            .expect("linux sample settings should be valid TOML");

        assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
        assert_eq!(loaded.settings.workspaces.len(), 1);
        assert_eq!(loaded.settings.workspaces[0].language, "Rust");
        assert_eq!(loaded.settings.command_tabs.len(), 2);
        assert_eq!(loaded.settings.command_tabs[0].name, "base");
        assert_eq!(loaded.settings.command_tabs[1].name, "rust");
    }

    #[test]
    fn save_and_load_round_trips_domain_settings() {
        let settings = AppSettings {
            view: ViewSettings::new("Consolas", 14, "dark"),
            languages: default_workspace_language_options(),
            tree_order: Vec::new(),
            categories: vec![Category::new("Backend").expect("category should be valid")],
            workspaces: vec![
                Workspace::new(r"C:\dev\j3DevHelper", "j3DevHelper", "Rust")
                    .expect("workspace should be valid"),
            ],
            command_tabs: vec![
                CommandTab::new(
                    "common",
                    vec![
                        CommandButton::new(
                            "Open Workspace",
                            r"C:\Windows\explorer.exe",
                            "{path}",
                            ExecutionType::ShellApi,
                        )
                        .expect("shell_api command button should be valid"),
                        CommandButton::new(
                            "Cargo Check",
                            "cargo",
                            "check",
                            ExecutionType::ExternalTerminal,
                        )
                        .expect("external_terminal command button should be valid"),
                    ],
                )
                .expect("command tab should be valid"),
            ],
        };
        let dir = unique_temp_dir("round-trip-settings");
        fs::create_dir_all(&dir).expect("temp directory should be created");
        let path = dir.join("j3DevHelper.toml");

        save_settings_to_path(&settings, &path).expect("settings should be saved");
        let loaded = load_settings_from_path(&path);

        assert!(loaded.warnings.is_empty());
        assert_eq!(loaded.settings, settings);

        fs::remove_dir_all(&dir).expect("temp directory should be removed");
    }

    #[test]
    fn save_and_load_round_trips_window_layout_settings() {
        let settings = AppSettings {
            view: ViewSettings::new("Segoe UI", 12, "system")
                .with_ui_language("en")
                .with_window_layout(Some(800), Some(600), Some(240)),
            ..AppSettings::default()
        };
        let dir = unique_temp_dir("round-trip-window-layout");
        fs::create_dir_all(&dir).expect("temp directory should be created");
        let path = dir.join("j3DevHelper.toml");

        save_settings_to_path(&settings, &path).expect("settings should be saved");
        let content = fs::read_to_string(&path).expect("saved settings should be readable");
        let loaded = load_settings_from_path(&path);

        assert!(content.contains("window_width = 800"));
        assert!(content.contains("ui_language = \"en\""));
        assert!(content.contains("window_height = 600"));
        assert!(content.contains("tree_panel_width = 240"));
        assert!(loaded.warnings.is_empty());
        assert_eq!(loaded.settings.view.window_width, Some(800));
        assert_eq!(loaded.settings.view.ui_language, "en");
        assert_eq!(loaded.settings.view.window_height, Some(600));
        assert_eq!(loaded.settings.view.tree_panel_width, Some(240));

        fs::remove_dir_all(&dir).expect("temp directory should be removed");
    }

    #[test]
    fn load_ignores_incomplete_window_size_settings() {
        let loaded = load_settings_from_str(
            r#"
[view]
window_width = 800
tree_panel_width = -1
"#,
        )
        .expect("TOML syntax should be valid");

        assert_eq!(loaded.settings.view.window_width, None);
        assert_eq!(loaded.settings.view.window_height, None);
        assert_eq!(loaded.settings.view.tree_panel_width, None);
        assert!(
            loaded
                .warnings
                .iter()
                .any(|warning| { warning.contains("view.window_width/view.window_height") })
        );
        assert!(
            loaded
                .warnings
                .iter()
                .any(|warning| warning.contains("view.tree_panel_width"))
        );
    }

    #[test]
    fn load_accepts_both_supported_execution_type_values() {
        let loaded = load_settings_from_str(
            r#"
[[command_tabs]]
name = "common"

[[command_tabs.buttons]]
button_name = "Open Workspace"
executable_path = 'C:\Windows\explorer.exe'
arguments = "{path}"
execution_type = "shell_api"

[[command_tabs.buttons]]
button_name = "Cargo Check"
executable_path = "cargo"
arguments = "check"
execution_type = "external_terminal"
"#,
        )
        .expect("valid TOML should load");

        let buttons = &loaded.settings.command_tabs[0].buttons;
        assert!(loaded.warnings.is_empty());
        assert_eq!(buttons[0].execution_type, ExecutionType::ShellApi);
        assert_eq!(buttons[1].execution_type, ExecutionType::ExternalTerminal);
    }

    #[test]
    fn load_and_save_preserves_user_language_config() {
        let loaded = load_settings_from_str(
            r#"
languages = ["rust", "c", "java"]

[[workspaces]]
path = 'C:\dev\j3DevHelper'
name = "j3DevHelper"
Language = "Rust"
"#,
        )
        .expect("valid TOML should load");

        assert!(loaded.warnings.is_empty());
        assert_eq!(loaded.settings.languages, vec!["rust", "c", "java"]);
        assert_eq!(loaded.settings.workspaces[0].language, "rust");
    }

    #[test]
    fn load_skips_invalid_workspaces_and_command_buttons() {
        let loaded = load_settings_from_str(
            r#"
[[workspaces]]
path = 'C:\dev\missing-language'
name = "missing language"

[[command_tabs]]
name = "common"

[[command_tabs.buttons]]
button_name = "Missing Executable"
arguments = "check"
execution_type = "shell_api"

[[command_tabs.buttons]]
button_name = "Invalid Execution Type"
executable_path = "cargo"
arguments = "check"
execution_type = "inline"
"#,
        )
        .expect("TOML syntax should be valid");

        assert!(loaded.settings.workspaces.is_empty());
        assert!(loaded.settings.command_tabs[0].buttons.is_empty());
        assert_eq!(loaded.warnings.len(), 3);
        assert!(
            loaded
                .warnings
                .iter()
                .any(|warning| warning.contains("Language"))
        );
        assert!(
            loaded
                .warnings
                .iter()
                .any(|warning| warning.contains("executable_path"))
        );
        assert!(
            loaded
                .warnings
                .iter()
                .any(|warning| warning.contains("inline"))
        );
    }

    #[test]
    fn load_skips_items_with_non_string_fields_without_dropping_other_settings() {
        let loaded = load_settings_from_str(
            r#"
[[workspaces]]
path = 'C:\dev\valid'
name = "valid workspace"
Language = "Rust"

[[workspaces]]
path = 123
name = "bad path"
Language = "Rust"

[[workspaces]]
path = 'C:\dev\bad-language'
name = "bad language"
Language = ["Rust"]

[[command_tabs]]
name = "common"

[[command_tabs.buttons]]
button_name = "Cargo Check"
executable_path = "cargo"
arguments = "check"
execution_type = "external_terminal"

[[command_tabs.buttons]]
button_name = "Bad Arguments"
executable_path = "cargo"
arguments = ["check"]
execution_type = "shell_api"

[[command_tabs]]
name = 7

[[command_tabs.buttons]]
button_name = "Skipped With Tab"
executable_path = "cargo"
arguments = "check"
execution_type = "shell_api"
"#,
        )
        .expect("recoverable string field type errors should not reject document");

        assert_eq!(loaded.settings.workspaces.len(), 1);
        assert_eq!(loaded.settings.workspaces[0].name, "valid workspace");
        assert_eq!(loaded.settings.command_tabs.len(), 1);
        assert_eq!(loaded.settings.command_tabs[0].name, "common");
        assert_eq!(loaded.settings.command_tabs[0].buttons.len(), 1);
        assert_eq!(
            loaded.settings.command_tabs[0].buttons[0].execution_type,
            ExecutionType::ExternalTerminal
        );
        assert_eq!(loaded.warnings.len(), 4);
        assert!(
            loaded
                .warnings
                .iter()
                .any(|warning| warning.contains("workspaces[1]") && warning.contains("path"))
        );
        assert!(
            loaded
                .warnings
                .iter()
                .any(|warning| warning.contains("workspaces[2]") && warning.contains("Language"))
        );
        assert!(loaded.warnings.iter().any(|warning| {
            warning.contains("command_tabs[0].buttons[1]") && warning.contains("arguments")
        }));
        assert!(
            loaded
                .warnings
                .iter()
                .any(|warning| warning.contains("command_tabs[1]") && warning.contains("name"))
        );
    }

    #[test]
    fn load_uses_default_for_unsupported_font_size_selection() {
        let loaded = load_settings_from_str(
            r#"
[view]
font_family = "Segoe UI"
font_size = 15
theme = "system"
"#,
        )
        .expect("TOML syntax should be valid");

        assert_eq!(
            loaded.settings.view.font_size,
            ViewSettings::default().font_size
        );
        assert_eq!(loaded.warnings.len(), 1);
        assert!(loaded.warnings[0].contains("view.font_size"));
        assert!(loaded.warnings[0].contains("15"));
    }

    #[test]
    fn load_uses_default_for_invalid_font_size_without_dropping_other_settings() {
        for (font_size, expected_warning_value) in [("70000", "70000"), ("\"large\"", "large")] {
            let content = format!(
                r#"
[view]
font_family = "Segoe UI"
font_size = {font_size}
theme = "dark"

[[workspaces]]
path = 'C:\dev\j3DevHelper'
name = "j3DevHelper"
Language = "Rust"

[[command_tabs]]
name = "common"
"#
            );

            let loaded = load_settings_from_str(&content)
                .expect("recoverable font_size errors should not reject document");

            assert_eq!(
                loaded.settings.view.font_size,
                ViewSettings::default().font_size
            );
            assert_eq!(loaded.settings.workspaces.len(), 1);
            assert_eq!(loaded.settings.workspaces[0].name, "j3DevHelper");
            assert_eq!(loaded.settings.command_tabs.len(), 1);
            assert_eq!(loaded.settings.command_tabs[0].name, "common");
            assert_eq!(loaded.warnings.len(), 1);
            assert!(loaded.warnings[0].contains("view.font_size"));
            assert!(loaded.warnings[0].contains(expected_warning_value));
        }
    }

    #[test]
    fn load_uses_default_for_invalid_ui_language() {
        let loaded = load_settings_from_str(
            r#"
[view]
ui_language = "jp"
"#,
        )
        .expect("TOML syntax should be valid");

        assert_eq!(loaded.settings.view.ui_language, "en");
        assert_eq!(loaded.warnings.len(), 1);
        assert!(loaded.warnings[0].contains("view.ui_language"));
        assert!(loaded.warnings[0].contains("jp"));
    }

    #[test]
    fn load_skips_workspaces_with_unsupported_language_values() {
        let loaded = load_settings_from_str(
            r#"
[[workspaces]]
path = 'C:\dev\unsupported-language'
name = "unsupported"
Language = "Haskell"
"#,
        )
        .expect("TOML syntax should be valid");

        assert!(loaded.settings.workspaces.is_empty());
        assert_eq!(loaded.warnings.len(), 1);
        assert!(loaded.warnings[0].contains("목록에 없는 언어"));
        assert!(loaded.warnings[0].contains("Haskell"));
    }

    #[test]
    fn load_skips_items_with_empty_required_fields_and_sets_status_message() {
        let loaded = load_settings_from_str(
            r#"
[[workspaces]]
path = 'C:\dev\empty-name'
name = " "
Language = "Rust"

[[command_tabs]]
name = " "

[[command_tabs]]
name = "valid"

[[command_tabs.buttons]]
button_name = " "
executable_path = "cargo"
execution_type = "shell_api"

[[command_tabs.buttons]]
button_name = "Missing Type"
executable_path = "cargo"
"#,
        )
        .expect("TOML syntax should be valid");
        let state = AppState::from_settings(loaded.settings, loaded.warnings);

        assert!(state.settings().workspaces.is_empty());
        assert_eq!(state.settings().command_tabs.len(), 1);
        assert!(state.settings().command_tabs[0].buttons.is_empty());
        assert!(state.status_message().contains("4 settings warnings"));
        assert!(state.status_message().contains("workspaces[0]"));
    }

    #[test]
    fn lowercase_language_key_is_not_treated_as_language_field() {
        let loaded = load_settings_from_str(
            r#"
[[workspaces]]
path = 'C:\dev\j3DevHelper'
name = "j3DevHelper"
language = "Rust"
"#,
        )
        .expect("TOML syntax should be valid");

        assert!(loaded.settings.workspaces.is_empty());
        assert_eq!(loaded.warnings.len(), 1);
        assert!(loaded.warnings[0].contains("Language"));
    }

    #[test]
    fn load_skips_duplicate_workspace_paths() {
        let loaded = load_settings_from_str(
            r#"
[[workspaces]]
path = 'C:\dev\j3DevHelper'
name = "first"
Language = "Rust"

[[workspaces]]
path = 'C:\DEV\j3DevHelper\'
name = "second"
Language = "Rust"
"#,
        )
        .expect("TOML syntax should be valid");

        assert_eq!(loaded.settings.workspaces.len(), 1);
        assert_eq!(loaded.settings.workspaces[0].name, "first");
        assert_eq!(loaded.warnings.len(), 1);
        assert!(
            loaded
                .warnings
                .iter()
                .any(|warning| warning.contains("이미 등록된 폴더"))
        );
    }

    #[test]
    fn load_skips_invalid_and_duplicate_categories() {
        let loaded = load_settings_from_str(
            r#"
[[categories]]
name = "Tools"

[[categories]]
name = "tools"

[[categories]]
name = 7
"#,
        )
        .expect("TOML syntax should be valid");

        assert_eq!(loaded.settings.categories.len(), 1);
        assert_eq!(loaded.settings.categories[0].name, "Tools");
        assert_eq!(loaded.warnings.len(), 2);
        assert!(
            loaded
                .warnings
                .iter()
                .any(|warning| warning.contains("categories[1]")
                    && warning.contains("이미 있는 분류"))
        );
        assert!(
            loaded
                .warnings
                .iter()
                .any(|warning| warning.contains("categories[2]") && warning.contains("name"))
        );
    }

    #[test]
    fn load_restores_workspace_category_only_when_category_exists() {
        let loaded = load_settings_from_str(
            r#"
[[categories]]
name = "Backend"

[[workspaces]]
path = 'C:\dev\api'
name = "api"
Language = "Rust"
category = "Backend"

[[workspaces]]
path = 'C:\dev\orphan'
name = "orphan"
Language = "Rust"
category = "Missing"
"#,
        )
        .expect("TOML syntax should be valid");

        assert_eq!(
            loaded.settings.workspaces[0].category.as_deref(),
            Some("Backend")
        );
        assert_eq!(loaded.settings.workspaces[1].category, None);
        assert!(
            loaded
                .warnings
                .iter()
                .any(|warning| warning.contains("없는 분류"))
        );
    }

    #[test]
    fn load_preserves_mixed_tree_order_when_present() {
        let loaded = load_settings_from_str(
            r#"
[[tree_order]]
type = "category"
name = "Backend"

[[tree_order]]
type = "workspace"
path = 'C:\dev\cli'

[[tree_order]]
type = "workspace"
path = 'C:\dev\tools'

[[tree_order]]
type = "category"
name = "Frontend"

[[categories]]
name = "Backend"

[[categories]]
name = "Frontend"

[[workspaces]]
path = 'C:\dev\cli'
name = "cli"
Language = "Rust"

[[workspaces]]
path = 'C:\dev\tools'
name = "tools"
Language = "Rust"
"#,
        )
        .expect("valid TOML should load");

        assert!(loaded.warnings.is_empty());
        assert_eq!(
            loaded.settings.tree_order,
            vec![
                TreeRootItem::category("Backend"),
                TreeRootItem::workspace(r"C:\dev\cli"),
                TreeRootItem::workspace(r"C:\dev\tools"),
                TreeRootItem::category("Frontend"),
            ]
        );
    }

    #[test]
    fn save_writes_language_key_and_execution_type_values() {
        let categorized = Workspace::new(r"C:\dev\j3DevHelper", "j3DevHelper", "Rust")
            .expect("workspace should be valid")
            .with_category(Some("Backend".to_owned()));
        let settings = AppSettings {
            view: ViewSettings::default(),
            languages: default_workspace_language_options(),
            tree_order: Vec::new(),
            categories: vec![Category::new("Backend").expect("category should be valid")],
            workspaces: vec![categorized],
            command_tabs: vec![
                CommandTab::new(
                    "common",
                    vec![
                        CommandButton::new(
                            "Open Workspace",
                            r"C:\Windows\explorer.exe",
                            "{path}",
                            ExecutionType::ShellApi,
                        )
                        .expect("command button should be valid"),
                    ],
                )
                .expect("command tab should be valid"),
            ],
        };
        let dir = unique_temp_dir("save-settings");
        fs::create_dir_all(&dir).expect("temp directory should be created");
        let path = dir.join("j3DevHelper.toml");

        save_settings_to_path(&settings, &path).expect("settings should be saved");
        let content = fs::read_to_string(&path).expect("saved settings should be readable");

        assert!(content.contains("\nLanguage = \"Rust\""));
        assert!(content.contains("languages = ["));
        assert!(!content.contains("\nlanguage = "));
        assert!(content.contains("category = \"Backend\""));
        assert!(content.contains("execution_type = \"shell_api\""));

        fs::remove_dir_all(&dir).expect("temp directory should be removed");
    }

    #[test]
    fn save_writes_mixed_tree_order_when_present() {
        let settings = AppSettings {
            view: ViewSettings::default(),
            languages: default_workspace_language_options(),
            tree_order: vec![
                TreeRootItem::category("Backend"),
                TreeRootItem::workspace(r"C:\dev\cli"),
                TreeRootItem::category("Frontend"),
            ],
            categories: vec![
                Category::new("Backend").expect("category should be valid"),
                Category::new("Frontend").expect("category should be valid"),
            ],
            workspaces: vec![
                Workspace::new(r"C:\dev\cli", "cli", "Rust").expect("workspace should be valid"),
            ],
            command_tabs: Vec::new(),
        };
        let dir = unique_temp_dir("save-tree-order-settings");
        fs::create_dir_all(&dir).expect("temp directory should be created");
        let path = dir.join("j3DevHelper.toml");

        save_settings_to_path(&settings, &path).expect("settings should be saved");
        let content = fs::read_to_string(&path).expect("saved settings should be readable");
        let loaded = load_settings_from_path(&path);

        assert!(content.contains("[[tree_order]]"));
        assert!(content.contains("type = \"category\""));
        assert!(content.contains("type = \"workspace\""));
        assert_eq!(loaded.settings.tree_order, settings.tree_order);

        fs::remove_dir_all(&dir).expect("temp directory should be removed");
    }

    #[cfg(windows)]
    #[test]
    fn save_replace_failure_preserves_existing_file_and_cleans_temp_file() {
        let dir = unique_temp_dir("replace-failure-settings");
        fs::create_dir_all(&dir).expect("temp directory should be created");
        let path = dir.join("j3DevHelper.toml");
        let original_content = "existing settings content";
        fs::write(&path, original_content).expect("existing settings should be written");

        let locked_file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .open(&path)
            .expect("settings file should be locked exclusively");

        let result = save_settings_to_path(&AppSettings::default(), &path);

        drop(locked_file);

        assert!(result.is_err());
        let preserved_content =
            fs::read_to_string(&path).expect("existing settings should remain readable");
        assert_eq!(preserved_content, original_content);
        assert!(
            temporary_settings_files(&dir, "j3DevHelper.toml.tmp.")
                .expect("temp directory should be readable")
                .is_empty()
        );

        fs::remove_dir_all(&dir).expect("temp directory should be removed");
    }

    #[test]
    fn save_refuses_to_overwrite_unloadable_existing_settings() {
        let dir = unique_temp_dir("preserve-unloadable-settings");
        fs::create_dir_all(&dir).expect("temp directory should be created");
        let path = dir.join("j3DevHelper.toml");
        let original_content = "view = \"not a table\"";
        fs::write(&path, original_content).expect("unloadable settings should be written");

        let result = save_settings_to_path(&AppSettings::default(), &path);

        match result {
            Err(SettingsSaveError::InvalidExistingFile {
                path: error_path, ..
            }) => assert_eq!(error_path, path),
            other => panic!("expected invalid existing file error, got {other:?}"),
        }
        let preserved_content =
            fs::read_to_string(&path).expect("existing settings should remain readable");
        assert_eq!(preserved_content, original_content);
        assert!(
            temporary_settings_files(&dir, "j3DevHelper.toml.tmp.")
                .expect("temp directory should be readable")
                .is_empty()
        );

        fs::remove_dir_all(&dir).expect("temp directory should be removed");
    }

    #[test]
    fn save_revalidates_existing_settings_after_file_changes() {
        let dir = unique_temp_dir("revalidate-changed-settings");
        fs::create_dir_all(&dir).expect("temp directory should be created");
        let path = dir.join("j3DevHelper.toml");
        let previous_settings = AppSettings::default();
        let changed_settings = AppSettings {
            view: ViewSettings::new("Consolas", 14, "dark"),
            ..AppSettings::default()
        };
        let invalid_content = "view = \"not a table\"\n# changed after a successful cached save\n";

        save_settings_to_path(&previous_settings, &path).expect("settings should be saved");
        fs::write(&path, invalid_content).expect("invalid settings should be written");

        let result = save_settings_to_path_with_change_check(
            &changed_settings,
            Some(&previous_settings),
            &path,
        );

        match result {
            Err(SettingsSaveError::InvalidExistingFile {
                path: error_path, ..
            }) => assert_eq!(error_path, path),
            other => panic!("expected invalid existing file error, got {other:?}"),
        }
        let preserved_content =
            fs::read_to_string(&path).expect("existing settings should remain readable");
        assert_eq!(preserved_content, invalid_content);

        fs::remove_dir_all(&dir).expect("temp directory should be removed");
    }

    #[test]
    fn missing_settings_file_starts_with_defaults_and_status_warning() {
        let dir = unique_temp_dir("missing-settings");
        fs::create_dir_all(&dir).expect("temp directory should be created");
        let path = dir.join("j3DevHelper.toml");

        let loaded = load_settings_from_path(&path);
        let state = AppState::from_settings(loaded.settings, loaded.warnings);

        assert_eq!(state.settings(), &AppSettings::default());
        assert_ne!(state.status_message(), INITIAL_STATUS_MESSAGE);
        assert!(state.status_message().contains("Settings file not found"));

        fs::remove_dir_all(&dir).expect("temp directory should be removed");
    }

    #[test]
    fn unreadable_settings_file_starts_with_defaults_and_status_warning() {
        let dir = unique_temp_dir("unreadable-settings");
        fs::create_dir_all(&dir).expect("temp directory should be created");

        let loaded = load_settings_from_path(&dir);
        let state = AppState::from_settings(loaded.settings, loaded.warnings);

        assert_eq!(state.settings(), &AppSettings::default());
        assert_ne!(state.status_message(), INITIAL_STATUS_MESSAGE);
        assert!(state.status_message().contains("Settings file read failed"));

        fs::remove_dir_all(&dir).expect("temp directory should be removed");
    }

    #[test]
    fn invalid_toml_starts_with_defaults_and_status_warning() {
        let dir = unique_temp_dir("invalid-toml-settings");
        fs::create_dir_all(&dir).expect("temp directory should be created");
        let path = dir.join("j3DevHelper.toml");
        fs::write(&path, "[view\nfont_family = \"Consolas\"")
            .expect("invalid settings file should be written");

        let loaded = load_settings_from_path(&path);
        let state = AppState::from_settings(loaded.settings, loaded.warnings);

        assert_eq!(state.settings(), &AppSettings::default());
        assert_ne!(state.status_message(), INITIAL_STATUS_MESSAGE);
        assert!(
            state
                .status_message()
                .contains("Settings file format error")
        );

        fs::remove_dir_all(&dir).expect("temp directory should be removed");
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "j3devhelper-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    fn temporary_settings_files(dir: &Path, prefix: &str) -> io::Result<Vec<PathBuf>> {
        fs::read_dir(dir)?
            .filter_map(|entry| match entry {
                Ok(entry) => {
                    let is_temp_file = entry
                        .file_name()
                        .to_str()
                        .is_some_and(|file_name| file_name.starts_with(prefix));
                    if is_temp_file {
                        Some(Ok(entry.path()))
                    } else {
                        None
                    }
                }
                Err(error) => Some(Err(error)),
            })
            .collect()
    }
}
