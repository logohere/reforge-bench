#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments
        .first()
        .is_some_and(|argument| argument == "inspect")
    {
        #[cfg(unix)]
        let result = rea_rune_ui::inspect(&arguments[1..]);
        #[cfg(not(unix))]
        let result: Result<(), String> =
            Err("The developer socket inspector is available only on macOS.".into());
        if let Err(error) = result {
            eprintln!("Reforge inspector: {error}");
            std::process::exit(1);
        }
        return;
    }
    if let Err(error) = rea_rune_ui::set_app_icon().and_then(|()| rea_rune_ui::run(arguments)) {
        eprintln!("Reforge: {error}");
        #[cfg(target_os = "windows")]
        {
            use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
            let message: Vec<u16> = error
                .replace('\0', "")
                .encode_utf16()
                .chain(Some(0))
                .collect();
            let title: Vec<u16> = "Reforge Bench".encode_utf16().chain(Some(0)).collect();
            // SAFETY: both strings are NUL-terminated and live for the modal call.
            unsafe {
                MessageBoxW(
                    std::ptr::null_mut(),
                    message.as_ptr(),
                    title.as_ptr(),
                    MB_OK | MB_ICONERROR,
                );
            }
        }
        std::process::exit(1);
    }
}
