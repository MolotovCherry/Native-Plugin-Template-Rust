use std::fs::{File, OpenOptions};

use eyre::Result;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use windows::Win32::Foundation::HINSTANCE;

use crate::{console::alloc_console, paths::get_dll_logs_filepath};

/// Setup logging for the plugin
///
/// NOTE: Have a particularly frustrating problem that you can't find EVEN with logging?
///       Using a Windows popup or debug console might be more helpful then.
///       DO NOT rely on popups in release mode. That will break the game!
pub fn setup_logging(module: HINSTANCE) -> Result<()> {
    // get the file path to `<path_to_my_dll_folder>\logs\my-plugin.log`
    let log_path = get_dll_logs_filepath(module, "my-plugin.log")?;

    // either create log, or append to it if it already exists
    let file = if log_path.exists() {
        OpenOptions::new().append(true).open(log_path)?
    } else {
        File::create(log_path)?
    };

    // Log as debug level if compiled in debug, otherwise use info for releases
    let level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    // enable logging
    CombinedLogger::init(vec![WriteLogger::new(level, Config::default(), file)])?;

    Ok(())
}

/// Debug console to see output on
pub fn debug_console(level: LevelFilter, title: &str) -> Result<()> {
    alloc_console(title)?;

    // enable logging
    TermLogger::init(
        level,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::AlwaysAnsi,
    )?;

    Ok(())
}
