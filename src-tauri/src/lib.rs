// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod config;
mod error;
mod structs;
mod win;
mod wx;
use error::MyError;
use serde_json::Value;
use std::os::windows::process::CommandExt;
use std::process::Command;
use structs::*;

#[tauri::command(async)]
fn wx_install_loc() -> (String, String) {
    wx::install_loc()
}

#[tauri::command(async)]
fn wx_init(exe_loc: &str, version: &str) -> Result<(), MyError> {
    wx::init(exe_loc, version)
}

#[tauri::command(async)]
fn wx_open_app(file: &str) -> Result<(), MyError> {
    Command::new(&file)
        .spawn()
        .map_err(|_| MyError::RunAppError)?;
    Ok(())
}

#[tauri::command(async)]
fn wx_open_url(url: &str) -> Result<(), MyError> {
    Command::new("cmd.exe")
        .creation_flags(0x08000000)
        .arg("/C")
        .arg("start")
        .arg(&url)
        .spawn()
        .map_err(|_| MyError::RunAppError)?;
    Ok(())
}

#[tauri::command(async)]
fn wx_open_folder(file: &str) -> Result<(), MyError> {
    Command::new("explorer")
        .args([&file])
        .spawn()
        .map_err(|_| MyError::RunAppError)?;
    Ok(())
}

#[tauri::command(async)]
fn wx_do_patch(patch_info:Value) -> Result<Vec<CoexistFileInfo>, MyError> {
   wx::do_patch(patch_info)
}

#[tauri::command(async)]
fn wx_list_all() -> Result<Vec<CoexistFileInfo>, MyError> {
    wx::list_all()
}

#[tauri::command(async)]
fn wx_del_corexist(list: &str) -> Result<(), MyError> {
    let file: CoexistFileInfo = serde_json::from_str(&list).map_err(MyError::from)?;
    let files: Vec<CoexistFileInfo> = vec![file];
    wx::del_corexist(&files)
}

#[tauri::command(async)]
fn wx_read_file_status(list: &str) -> Result<Vec<CoexistFileInfo>, MyError> {
    let file: CoexistFileInfo = serde_json::from_str(&list).map_err(MyError::from)?;
    let mut files: Vec<CoexistFileInfo> = vec![file];
    wx::read_file_status(&mut files)?;
    Ok(files)
}

#[tauri::command(async)]
fn win_is_admin() -> bool {
    win::is_running_as_admin()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            wx_open_url,
            wx_install_loc,
            wx_init,
            wx_do_patch,
            wx_list_all,
            wx_read_file_status,
            wx_del_corexist,
            wx_open_app,
            wx_open_folder,
            win_is_admin
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
