use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::{ABOUT_FILE_NAME, default_about_text};

pub fn load_about_text() -> String {
    about_file_candidates()
        .into_iter()
        .find_map(read_non_empty_text)
        .unwrap_or_else(|| default_about_text().to_owned())
}

fn about_file_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = executable_directory_about_file() {
        candidates.push(path);
    }
    if let Some(path) = current_directory_about_file()
        && !candidates.iter().any(|candidate| candidate == &path)
    {
        candidates.push(path);
    }
    candidates
}

fn executable_directory_about_file() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    executable
        .parent()
        .map(|directory| directory.join(ABOUT_FILE_NAME))
}

fn current_directory_about_file() -> Option<PathBuf> {
    std::env::current_dir()
        .ok()
        .map(|path| path.join(ABOUT_FILE_NAME))
}

fn read_non_empty_text(path: PathBuf) -> Option<String> {
    read_text_if_file(path.as_path()).filter(|text| !text.trim().is_empty())
}

fn read_text_if_file(path: &Path) -> Option<String> {
    if path.is_file() {
        fs::read_to_string(path).ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::read_text_if_file;

    #[test]
    fn missing_about_file_returns_none() {
        let path = std::env::temp_dir().join(format!(
            "j3devhelper-missing-about-{}-{}.txt",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after Unix epoch")
                .as_nanos()
        ));

        assert_eq!(read_text_if_file(&path), None);
    }
}
