use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopEntryMetadata {
    pub application_id: &'static str,
    pub display_name: &'static str,
    pub comment: &'static str,
    pub categories: &'static str,
    pub icon_svg_file_name: &'static str,
    pub icon_png_file_name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntryInstallSummary {
    pub desktop_entry_path: PathBuf,
    pub icon_path: PathBuf,
    pub desktop_entry_changed: bool,
    pub icon_changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntryUninstallSummary {
    pub desktop_entry_path: PathBuf,
    pub icon_path: PathBuf,
    pub desktop_entry_removed: bool,
    pub icon_removed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntryError {
    message: String,
}

impl DesktopEntryError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for DesktopEntryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for DesktopEntryError {}

pub fn install(
    metadata: DesktopEntryMetadata,
) -> Result<DesktopEntryInstallSummary, DesktopEntryError> {
    imp::install(metadata)
}

pub fn uninstall(
    metadata: DesktopEntryMetadata,
) -> Result<DesktopEntryUninstallSummary, DesktopEntryError> {
    imp::uninstall(metadata)
}

#[cfg(target_os = "linux")]
mod imp {
    use std::ffi::OsString;
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    use super::{
        DesktopEntryError, DesktopEntryInstallSummary, DesktopEntryMetadata,
        DesktopEntryUninstallSummary,
    };

    const HICOLOR_PNG_ICON_SIZE_DIR: &str = "256x256";

    pub fn install(
        metadata: DesktopEntryMetadata,
    ) -> Result<DesktopEntryInstallSummary, DesktopEntryError> {
        let executable_path = current_executable_path()?;
        let icon_source =
            find_icon_source_path(metadata.icon_svg_file_name, metadata.icon_png_file_name)?;
        let data_home = xdg_data_home()?;
        let summary = install_into(metadata, &executable_path, &icon_source, &data_home)?;
        refresh_desktop_caches(&data_home);
        Ok(summary)
    }

    pub fn uninstall(
        metadata: DesktopEntryMetadata,
    ) -> Result<DesktopEntryUninstallSummary, DesktopEntryError> {
        let data_home = xdg_data_home()?;
        let paths = DesktopEntryPaths::new(&data_home, metadata.application_id);
        let current_removed = remove_application_entries(&data_home, metadata.application_id)?;
        refresh_desktop_caches(&data_home);
        Ok(DesktopEntryUninstallSummary {
            desktop_entry_path: paths.desktop_entry_path,
            icon_path: paths.icon_svg_path,
            desktop_entry_removed: current_removed.desktop_entry_removed,
            icon_removed: current_removed.icon_removed,
        })
    }

    fn current_executable_path() -> Result<PathBuf, DesktopEntryError> {
        std::env::current_exe().map_err(|source| {
            DesktopEntryError::new(format!("실행 파일 경로를 확인할 수 없습니다: {source}"))
        })
    }

    fn xdg_data_home() -> Result<PathBuf, DesktopEntryError> {
        if let Some(data_home) = non_empty_env_path("XDG_DATA_HOME") {
            if data_home.is_absolute() {
                return Ok(data_home);
            }
            return Err(DesktopEntryError::new(
                "XDG_DATA_HOME은 절대 경로여야 합니다.",
            ));
        }
        let Some(home) = non_empty_env_path("HOME") else {
            return Err(DesktopEntryError::new(
                "HOME 경로를 확인할 수 없어 desktop entry를 설치할 수 없습니다.",
            ));
        };
        Ok(home.join(".local/share"))
    }

    fn non_empty_env_path(key: &str) -> Option<PathBuf> {
        std::env::var_os(key)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    }

    fn find_icon_source_path(
        svg_file_name: &str,
        png_file_name: &str,
    ) -> Result<IconSource, DesktopEntryError> {
        let mut candidates = Vec::new();
        if let Ok(exe_path) = std::env::current_exe()
            && let Some(exe_dir) = exe_path.parent()
        {
            candidates.push(exe_dir.to_path_buf());
        }
        if let Ok(current_dir) = std::env::current_dir() {
            candidates.push(current_dir);
        }
        for candidate in &candidates {
            let path = candidate.join(svg_file_name);
            if path.is_file() {
                return Ok(IconSource {
                    path,
                    format: IconFormat::Svg,
                });
            }
        }
        for candidate in &candidates {
            let path = candidate.join(png_file_name);
            if path.is_file() {
                return Ok(IconSource {
                    path,
                    format: IconFormat::Png,
                });
            }
        }
        Err(DesktopEntryError::new(format!(
            "아이콘 파일을 찾을 수 없습니다: {svg_file_name} 또는 {png_file_name}"
        )))
    }

    fn install_into(
        metadata: DesktopEntryMetadata,
        executable_path: &Path,
        icon_source: &IconSource,
        data_home: &Path,
    ) -> Result<DesktopEntryInstallSummary, DesktopEntryError> {
        let paths = DesktopEntryPaths::new(data_home, metadata.application_id);
        fs::create_dir_all(&paths.applications_dir).map_err(|source| {
            io_error(
                "desktop entry 디렉터리를 만들 수 없습니다",
                &paths.applications_dir,
                source,
            )
        })?;
        let icon_dir = icon_source.format.icon_dir(&paths);
        fs::create_dir_all(icon_dir)
            .map_err(|source| io_error("아이콘 디렉터리를 만들 수 없습니다", icon_dir, source))?;

        let mut desktop_entry_changed = false;
        let mut icon_changed = false;
        for identity in application_identities(metadata.application_id) {
            let paths = DesktopEntryPaths::new(data_home, &identity.application_id);
            let desktop_entry =
                desktop_entry_content(metadata, executable_path, &identity.application_id);
            desktop_entry_changed |= write_text_if_changed(
                &paths.desktop_entry_path,
                &desktop_entry.with_no_display(identity.no_display),
            )?;

            let icon_path = icon_source.format.icon_path(&paths);
            icon_changed |= copy_file_if_changed(&icon_source.path, icon_path)?;
            if icon_source.format == IconFormat::Svg {
                icon_changed |= remove_file_if_exists(&paths.icon_png_path)?;
            }
        }

        let icon_path = icon_source.format.icon_path(&paths).to_path_buf();
        Ok(DesktopEntryInstallSummary {
            desktop_entry_path: paths.desktop_entry_path,
            icon_path,
            desktop_entry_changed,
            icon_changed,
        })
    }

    fn desktop_entry_content(
        metadata: DesktopEntryMetadata,
        executable_path: &Path,
        application_id: &str,
    ) -> DesktopEntryContent {
        DesktopEntryContent {
            content: format!(
                "# Managed by {} --install\n\
                 [Desktop Entry]\n\
                 Type=Application\n\
                 Name={}\n\
                 Comment={}\n\
                 Exec={}\n\
                 Icon={application_id}\n\
                 Terminal=false\n\
                 Categories={}\n\
                 StartupNotify=true\n\
                 StartupWMClass={application_id}\n",
                metadata.display_name,
                metadata.display_name,
                metadata.comment,
                desktop_exec_path(executable_path),
                metadata.categories,
            ),
        }
    }

    fn desktop_exec_path(path: &Path) -> String {
        let value = path.to_string_lossy();
        if value
            .chars()
            .all(|ch| !ch.is_whitespace() && !matches!(ch, '"' | '\\' | '$' | '`'))
        {
            return value.into_owned();
        }

        let mut escaped = String::from("\"");
        for ch in value.chars() {
            match ch {
                '"' | '\\' | '$' | '`' => {
                    escaped.push('\\');
                    escaped.push(ch);
                }
                _ => escaped.push(ch),
            }
        }
        escaped.push('"');
        escaped
    }

    struct DesktopEntryContent {
        content: String,
    }

    impl DesktopEntryContent {
        fn with_no_display(mut self, no_display: bool) -> String {
            if no_display {
                self.content.push_str("NoDisplay=true\n");
            }
            self.content
        }
    }

    fn write_text_if_changed(path: &Path, content: &str) -> Result<bool, DesktopEntryError> {
        if fs::read_to_string(path).is_ok_and(|existing| existing == content) {
            return Ok(false);
        }
        fs::write(path, content)
            .map_err(|source| io_error("desktop entry를 쓸 수 없습니다", path, source))?;
        Ok(true)
    }

    fn copy_file_if_changed(
        source_path: &Path,
        destination_path: &Path,
    ) -> Result<bool, DesktopEntryError> {
        let source = fs::read(source_path)
            .map_err(|source| io_error("아이콘 파일을 읽을 수 없습니다", source_path, source))?;
        if fs::read(destination_path).is_ok_and(|existing| existing == source) {
            return Ok(false);
        }
        fs::write(destination_path, source).map_err(|source| {
            io_error("아이콘 파일을 설치할 수 없습니다", destination_path, source)
        })?;
        Ok(true)
    }

    fn remove_file_if_exists(path: &Path) -> Result<bool, DesktopEntryError> {
        match fs::remove_file(path) {
            Ok(()) => Ok(true),
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(source) => Err(io_error("파일을 제거할 수 없습니다", path, source)),
        }
    }

    fn remove_application_entries(
        data_home: &Path,
        application_id: &str,
    ) -> Result<RemovedFiles, DesktopEntryError> {
        let mut removed = RemovedFiles::default();
        for identity in application_identities(application_id) {
            let paths = DesktopEntryPaths::new(data_home, &identity.application_id);
            removed.desktop_entry_removed |= remove_file_if_exists(&paths.desktop_entry_path)?;
            removed.icon_removed |= remove_file_if_exists(&paths.icon_svg_path)?;
            removed.icon_removed |= remove_file_if_exists(&paths.icon_png_path)?;
        }
        Ok(removed)
    }

    fn refresh_desktop_caches(data_home: &Path) {
        let applications_dir = data_home.join("applications");
        let hicolor_dir = data_home.join("icons/hicolor");
        run_cache_command(
            "update-desktop-database",
            &[applications_dir.into_os_string()],
        );
        run_cache_command(
            "gtk-update-icon-cache",
            &[
                OsString::from("-f"),
                OsString::from("-t"),
                hicolor_dir.into_os_string(),
            ],
        );
        if !run_cache_command("kbuildsycoca6", &[OsString::from("--noincremental")]) {
            run_cache_command("kbuildsycoca5", &[OsString::from("--noincremental")]);
        }
    }

    fn run_cache_command(program: &str, args: &[OsString]) -> bool {
        Command::new(program)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }

    fn io_error(action: &str, path: &Path, source: io::Error) -> DesktopEntryError {
        DesktopEntryError::new(format!("{action}: {}: {source}", path.display()))
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ApplicationIdentity {
        application_id: String,
        no_display: bool,
    }

    fn application_identities(application_id: &str) -> Vec<ApplicationIdentity> {
        let lowercase_application_id = application_id.to_ascii_lowercase();
        let mut identities = vec![ApplicationIdentity {
            application_id: application_id.to_owned(),
            no_display: false,
        }];
        if lowercase_application_id != application_id {
            identities.push(ApplicationIdentity {
                application_id: lowercase_application_id,
                no_display: true,
            });
        }
        identities
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    struct RemovedFiles {
        desktop_entry_removed: bool,
        icon_removed: bool,
    }

    #[derive(Debug, Clone)]
    struct DesktopEntryPaths {
        applications_dir: PathBuf,
        scalable_icon_dir: PathBuf,
        png_icon_dir: PathBuf,
        desktop_entry_path: PathBuf,
        icon_svg_path: PathBuf,
        icon_png_path: PathBuf,
    }

    impl DesktopEntryPaths {
        fn new(data_home: &Path, application_id: &str) -> Self {
            let applications_dir = data_home.join("applications");
            let scalable_icon_dir = data_home.join("icons/hicolor/scalable/apps");
            let png_icon_dir = data_home
                .join("icons/hicolor")
                .join(HICOLOR_PNG_ICON_SIZE_DIR)
                .join("apps");
            Self {
                desktop_entry_path: applications_dir.join(format!("{application_id}.desktop")),
                icon_svg_path: scalable_icon_dir.join(format!("{application_id}.svg")),
                icon_png_path: png_icon_dir.join(format!("{application_id}.png")),
                applications_dir,
                scalable_icon_dir,
                png_icon_dir,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct IconSource {
        path: PathBuf,
        format: IconFormat,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum IconFormat {
        Svg,
        Png,
    }

    impl IconFormat {
        fn icon_dir(self, paths: &DesktopEntryPaths) -> &Path {
            match self {
                Self::Svg => &paths.scalable_icon_dir,
                Self::Png => &paths.png_icon_dir,
            }
        }

        fn icon_path(self, paths: &DesktopEntryPaths) -> &Path {
            match self {
                Self::Svg => &paths.icon_svg_path,
                Self::Png => &paths.icon_png_path,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::time::{SystemTime, UNIX_EPOCH};

        use super::*;

        const APP_ID: &str = "io.github.Example.App";
        const APP_ID_LOWER: &str = "io.github.example.app";
        const DESKTOP_NAME: &str = "io.github.Example.App";

        struct TempDir {
            path: PathBuf,
        }

        impl TempDir {
            fn new(label: &str) -> Self {
                let stamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("system time should be after epoch")
                    .as_nanos();
                let path = std::env::temp_dir().join(format!(
                    "j3devhelper-desktop-entry-{label}-{}-{stamp}",
                    std::process::id()
                ));
                fs::create_dir_all(&path).expect("temp dir should be created");
                Self { path }
            }
        }

        impl Drop for TempDir {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.path);
            }
        }

        fn metadata() -> DesktopEntryMetadata {
            DesktopEntryMetadata {
                application_id: APP_ID,
                display_name: DESKTOP_NAME,
                comment: DESKTOP_NAME,
                categories: "Utility;",
                icon_svg_file_name: "icon.svg",
                icon_png_file_name: "icon.png",
            }
        }

        fn write_icon_source(
            root: &Path,
            file_name: &str,
            content: &[u8],
            format: IconFormat,
        ) -> IconSource {
            let source_dir = root.join("source");
            fs::create_dir_all(&source_dir).expect("source dir should be created");
            let path = source_dir.join(file_name);
            fs::write(&path, content).expect("icon source should be written");
            IconSource { path, format }
        }

        fn read_text(path: &Path) -> String {
            fs::read_to_string(path).expect("text file should be readable")
        }

        #[test]
        fn install_svg_creates_primary_and_lowercase_alias_entries_and_icons() {
            let temp = TempDir::new("install-svg-alias");
            let data_home = temp.path.join("data");
            let icon_source =
                write_icon_source(&temp.path, "icon.svg", b"<svg></svg>", IconFormat::Svg);

            let summary = install_into(
                metadata(),
                Path::new("/opt/j3/app"),
                &icon_source,
                &data_home,
            )
            .expect("install should succeed");

            let primary = DesktopEntryPaths::new(&data_home, APP_ID);
            let alias = DesktopEntryPaths::new(&data_home, APP_ID_LOWER);
            assert_eq!(summary.desktop_entry_path, primary.desktop_entry_path);
            assert_eq!(summary.icon_path, primary.icon_svg_path);
            assert!(summary.desktop_entry_changed);
            assert!(summary.icon_changed);

            let primary_entry = read_text(&primary.desktop_entry_path);
            assert!(primary_entry.contains("Name=io.github.Example.App\n"));
            assert!(primary_entry.contains("Comment=io.github.Example.App\n"));
            assert!(primary_entry.contains("Exec=/opt/j3/app\n"));
            assert!(primary_entry.contains("Icon=io.github.Example.App\n"));
            assert!(primary_entry.contains("Categories=Utility;\n"));
            assert!(primary_entry.contains("StartupWMClass=io.github.Example.App\n"));
            assert!(!primary_entry.contains("NoDisplay=true\n"));

            let alias_entry = read_text(&alias.desktop_entry_path);
            assert!(alias_entry.contains("Icon=io.github.example.app\n"));
            assert!(alias_entry.contains("StartupWMClass=io.github.example.app\n"));
            assert!(alias_entry.contains("NoDisplay=true\n"));
            assert_eq!(
                fs::read(&primary.icon_svg_path).expect("primary svg should be readable"),
                b"<svg></svg>"
            );
            assert_eq!(
                fs::read(&alias.icon_svg_path).expect("alias svg should be readable"),
                b"<svg></svg>"
            );
        }

        #[test]
        fn install_svg_removes_stale_png_icons_for_primary_and_alias() {
            let temp = TempDir::new("install-svg-removes-png");
            let data_home = temp.path.join("data");
            let icon_source =
                write_icon_source(&temp.path, "icon.svg", b"<svg></svg>", IconFormat::Svg);
            let primary = DesktopEntryPaths::new(&data_home, APP_ID);
            let alias = DesktopEntryPaths::new(&data_home, APP_ID_LOWER);
            fs::create_dir_all(&primary.png_icon_dir).expect("png dir should be created");
            fs::write(&primary.icon_png_path, b"stale").expect("primary stale png should be set");
            fs::write(&alias.icon_png_path, b"stale").expect("alias stale png should be set");

            let summary = install_into(
                metadata(),
                Path::new("/opt/j3/app"),
                &icon_source,
                &data_home,
            )
            .expect("install should succeed");

            assert!(summary.icon_changed);
            assert!(!primary.icon_png_path.exists());
            assert!(!alias.icon_png_path.exists());
        }

        #[test]
        fn install_png_fallback_creates_primary_and_alias_png_icons() {
            let temp = TempDir::new("install-png");
            let data_home = temp.path.join("data");
            let icon_source = write_icon_source(&temp.path, "icon.png", b"png", IconFormat::Png);

            let summary = install_into(
                metadata(),
                Path::new("/opt/j3/app"),
                &icon_source,
                &data_home,
            )
            .expect("install should succeed");

            let primary = DesktopEntryPaths::new(&data_home, APP_ID);
            let alias = DesktopEntryPaths::new(&data_home, APP_ID_LOWER);
            assert_eq!(summary.icon_path, primary.icon_png_path);
            assert_eq!(
                fs::read(&primary.icon_png_path).expect("primary png should be readable"),
                b"png"
            );
            assert_eq!(
                fs::read(&alias.icon_png_path).expect("alias png should be readable"),
                b"png"
            );
            assert!(!primary.icon_svg_path.exists());
            assert!(!alias.icon_svg_path.exists());
        }

        #[test]
        fn install_is_idempotent_when_content_matches() {
            let temp = TempDir::new("install-idempotent");
            let data_home = temp.path.join("data");
            let icon_source =
                write_icon_source(&temp.path, "icon.svg", b"<svg></svg>", IconFormat::Svg);
            install_into(
                metadata(),
                Path::new("/opt/j3/app"),
                &icon_source,
                &data_home,
            )
            .expect("first install should succeed");

            let summary = install_into(
                metadata(),
                Path::new("/opt/j3/app"),
                &icon_source,
                &data_home,
            )
            .expect("second install should succeed");

            assert!(!summary.desktop_entry_changed);
            assert!(!summary.icon_changed);
        }

        #[test]
        fn install_updates_desktop_entry_when_executable_path_changes() {
            let temp = TempDir::new("install-path-update");
            let data_home = temp.path.join("data");
            let icon_source =
                write_icon_source(&temp.path, "icon.svg", b"<svg></svg>", IconFormat::Svg);
            install_into(
                metadata(),
                Path::new("/opt/j3/old-app"),
                &icon_source,
                &data_home,
            )
            .expect("first install should succeed");

            let summary = install_into(
                metadata(),
                Path::new("/opt/j3/new-app"),
                &icon_source,
                &data_home,
            )
            .expect("second install should succeed");

            let primary = DesktopEntryPaths::new(&data_home, APP_ID);
            assert!(summary.desktop_entry_changed);
            assert!(!summary.icon_changed);
            assert!(read_text(&primary.desktop_entry_path).contains("Exec=/opt/j3/new-app\n"));
        }

        #[test]
        fn remove_application_entries_removes_primary_and_alias_files_idempotently() {
            let temp = TempDir::new("remove-current");
            let data_home = temp.path.join("data");
            let icon_source =
                write_icon_source(&temp.path, "icon.svg", b"<svg></svg>", IconFormat::Svg);
            install_into(
                metadata(),
                Path::new("/opt/j3/app"),
                &icon_source,
                &data_home,
            )
            .expect("install should succeed");

            let primary = DesktopEntryPaths::new(&data_home, APP_ID);
            let alias = DesktopEntryPaths::new(&data_home, APP_ID_LOWER);
            let removed =
                remove_application_entries(&data_home, APP_ID).expect("remove should succeed");

            assert!(removed.desktop_entry_removed);
            assert!(removed.icon_removed);
            assert!(!primary.desktop_entry_path.exists());
            assert!(!alias.desktop_entry_path.exists());
            assert!(!primary.icon_svg_path.exists());
            assert!(!alias.icon_svg_path.exists());

            let removed_again =
                remove_application_entries(&data_home, APP_ID).expect("second remove should work");
            assert!(!removed_again.desktop_entry_removed);
            assert!(!removed_again.icon_removed);
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod imp {
    use super::{
        DesktopEntryError, DesktopEntryInstallSummary, DesktopEntryMetadata,
        DesktopEntryUninstallSummary,
    };

    pub fn install(
        _metadata: DesktopEntryMetadata,
    ) -> Result<DesktopEntryInstallSummary, DesktopEntryError> {
        Err(DesktopEntryError::new(
            "이 플랫폼에서는 desktop entry 설치를 지원하지 않습니다.",
        ))
    }

    pub fn uninstall(
        _metadata: DesktopEntryMetadata,
    ) -> Result<DesktopEntryUninstallSummary, DesktopEntryError> {
        Err(DesktopEntryError::new(
            "이 플랫폼에서는 desktop entry 제거를 지원하지 않습니다.",
        ))
    }
}
