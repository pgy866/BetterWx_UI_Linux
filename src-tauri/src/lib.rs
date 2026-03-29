mod config;
mod error;
mod structs;
mod win;
mod wx;

use error::MyError;
use structs::CoexistFileInfo;

#[tauri::command]
fn wx_install_loc() -> (String, String) {
    wx::install_loc()
}

#[tauri::command]
fn wx_init(exe_loc: &str, version: &str) -> Result<(), MyError> {
    wx::init(exe_loc, version)
}

#[tauri::command]
fn wx_list_all() -> Result<Vec<CoexistFileInfo>, MyError> {
    wx::list_all()
}

#[tauri::command]
fn wx_do_patch(patch_info: serde_json::Value) -> Result<Vec<CoexistFileInfo>, MyError> {
    wx::do_patch(patch_info)
}

#[tauri::command]
fn wx_del_corexist(list: String) -> Result<(), MyError> {
    let files: Vec<CoexistFileInfo> =
        serde_json::from_str(&list).map_err(|e| MyError::SerdeJsonError(e))?;
    wx::del_corexist(&files)
}

#[tauri::command]
fn wx_open_folder(file: &str) -> Result<(), MyError> {
    // Use xdg-open on Linux to open file manager
    std::process::Command::new("xdg-open")
        .arg(file)
        .spawn()
        .map_err(|_| MyError::RunAppError)?;
    Ok(())
}

#[tauri::command]
fn wx_open_url(url: &str) -> Result<(), MyError> {
    // Use xdg-open on Linux to open URLs in default browser
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map_err(|_| MyError::RunAppError)?;
    Ok(())
}

#[tauri::command]
fn wx_open_app(file: &str) -> Result<(), MyError> {
    // Launch the WeChat executable on Linux
    std::process::Command::new(file)
        .spawn()
        .map_err(|_| MyError::RunAppError)?;
    Ok(())
}

#[tauri::command]
fn win_is_admin() -> bool {
    win::is_running_as_root()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            wx_install_loc,
            wx_init,
            wx_list_all,
            wx_do_patch,
            wx_del_corexist,
            wx_open_folder,
            wx_open_url,
            wx_open_app,
            win_is_admin,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
