use std::env;
use std::error::Error;
use std::path::Path;

const APP_ICON_PATH: &str = "icon.ico";
const APP_MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="0.2.0.0" processorArchitecture="*" name="j3DevHelper.app" type="win32" />
  <description>j3DevHelper</description>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*" />
    </dependentAssembly>
  </dependency>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v2">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2, PerMonitor</dpiAwareness>
    </windowsSettings>
  </application>
</assembly>"#;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed={APP_ICON_PATH}");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        if should_skip_windows_resource_compile() {
            println!(
                "cargo:warning=skipping Windows resource compilation: no resource compiler found on this host"
            );
            return Ok(());
        }

        let mut resource = winresource::WindowsResource::new();
        resource.set_icon(APP_ICON_PATH);
        resource.set_manifest(APP_MANIFEST);
        resource.compile()?;
    }

    Ok(())
}

fn should_skip_windows_resource_compile() -> bool {
    if env::var("HOST").ok() == env::var("TARGET").ok() {
        return false;
    }

    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let candidates = if target_env == "msvc" {
        &["rc", "rc.exe", "llvm-rc"][..]
    } else {
        &[
            "windres",
            "x86_64-w64-mingw32-windres",
            "llvm-windres",
            "llvm-rc",
        ][..]
    };

    !candidates.iter().any(|candidate| command_exists(candidate))
}

fn command_exists(command: &str) -> bool {
    let command_path = Path::new(command);
    if command_path.components().count() > 1 {
        return command_path.is_file();
    }

    let Some(paths) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&paths).any(|path| {
        let candidate = path.join(command);
        if candidate.is_file() {
            return true;
        }

        if cfg!(windows) {
            env::var_os("PATHEXT")
                .map(|extensions| {
                    env::split_paths(&extensions).any(|extension| {
                        let extension = extension.to_string_lossy();
                        let extension = extension.trim_start_matches('.');
                        path.join(format!("{command}.{extension}")).is_file()
                    })
                })
                .unwrap_or(false)
        } else {
            false
        }
    })
}
