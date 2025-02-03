use std::{
    ffi::OsString,
    fs,
    os::windows::prelude::OsStringExt,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use eyre::{bail, OptionExt as _, Result};
use windows::Win32::{
    Foundation::{GetLastError, HINSTANCE, MAX_PATH},
    System::LibraryLoader::GetModuleFileNameW,
};

/// Get path to dll's parent dir
pub fn get_dll_dir(module: HINSTANCE) -> Result<&'static PathBuf> {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    const PATH_SIZE: usize = (MAX_PATH * 2) as usize;

    if let Some(path) = PATH.get() {
        return Ok(path);
    }

    // create pre-allocated stack array of correct size
    let mut path = vec![0; PATH_SIZE];
    // returns how many bytes written
    let written_len = unsafe { GetModuleFileNameW(Some(module.into()), &mut path) as usize };

    // bubble up error if there was any, for example, ERROR_INSUFFICIENT_BUFFER
    let err = unsafe { GetLastError() };
    let err = err.to_hresult();
    if err.is_err() {
        bail!("{err}");
    }

    let path = {
        let os = OsString::from_wide(&path[..written_len]);
        PathBuf::from(os)
    };

    let dll_folder = path
        .parent()
        .ok_or_eyre("Failed to get parent of dll")?
        .to_path_buf();

    PATH.set(dll_folder).unwrap();

    Ok(PATH.get().unwrap())
}

/// Get path to `<dll_dir>\logs\`
/// Also creates `logs` dir if it doesn't exist
pub fn get_dll_logs_dir(module: HINSTANCE) -> Result<PathBuf> {
    let logs_dir = get_dll_dir(module)?.join("logs");

    if !logs_dir.exists() {
        fs::create_dir(&logs_dir)?;
    }

    Ok(logs_dir)
}

/// Get path to `<dll_dir>\<filename>`
pub fn get_dll_dir_filepath<P: AsRef<Path>>(module: HINSTANCE, path: P) -> Result<PathBuf> {
    Ok(get_dll_dir(module)?.join(path))
}

/// Get path to `<dll_dir>\logs\<filename>`
/// Also creates `logs` dir if it doesn't exist
pub fn get_dll_logs_filepath<P: AsRef<Path>>(module: HINSTANCE, path: P) -> Result<PathBuf> {
    let logs_dir = get_dll_logs_dir(module)?;
    Ok(logs_dir.join(path))
}
