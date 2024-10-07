mod backtrace;
mod config;
mod console;
mod logging;
mod panic;
mod paths;
mod popup;

use std::ffi::c_void;
use std::thread;

use eyre::{Context, Error};
use libmem::*;
use log::{error, LevelFilter};
use native_plugin_lib::declare_plugin;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::{Foundation::HINSTANCE, System::Diagnostics::Debug::IsDebuggerPresent};

use config::Config;
use logging::{debug_console, setup_logging};
use paths::get_dll_dir_filepath;
use popup::{display_popup, MessageBoxIcon};

// Declare your plugin name and description
// This will be accessible by anyone who uses the Native-Plugin-Lib to get the info
declare_plugin! {
    "MyPlugin",
    "Author",
    "My Plugin Description"
}

/// Callback which is executed after the dll is loaded. It is safe to do anything you want in this call.
/// It is HIGHLY preferred to use Init for everything and only use DllMain for very very basic tasks you
/// _have to_ use it for. thread init, running, stuff, loadlibrary, etc., literally almost everything
/// should be done inside Init.
///
/// Why? Because actually doing anything inside DllMain is a _very bad idea_.
/// Deadlocks, UB (even silent UB), and a whole host of other nasty things can happen if you
/// use DllMain for anything except simple tasks.
///
/// See articles below. You have been warned!
/// https://devblogs.microsoft.com/oldnewthing/20070904-00/?p=25283
/// https://devblogs.microsoft.com/oldnewthing/20040128-00/?p=40853
/// https://devblogs.microsoft.com/oldnewthing/20040127-00/?p=40873
/// https://devblogs.microsoft.com/oldnewthing/20100115-00/?p=15253
/// https://blog.barthe.ph/2009/07/30/no-stdlib-in-dllmai/
/// https://learn.microsoft.com/en-us/windows/win32/dlls/dllmain?redirectedfrom=MSDN (see warning section)
/// https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-best-practices
///
/// Currently [YABG3ML](https://github.com/MolotovCherry/Yet-Another-BG3-Native-Mod-Loader) will
/// execute Init fns. But other mod loaders may not (e.g. native mod loader). Keep this in mind
/// and do testing, or know that your mod may be only compatible with 1 program.
#[no_mangle]
extern "C-unwind" fn Init() {
    // do stuff
}

struct SendHandle(HINSTANCE);
unsafe impl Send for SendHandle {}
unsafe impl Sync for SendHandle {}

/// Dll entry point
///
/// You should NOT use DllMain for _anything_. See documentation on [Init] function for why.
/// STRONGLY recommended to put this code inside Init() instead.
///
/// So why is this template code inside DllMain and not Init? Because other mod loaders may not call
/// the Init callback. YABG3Ml _does_ call it though. :)
#[no_mangle]
extern "stdcall-unwind" fn DllMain(
    module: HINSTANCE,
    fdw_reason: u32,
    _lpv_reserved: *const c_void,
) -> bool {
    let handle = SendHandle(module);

    #[allow(clippy::single_match)]
    match fdw_reason {
        DLL_PROCESS_ATTACH => {
            // you should be doing stuff in a thread, cause what you can do in DllMain is very restricted.
            // the best thing would be if the loader does a callback on an exported Init fn, but unfortunately
            // many loads don't do this.
            thread::spawn(move || {
                let handle = handle;

                // Wait for debugger if in debug mode
                if cfg!(debug_assertions) {
                    let is_debugger_present = || unsafe { IsDebuggerPresent().as_bool() };

                    while !is_debugger_present() {
                        // 60hz polling
                        std::thread::sleep(std::time::Duration::from_millis(16));
                    }
                }

                // Set up a custom panic hook so we can log all panics to logfile
                panic::set_hook();

                // Note: While it's technically safe to panic across FFI with C-unwind ABI, I STRONGLY recommend to
                // catch and handle ALL panics. If you don't, you could crash the game by accident!
                //
                // catch_unwind returns a Result with the panic info, but we actually don't need it, because
                // we set a panic_hook up at top which will log all panics to the logfile.
                // if for any reason we can't actually log the panic, we *could* popup a
                // messagebox instead (for debugging use only of course)
                let result = std::panic::catch_unwind(|| {
                    // set up our actual log file handling
                    if cfg!(debug_assertions) {
                        debug_console(LevelFilter::Trace, "Native Plugin Template Debug Console")?;
                    } else {
                        setup_logging(handle.0).context("Failed to setup logging")?;
                    }

                    entry(handle.0);

                    Ok::<_, Error>(())
                });

                // If there was no panic, but error was bubbled up, then log the error
                if let Ok(Err(e)) = result {
                    error!("{e}");
                }
            });
        }

        _ => (),
    }

    true
}

// All of our main plugin code goes here!
//
// To log to the logfile, use the log macros: log::debug!(), log::info!(), log::warn!(), log::error!()
// Recommend to catch and handle potential panics instead of panicking; log instead, it's much cleaner
fn entry(module: HINSTANCE) {
    // TODO: Place all your hooking code here

    // Show the hook was injected. DO NOT popup in production code!
    display_popup(
        "Success",
        "Plugin successfully injected",
        MessageBoxIcon::Information,
    );

    // load a config
    let config_path =
        get_dll_dir_filepath(module, "my-config.toml").expect("Failed to find config path");
    let config = Config::load(config_path).expect("Failed to load config");

    todo!("Implement hooking logic");
}
