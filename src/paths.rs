use std::{
    ffi::OsString,
    fs,
    os::windows::prelude::OsStringExt,
    path::{Path, PathBuf},
};

use eyre::{bail, eyre, Result};
use windows::Win32::{
    Foundation::{GetLastError, HINSTANCE, MAX_PATH},
    System::LibraryLoader::GetModuleFileNameW,
};

/// Get path to dll `<dll_dir>\myplugin.dll`
pub fn get_dll_path(module: HINSTANCE) -> Result<PathBuf> {
    const PATH_SIZE: usize = (MAX_PATH * 2) as usize;

    // create pre-allocated stack array of correct size
    let mut path = vec![0; PATH_SIZE];
    // returns how many bytes written
    let written_len = unsafe { GetModuleFileNameW(module, &mut path) as usize };

    // bubble up error if there was any, for example, ERROR_INSUFFICIENT_BUFFER
    let err = unsafe { GetLastError() };
    let err = err.to_hresult();
    if err.is_err() {
        bail!("{err}");
    }

    let path = OsString::from_wide(&path[..written_len]);
    Ok(PathBuf::from(path))
}

/// Get path to dll's parent dir
pub fn get_dll_dir(module: HINSTANCE) -> Result<PathBuf> {
    let dll_folder = get_dll_path(module)?
        .parent()
        .ok_or(eyre!("Failed to get parent of dll"))?
        .to_path_buf();

    Ok(dll_folder)
}

/// Get path to `<dll_dir>\logs\`
/// Also creates `logs` dir if it doesn't exist
pub fn get_dll_logs_dir(module: HINSTANCE) -> Result<PathBuf> {
    let mut logs_dir = get_dll_dir(module)?;
    logs_dir.push("logs");

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
