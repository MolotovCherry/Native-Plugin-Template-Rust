mod backtrace;
mod config;
mod console;
mod logging;
mod panic_hook;
mod paths;
mod popup;
mod utils;

use std::{ffi::c_void, mem, panic, sync::OnceLock, thread, time};

use extern_c::extern_system;
use eyre::{Context, ContextCompat, Error};
// this imports all of libmem's functions so you can use them
// alternatively, you can import the specific ones you want to use
// instead of a glob import
use libmem::*;
use log::{LevelFilter, error};
use native_plugin_lib::declare_plugin;
use windows::{
    Win32::{
        Foundation::{CloseHandle, HINSTANCE, TRUE},
        System::{
            Diagnostics::Debug::IsDebuggerPresent,
            SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
            Threading::{
                CreateThread, OpenEventW, SYNCHRONIZATION_SYNCHRONIZE,
                THREAD_CREATE_RUN_IMMEDIATELY,
            },
        },
    },
    core::{BOOL, w},
};

use config::Config;
use logging::{debug_console, setup_logging};
use paths::get_dll_dir_filepath;
use popup::{MessageBoxIcon, display_popup};
use utils::ThreadedWrapper;

static MODULE: OnceLock<ThreadedWrapper<HINSTANCE>> = OnceLock::new();

// Declare your plugin name and description
// This will be accessible by anyone who uses the Native-Plugin-Lib to get the info
declare_plugin! {
    "MyPlugin",
    "Author",
    "My Plugin Description"
}

/// All of our main plugin code goes here!
///
/// To log to the logfile, use the log macros: `log::debug!()`, `log::info!()`, `log::warn!()`, `log::error!()`
/// Recommend to catch and handle potential panics instead of panicking; log instead, it's much cleaner
///
/// You can use tracing for logging if you prefer a much higher quality logger, but its api is also
/// much more complex, and as such is harder to learn
fn entry(module: HINSTANCE) {
    // Show the hook was injected. DO NOT popup in production code! This is just for a POC
    display_popup(
        "Success",
        "Plugin successfully injected",
        MessageBoxIcon::Information,
    );

    // load a config
    let config_path =
        get_dll_dir_filepath(module, "my-config.toml").expect("Failed to find config path");
    let config = Config::load(config_path).expect("Failed to load config");
    // TODO: Do something with config

    todo!("Implement libmem/memory lib hooking logic");
}

/// Callback which is executed after the dll is loaded. It is safe to do anything you want in this call.
/// It is HIGHLY preferred to use Init for everything and only use `DllMain` for very very basic tasks you
/// _have to_ use it for. thread init, running, stuff, loadlibrary, etc., literally almost everything
/// should be done inside Init.
///
/// [YABG3NML](https://github.com/MolotovCherry/Yet-Another-BG3-Native-Mod-Loader) will
/// natively call Init. But other mod loaders may not (e.g. native mod loader). Keep this in mind
/// and do testing. This Init fn also executes in a new thread from `DllMain` due to compatibility reasons.
/// So while doing anything here is safe from yabg3nml, it is not necessarily from `DllMain`. This
/// template is already set up to run only Init in yabg3nml and fallback to running Init in `DllMain`
/// for other ones.
#[unsafe(no_mangle)]
extern "C-unwind" fn Init() {
    // Set up a custom panic hook so we can log all panics to logfile
    // This is also only triggered once. Safe to call it multiple times.
    panic_hook::set_hook();

    // Note: While it's technically safe to panic across FFI with C-unwind ABI, I STRONGLY recommend to
    // catch and handle ALL panics. If you don't, you could crash the game by accident!
    //
    // catch_unwind returns a Result with the panic info, but we actually don't need it, because
    // we set a panic_hook up at top which will log all panics to the logfile.
    // if for any reason we can't actually log the panic, we *could* popup a
    // messagebox instead (for debugging use only of course)
    let result = panic::catch_unwind(|| {
        let module = *MODULE
            .get()
            .context("HINSTANCE not set in DllMain")?
            .inner();

        // set up our actual log file handling
        if cfg!(debug_assertions) {
            debug_console(LevelFilter::Trace, "Native Plugin Template Debug Console")
                .context("debug console spawn failed")?;
        } else {
            setup_logging(module).context("failed to setup logging")?;
        }

        entry(module);

        Ok::<_, Error>(())
    });

    match result {
        // all good
        Ok(Ok(_)) => (),
        // there was no panic, but an error was bubbled up, so log the error
        Ok(Err(e)) => error!("{e}"),
        // a panic was caught!
        //
        // dropping the panic payload may itself panic, so we should forget it.
        // > Finally, be careful in how you drop the result of this function. If it is Err, it
        // > contains the panic payload, and dropping that may in turn panic!
        //
        // we also don't need to handle this panic cause our custom panic hook already did
        Err(e) => mem::forget(e),
    }
}

/// Dll entry point
///
/// You should NOT use `DllMain` for _anything_.
///
/// Why? Because actually doing anything inside `DllMain` is a _very bad idea_.
/// Deadlocks, UB (even silent UB), and a whole host of other nasty things can happen if you
/// use `DllMain` for anything except simple tasks.
///
/// > The entry-point function should perform only simple initialization or termination tasks.
///
/// <https://learn.microsoft.com/en-us/windows/win32/dlls/dllmain#remarks>
///
/// Unfortunately though, some mod loaders may only execute this entry point.
/// If the mod loader you're designing for only loads from this entry point
/// then you have to launch init code from a new thread inside `DllMain`.
/// > Call CreateThread. Creating a thread can work if you do not synchronize with
/// > other threads, but it is risky.
///
/// Note that if you do init here AND have your init code in `Init()`, then you're
/// effectively doing init TWICE in YABG3NML, which you don't want to do.
/// We solve this by having a special call which detects if yabg3nml
/// was the one that responsible for loading this. It's safe to call from `DllMain`.
/// It can be used to noop `DllMain`, but otherwise fallthrough to fallback execution.
/// We define the Init in the exported Init fn and call that in the fallback here.
/// So everybody's happy.
///
/// See articles below. You have been warned!
/// <https://devblogs.microsoft.com/oldnewthing/20070904-00/?p=25283>
/// <https://devblogs.microsoft.com/oldnewthing/20040128-00/?p=40853>
/// <https://devblogs.microsoft.com/oldnewthing/20040127-00/?p=40873>
/// <https://devblogs.microsoft.com/oldnewthing/20100115-00/?p=15253>
/// <https://blog.barthe.ph/2009/07/30/no-stdlib-in-dllmai/>
/// <https://learn.microsoft.com/en-us/windows/win32/dlls/dllmain?redirectedfrom=MSDN> (see warning section)
/// <https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-best-practices>
#[unsafe(no_mangle)]
extern "system-unwind" fn DllMain(
    module: HINSTANCE,
    fdw_reason: u32,
    _lpv_reserved: *const c_void,
) -> BOOL {
    match fdw_reason {
        DLL_PROCESS_ATTACH => 'attach: {
            // basic dll init code here

            // If you're getting a hang on the game when you start it, it's because you compiled in debug mode,
            // haven't attached a debugger, and this code here is still enabled!
            //
            // If you don't want to wait to ever attach a debugger, then comment or remove this line
            if cfg!(debug_assertions) {
                let is_debugger_present = || unsafe { IsDebuggerPresent().as_bool() };
                while !is_debugger_present() {
                    // 60hz polling
                    thread::sleep(time::Duration::from_millis(16));
                }
            }

            // Note about calling `DisableThreadLibraryCalls`. By default crt static is enabled (see .cargo/config.toml),
            // so you should not call this function unless you remove `-Ctarget-feature=+crt-static` from the file.
            //
            // > Consider calling DisableThreadLibraryCalls when receiving DLL_PROCESS_ATTACH, unless your DLL is
            // > linked with static C run-time library (CRT).

            _ = MODULE.set(unsafe { ThreadedWrapper::new(module) });

            // noop if it was called from yabg3nml
            // because we prefer to call actual init functionality properly instead of in DllMain where there can be problems
            // but we will fallback to calling Init below anyways since we have no choice
            if is_yabg3nml() {
                break 'attach;
            }

            // > Call CreateThread. Creating a thread can work if you do not synchronize with
            //   other threads, but it is risky.

            // we do not use thread::spawn because we have no guarantee
            // what will change in its implementation; calling this directly gives us more safety
            _ = unsafe {
                CreateThread(
                    None,
                    0,
                    Some(extern_system(|_: *mut c_void| {
                        Init();
                        0
                    })),
                    None,
                    THREAD_CREATE_RUN_IMMEDIATELY,
                    None,
                )
            };
        }

        DLL_PROCESS_DETACH => {
            // TODO: deinit code here
        }

        _ => (),
    }

    TRUE
}

/// Detects if yabg3nml injected this dll.
/// This is safe to use from `DllMain`
fn is_yabg3nml() -> bool {
    static CACHE: OnceLock<bool> = OnceLock::new();

    *CACHE.get_or_init(|| {
        match unsafe {
            OpenEventW(
                SYNCHRONIZATION_SYNCHRONIZE,
                false,
                w!(r"Global\yet-another-bg3-native-mod-loader"),
            )
        } {
            Ok(h) => {
                _ = unsafe { CloseHandle(h) };
                true
            }

            Err(_) => false,
        }
    })
}
