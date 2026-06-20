#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    match j3devhelper::app::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            report_startup_error(&format!("j3DevHelper 시작 실패: {error}"));
            ExitCode::FAILURE
        }
    }
}

#[cfg(target_os = "windows")]
fn report_startup_error(message: &str) {
    use std::iter::once;
    use std::ptr::null_mut;

    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};

    let caption: Vec<u16> = "j3DevHelper".encode_utf16().chain(once(0)).collect();
    let message: Vec<u16> = message.encode_utf16().chain(once(0)).collect();

    // SAFETY: The null owner is valid for an application-modal startup error. The caption and
    // message buffers are NUL-terminated and remain alive for the duration of the call.
    unsafe {
        MessageBoxW(
            null_mut(),
            message.as_ptr(),
            caption.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

#[cfg(not(target_os = "windows"))]
fn report_startup_error(message: &str) {
    eprintln!("{message}");
}
