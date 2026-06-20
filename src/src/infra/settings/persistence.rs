use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{
    MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
};

use super::SettingsSaveError;

const TEMP_FILE_ATTEMPTS: u32 = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SettingsFileFingerprint {
    len: u64,
    modified: SystemTime,
}

trait SettingsFileWriter {
    fn write_all_settings(&mut self, content: &[u8]) -> io::Result<()>;
    fn flush_settings(&mut self) -> io::Result<()>;
    fn sync_all_settings(&mut self) -> io::Result<()>;
}

impl SettingsFileWriter for File {
    fn write_all_settings(&mut self, content: &[u8]) -> io::Result<()> {
        self.write_all(content)
    }

    fn flush_settings(&mut self) -> io::Result<()> {
        self.flush()
    }

    fn sync_all_settings(&mut self) -> io::Result<()> {
        self.sync_all()
    }
}

pub(super) fn read_settings_file(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}

pub(super) fn settings_file_fingerprint(
    path: &Path,
) -> io::Result<Option<SettingsFileFingerprint>> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() {
        return Ok(None);
    }

    let modified = match metadata.modified() {
        Ok(modified) => modified,
        Err(_) => return Ok(None),
    };

    Ok(Some(SettingsFileFingerprint {
        len: metadata.len(),
        modified,
    }))
}

pub(super) fn write_settings_atomically(
    path: &Path,
    content: &[u8],
) -> Result<(), SettingsSaveError> {
    let (temp_path, mut temp_file) = create_temporary_settings_file(path)?;

    let write_result = write_settings(&mut temp_file, &temp_path, content);
    drop(temp_file);

    match write_result {
        Ok(()) => match replace_settings_file(&temp_path, path) {
            Ok(()) => Ok(()),
            Err(source) => cleanup_temporary_settings_file(
                &temp_path,
                SettingsSaveError::Write {
                    path: path.to_path_buf(),
                    source,
                },
            ),
        },
        Err(error) => cleanup_temporary_settings_file(&temp_path, error),
    }
}

fn create_temporary_settings_file(path: &Path) -> Result<(PathBuf, File), SettingsSaveError> {
    for attempt in 0..TEMP_FILE_ATTEMPTS {
        let temp_path = temporary_settings_file_path(path, attempt)?;
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((temp_path, file)),
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
            Err(source) => {
                return Err(SettingsSaveError::Write {
                    path: temp_path,
                    source,
                });
            }
        }
    }

    Err(SettingsSaveError::Write {
        path: path.to_path_buf(),
        source: io::Error::new(
            io::ErrorKind::AlreadyExists,
            "unique temporary settings file could not be created",
        ),
    })
}

fn temporary_settings_file_path(path: &Path, attempt: u32) -> Result<PathBuf, SettingsSaveError> {
    let file_name = path
        .file_name()
        .filter(|file_name| !file_name.is_empty())
        .ok_or_else(|| SettingsSaveError::Write {
            path: path.to_path_buf(),
            source: io::Error::new(
                io::ErrorKind::InvalidInput,
                "settings file path does not include a file name",
            ),
        })?;

    let mut temp_file_name = OsString::from(file_name);
    temp_file_name.push(format!(".tmp.{}.{attempt}", std::process::id()));
    Ok(path.with_file_name(temp_file_name))
}

fn write_settings(
    file: &mut impl SettingsFileWriter,
    temp_path: &Path,
    content: &[u8],
) -> Result<(), SettingsSaveError> {
    file.write_all_settings(content)
        .map_err(|source| settings_write_error(temp_path, source))?;
    file.flush_settings()
        .map_err(|source| settings_write_error(temp_path, source))?;
    file.sync_all_settings()
        .map_err(|source| settings_write_error(temp_path, source))
}

fn settings_write_error(temp_path: &Path, source: io::Error) -> SettingsSaveError {
    SettingsSaveError::Write {
        path: temp_path.to_path_buf(),
        source,
    }
}

fn cleanup_temporary_settings_file(
    temp_path: &Path,
    cause: SettingsSaveError,
) -> Result<(), SettingsSaveError> {
    match fs::remove_file(temp_path) {
        Ok(()) => Err(cause),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Err(cause),
        Err(source) => Err(SettingsSaveError::Cleanup {
            path: temp_path.to_path_buf(),
            source,
            cause: Box::new(cause),
        }),
    }
}

#[cfg(windows)]
fn replace_settings_file(temp_path: &Path, target_path: &Path) -> io::Result<()> {
    let temp_path_wide = path_to_null_terminated_wide(temp_path);
    let target_path_wide = path_to_null_terminated_wide(target_path);

    // SAFETY: both path buffers are null-terminated UTF-16 and live for the call.
    let result = unsafe {
        MoveFileExW(
            temp_path_wide.as_ptr(),
            target_path_wide.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };

    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(windows)]
fn path_to_null_terminated_wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(Some(0)).collect()
}

#[cfg(not(windows))]
fn replace_settings_file(temp_path: &Path, target_path: &Path) -> io::Result<()> {
    fs::rename(temp_path, target_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum WriterStep {
        Write,
        Flush,
        Sync,
    }

    struct FakeSettingsFile {
        calls: Vec<WriterStep>,
        fail_on: Option<WriterStep>,
    }

    impl FakeSettingsFile {
        fn new() -> Self {
            Self {
                calls: Vec::new(),
                fail_on: None,
            }
        }

        fn failing(fail_on: WriterStep) -> Self {
            Self {
                calls: Vec::new(),
                fail_on: Some(fail_on),
            }
        }

        fn step_result(&self, step: WriterStep) -> io::Result<()> {
            if self.fail_on == Some(step) {
                Err(io::Error::other("forced writer failure"))
            } else {
                Ok(())
            }
        }
    }

    impl SettingsFileWriter for FakeSettingsFile {
        fn write_all_settings(&mut self, _content: &[u8]) -> io::Result<()> {
            self.calls.push(WriterStep::Write);
            self.step_result(WriterStep::Write)
        }

        fn flush_settings(&mut self) -> io::Result<()> {
            self.calls.push(WriterStep::Flush);
            self.step_result(WriterStep::Flush)
        }

        fn sync_all_settings(&mut self) -> io::Result<()> {
            self.calls.push(WriterStep::Sync);
            self.step_result(WriterStep::Sync)
        }
    }

    #[test]
    fn write_settings_flushes_and_syncs_after_write() {
        let mut file = FakeSettingsFile::new();

        let result = write_settings(&mut file, Path::new("settings.json.temp-test"), b"content");

        assert!(result.is_ok());
        assert_eq!(
            file.calls,
            [WriterStep::Write, WriterStep::Flush, WriterStep::Sync]
        );
    }

    #[test]
    fn write_settings_reports_sync_failure_as_write_error() {
        let temp_path = Path::new("settings.json.temp-test");
        let mut file = FakeSettingsFile::failing(WriterStep::Sync);

        let result = write_settings(&mut file, temp_path, b"content");

        assert_eq!(
            file.calls,
            [WriterStep::Write, WriterStep::Flush, WriterStep::Sync]
        );
        let Err(SettingsSaveError::Write { path, source }) = result else {
            panic!("expected sync failure to be reported as a write error");
        };
        assert_eq!(path, temp_path);
        assert_eq!(source.kind(), io::ErrorKind::Other);
    }
}
