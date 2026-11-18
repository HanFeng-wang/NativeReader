#![windows_subsystem = "windows"]

mod app;
mod associations;
mod icon;
mod markdown;
mod rtf;
mod syntax;
mod theme;
mod window;

use std::path::PathBuf;

use windows::core::PCWSTR;
use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK, MB_SETFOREGROUND};

fn main() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    if let Some(command) = args.first().and_then(|value| value.to_str()) {
        match command {
            "--apply-associations" => {
                let code = match associations::apply_configured() {
                    Ok(report) if report.protected_extensions.is_empty() => 0,
                    Ok(_) => 2,
                    Err(_) => 1,
                };
                std::process::exit(code);
            }
            "--remove-associations" => {
                std::process::exit(if associations::remove_registered().is_ok() {
                    0
                } else {
                    1
                });
            }
            "--open-default-apps" => {
                associations::open_default_apps();
                std::process::exit(0);
            }
            _ => {}
        }
    }
    let initial = args
        .first()
        .filter(|value| !value.to_string_lossy().starts_with('-'))
        .map(PathBuf::from);
    if let Err(error) = window::run(initial) {
        let message: Vec<u16> = format!("{error}\0").encode_utf16().collect();
        let title: Vec<u16> = "Native Reader\0".encode_utf16().collect();
        unsafe {
            MessageBoxW(
                None,
                PCWSTR(message.as_ptr()),
                PCWSTR(title.as_ptr()),
                MB_OK | MB_ICONERROR | MB_SETFOREGROUND,
            );
        }
    }
}
