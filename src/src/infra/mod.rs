pub mod about;
pub mod desktop_entry;
pub mod settings;

#[cfg(target_os = "linux")]
pub mod gtk4;

#[cfg(target_os = "windows")]
pub mod win32;
