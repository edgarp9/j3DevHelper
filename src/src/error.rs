use std::error::Error;
use std::fmt::{self, Display, Formatter};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    DesktopEntry {
        message: String,
    },
    InvalidArguments {
        message: String,
    },
    WindowsApi {
        operation: &'static str,
        code: u32,
    },
    WindowsHresult {
        operation: &'static str,
        hresult: i32,
    },
}

impl AppError {
    pub fn desktop_entry(message: impl Into<String>) -> Self {
        Self::DesktopEntry {
            message: message.into(),
        }
    }

    pub fn invalid_arguments(message: impl Into<String>) -> Self {
        Self::InvalidArguments {
            message: message.into(),
        }
    }

    pub fn windows_api(operation: &'static str, code: u32) -> Self {
        Self::WindowsApi { operation, code }
    }

    pub fn windows_hresult(operation: &'static str, hresult: i32) -> Self {
        Self::WindowsHresult { operation, hresult }
    }
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DesktopEntry { message } => write!(formatter, "{message}"),
            Self::InvalidArguments { message } => write!(formatter, "{message}"),
            Self::WindowsApi { operation, code } => {
                write!(formatter, "{operation} failed with Windows error {code}")
            }
            Self::WindowsHresult { operation, hresult } => {
                write!(
                    formatter,
                    "{operation} failed with HRESULT 0x{:08X}",
                    *hresult as u32
                )
            }
        }
    }
}

impl Error for AppError {}
