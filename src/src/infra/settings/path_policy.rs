use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::SettingsPathError;

pub(super) fn current_settings_file_path() -> Result<PathBuf, SettingsPathError> {
    let executable_path = std::env::current_exe().map_err(SettingsPathError::CurrentExe)?;
    settings_path_for_executable(&executable_path)
}

pub(super) fn settings_path_for_executable(
    executable_path: &Path,
) -> Result<PathBuf, SettingsPathError> {
    let parent = executable_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or_else(|| SettingsPathError::MissingParent(executable_path.to_path_buf()))?;
    let file_stem = executable_path
        .file_stem()
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| SettingsPathError::MissingFileStem(executable_path.to_path_buf()))?;

    let mut file_name = OsString::from(file_stem);
    file_name.push(".toml");
    Ok(parent.join(file_name))
}
