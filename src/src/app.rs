use std::ffi::OsString;
use std::path::PathBuf;

use crate::domain::{
    APP_ICON_PNG_FILE_NAME, APP_ICON_SVG_FILE_NAME, APP_LINUX_APPLICATION_ID,
    APP_LINUX_DESKTOP_ENTRY_NAME, AppState, MainWindowSpec,
};
use crate::error::{AppError, AppResult};
use crate::infra::desktop_entry::DesktopEntryMetadata;
use crate::infra::{desktop_entry, settings};

#[cfg(target_os = "linux")]
use crate::infra::gtk4 as desktop_ui;
#[cfg(target_os = "windows")]
use crate::infra::win32 as desktop_ui;

pub fn run() -> AppResult<()> {
    run_with_args(std::env::args_os())
}

fn run_with_args(args: impl IntoIterator<Item = OsString>) -> AppResult<()> {
    match CliCommand::from_args(args)? {
        CliCommand::Run(options) => DesktopApp::from_startup_options(options).run(),
        CliCommand::Install => install_desktop_entry(),
        CliCommand::Uninstall => uninstall_desktop_entry(),
    }
}

pub struct DesktopApp {
    state: AppState,
}

impl DesktopApp {
    pub fn initial() -> Self {
        Self::from_startup_options(StartupOptions::default())
    }

    fn from_startup_options(options: StartupOptions) -> Self {
        match options.settings_file_path {
            Some(settings_file_path) => settings::set_user_settings_file_path(settings_file_path),
            None => settings::clear_user_settings_file_path(),
        }

        let loaded_settings = settings::load_user_settings();
        Self {
            state: AppState::from_settings(loaded_settings.settings, loaded_settings.warnings),
        }
    }

    fn run(self) -> AppResult<()> {
        let window = MainWindowSpec::initial(self.state);
        desktop_ui::run_main_window(window)
    }
}

#[derive(Debug, Eq, PartialEq)]
enum CliCommand {
    Run(StartupOptions),
    Install,
    Uninstall,
}

impl CliCommand {
    fn from_args(args: impl IntoIterator<Item = OsString>) -> AppResult<Self> {
        let mut args = args.into_iter();
        let _program_name = args.next();
        let Some(first) = args.next() else {
            return Ok(Self::Run(StartupOptions::default()));
        };

        if first == "--install" {
            ensure_no_extra_args(args)?;
            return Ok(Self::Install);
        }
        if first == "--uninstall" {
            ensure_no_extra_args(args)?;
            return Ok(Self::Uninstall);
        }
        if first.to_str().is_some_and(|arg| arg.starts_with("--")) {
            return Err(AppError::invalid_arguments(format!(
                "unknown argument: {}",
                first.to_string_lossy()
            )));
        }

        Ok(Self::Run(StartupOptions::from_first_path(first, args)?))
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct StartupOptions {
    settings_file_path: Option<PathBuf>,
}

impl StartupOptions {
    #[cfg(test)]
    fn from_args(args: impl IntoIterator<Item = OsString>) -> AppResult<Self> {
        let mut args = args.into_iter();
        let _program_name = args.next();

        let Some(first) = args.next() else {
            return Ok(Self::default());
        };
        Self::from_first_path(first, args)
    }

    fn from_first_path(
        first: OsString,
        mut args: impl Iterator<Item = OsString>,
    ) -> AppResult<Self> {
        let settings_file_path = match Some(first) {
            Some(path) if path.as_os_str().is_empty() => {
                return Err(AppError::invalid_arguments(
                    "settings file path argument is empty",
                ));
            }
            Some(path) => Some(PathBuf::from(path)),
            None => None,
        };

        if args.next().is_some() {
            return Err(AppError::invalid_arguments(
                "expected at most one settings file path argument",
            ));
        }

        Ok(Self { settings_file_path })
    }
}

fn ensure_no_extra_args(mut args: impl Iterator<Item = OsString>) -> AppResult<()> {
    if args.next().is_some() {
        return Err(AppError::invalid_arguments(
            "--install/--uninstall does not accept extra arguments",
        ));
    }
    Ok(())
}

fn install_desktop_entry() -> AppResult<()> {
    let summary = desktop_entry::install(desktop_entry_metadata())
        .map_err(|error| AppError::desktop_entry(error.to_string()))?;
    if summary.desktop_entry_changed || summary.icon_changed {
        println!(
            "desktop entry를 설치했습니다: {}",
            summary.desktop_entry_path.display()
        );
    } else {
        println!(
            "desktop entry가 이미 최신 상태입니다: {}",
            summary.desktop_entry_path.display()
        );
    }
    Ok(())
}

fn uninstall_desktop_entry() -> AppResult<()> {
    let summary = desktop_entry::uninstall(desktop_entry_metadata())
        .map_err(|error| AppError::desktop_entry(error.to_string()))?;
    if summary.desktop_entry_removed || summary.icon_removed {
        println!(
            "desktop entry를 제거했습니다: {}",
            summary.desktop_entry_path.display()
        );
    } else {
        println!(
            "desktop entry가 이미 제거된 상태입니다: {}",
            summary.desktop_entry_path.display()
        );
    }
    Ok(())
}

fn desktop_entry_metadata() -> DesktopEntryMetadata {
    DesktopEntryMetadata {
        application_id: APP_LINUX_APPLICATION_ID,
        display_name: APP_LINUX_DESKTOP_ENTRY_NAME,
        comment: APP_LINUX_DESKTOP_ENTRY_NAME,
        categories: "Utility;",
        icon_svg_file_name: APP_ICON_SVG_FILE_NAME,
        icon_png_file_name: APP_ICON_PNG_FILE_NAME,
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::PathBuf;

    use super::{CliCommand, DesktopApp, StartupOptions};
    use crate::infra::settings;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn startup_options_use_default_settings_path_without_argument() {
        let options =
            StartupOptions::from_args(args(&["j3devhelper"])).expect("args should be valid");

        assert_eq!(options.settings_file_path, None);
    }

    #[test]
    fn startup_options_accept_single_settings_file_path_argument() {
        let options = StartupOptions::from_args(args(&["j3devhelper", "./j3devhelper-linux.toml"]))
            .expect("args should be valid");

        assert_eq!(
            options.settings_file_path,
            Some(PathBuf::from("./j3devhelper-linux.toml"))
        );
    }

    #[test]
    fn startup_options_reject_extra_arguments() {
        let error = StartupOptions::from_args(args(&["j3devhelper", "one.toml", "two.toml"]))
            .expect_err("extra args should be rejected");

        assert_eq!(
            error.to_string(),
            "expected at most one settings file path argument"
        );
    }

    #[test]
    fn startup_options_reject_empty_settings_file_path_argument() {
        let error = StartupOptions::from_args(args(&["j3devhelper", ""]))
            .expect_err("empty settings path should be rejected");

        assert_eq!(error.to_string(), "settings file path argument is empty");
    }

    #[test]
    fn cli_command_accepts_install_without_loading_settings() {
        let command = CliCommand::from_args(args(&["j3devhelper", "--install"]))
            .expect("install args should be valid");

        assert_eq!(command, CliCommand::Install);
    }

    #[test]
    fn cli_command_accepts_uninstall_without_loading_settings() {
        let command = CliCommand::from_args(args(&["j3devhelper", "--uninstall"]))
            .expect("uninstall args should be valid");

        assert_eq!(command, CliCommand::Uninstall);
    }

    #[test]
    fn cli_command_rejects_install_with_extra_args() {
        let error = CliCommand::from_args(args(&["j3devhelper", "--install", "extra"]))
            .expect_err("extra install args should be rejected");

        assert_eq!(
            error.to_string(),
            "--install/--uninstall does not accept extra arguments"
        );
    }

    #[test]
    fn cli_command_rejects_uninstall_with_extra_args() {
        let error = CliCommand::from_args(args(&["j3devhelper", "--uninstall", "extra"]))
            .expect_err("extra uninstall args should be rejected");

        assert_eq!(
            error.to_string(),
            "--install/--uninstall does not accept extra arguments"
        );
    }

    #[test]
    fn cli_command_rejects_unknown_option() {
        let error = CliCommand::from_args(args(&["j3devhelper", "--unknown"]))
            .expect_err("unknown option should be rejected");

        assert_eq!(error.to_string(), "unknown argument: --unknown");
    }

    #[test]
    fn desktop_app_loads_settings_from_path_argument() {
        struct UserSettingsFilePathReset;

        impl Drop for UserSettingsFilePathReset {
            fn drop(&mut self) {
                settings::clear_user_settings_file_path();
            }
        }

        let _reset = UserSettingsFilePathReset;
        let settings_file_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("j3devhelper-linux.toml");

        let app = DesktopApp::from_startup_options(StartupOptions {
            settings_file_path: Some(settings_file_path),
        });

        assert_eq!(app.state.settings().workspaces.len(), 1);
        assert_eq!(app.state.settings().command_tabs[0].name, "base");
        assert_eq!(app.state.settings().command_tabs[1].name, "rust");
    }
}
