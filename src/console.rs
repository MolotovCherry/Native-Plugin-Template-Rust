use std::iter;

use eyre::Result;
use windows::{
    core::PCWSTR,
    Win32::System::Console::{
        AllocConsole, GetStdHandle, SetConsoleMode, SetConsoleTitleW, ENABLE_PROCESSED_OUTPUT,
        ENABLE_VIRTUAL_TERMINAL_PROCESSING, ENABLE_WRAP_AT_EOL_OUTPUT, STD_OUTPUT_HANDLE,
    },
};

/// Note, only one console can be shown per process. So this will not work if bg3 already
/// has spawned a console (perhaps from some other plugin). As such, this is mainly for testing,
/// but you could also make it a config option which is disabled by default if you want users
/// to be able to see a console
pub fn alloc_console(title: &str) -> Result<()> {
    unsafe {
        AllocConsole()?;
    }

    let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE)? };

    unsafe {
        SetConsoleMode(
            handle,
            ENABLE_PROCESSED_OUTPUT
                | ENABLE_WRAP_AT_EOL_OUTPUT
                | ENABLE_VIRTUAL_TERMINAL_PROCESSING,
        )?;
    }

    let title = title
        .encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<_>>();

    unsafe {
        SetConsoleTitleW(PCWSTR(title.as_ptr()))?;
    }

    Ok(())
}
