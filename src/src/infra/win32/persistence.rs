use std::path::PathBuf;

use windows_sys::Win32::Foundation::HWND;

use crate::domain::{AppSettings, AppState, UiLanguage, current_ui_language};
use crate::infra::settings;

use super::{show_error_message, tr};

pub(super) fn persist_settings_or_restore<R>(
    hwnd: HWND,
    state: &mut AppState,
    restore_point: R,
) -> bool
where
    R: Into<SettingsRestorePoint>,
{
    let restore_point = restore_point.into();
    match persist_settings(hwnd, state, restore_point.settings()) {
        Ok(_) => true,
        Err(error) => {
            restore_state_after_failed_persist(state, restore_point, &error);
            false
        }
    }
}

pub(super) struct SettingsRestorePoint {
    settings: AppSettings,
    selected_workspace_index: Option<usize>,
    selected_command_tab_index: Option<usize>,
    selected_command_button_index: Option<usize>,
}

impl SettingsRestorePoint {
    pub(super) fn capture(state: &AppState) -> Self {
        Self {
            settings: state.settings().clone(),
            selected_workspace_index: state.selected_workspace_index(),
            selected_command_tab_index: state.selected_command_tab_index(),
            selected_command_button_index: state.selected_command_button_index(),
        }
    }

    fn settings(&self) -> &AppSettings {
        &self.settings
    }
}

impl From<AppState> for SettingsRestorePoint {
    fn from(state: AppState) -> Self {
        Self::capture(&state)
    }
}

pub(super) fn restore_state_after_failed_persist(
    state: &mut AppState,
    restore_point: SettingsRestorePoint,
    error: &settings::SettingsSaveError,
) {
    let language = current_ui_language(&restore_point.settings.view);
    let message = settings_save_failure_message(language, error);
    *state = AppState::from_settings(restore_point.settings, Vec::new());
    state.select_workspace(restore_point.selected_workspace_index);
    state.select_command_tab(restore_point.selected_command_tab_index);
    state.select_command_button(restore_point.selected_command_button_index);
    state.set_status_message(message);
}

fn settings_save_failure_message(
    language: UiLanguage,
    error: &settings::SettingsSaveError,
) -> String {
    match language {
        UiLanguage::Korean => format!("설정 저장 실패\n\n{error}"),
        UiLanguage::English => format!("Settings save failed\n\n{error}"),
    }
}

fn persist_settings(
    hwnd: HWND,
    state: &mut AppState,
    previous_settings: &AppSettings,
) -> Result<PathBuf, settings::SettingsSaveError> {
    let result = settings::save_user_settings_if_changed(state.settings(), previous_settings);
    let language = current_ui_language(&state.settings().view);
    match &result {
        Ok(path) => state.set_status_message(match language {
            UiLanguage::Korean => format!("설정 저장: {}", path.display()),
            UiLanguage::English => format!("Settings saved: {}", path.display()),
        }),
        Err(error) => {
            let message = settings_save_failure_message(language, error);
            state.set_status_message(message.clone());
            show_error_message(hwnd, tr(language, "설정", "Settings"), &message);
        }
    }
    result
}
