use crate::config::*;
use crate::error::MyError;
use crate::structs::*;
use faster_hex::{hex_decode, hex_string};
use regex::Regex;
use serde_json::{Map, Value};
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// Global state
static WXINFO: OnceLock<WxInfo> = OnceLock::new();

/// Initialize with the WeChat installation path and version
pub fn init(exe_loc: &str, version: &str) -> Result<(), MyError> {
    let wx_path: WxPath = set_path_and_backup(exe_loc, version)?;
    let (wx_data, lib_data_hex, exe_data_hex) = load_file(&wx_path)?;
    let patchs = search_patchs(&lib_data_hex, &exe_data_hex, version)?;
    WXINFO.get_or_init(|| WxInfo {
        wx_path,
        wx_data,
        patchs,
    });
    Ok(())
}

/// Detect WeChat installation path on Linux
pub fn install_loc() -> (String, String) {
    let mut install_location = String::new();
    let mut install_version = String::new();

    // Search common Linux installation paths
    for base_path in LINUX_WX_PATHS {
        let base = Path::new(base_path);
        if !base.exists() {
            continue;
        }

        let exe_path = base.join(WX_EXE_NAME);
        if exe_path.exists() {
            install_location = base_path.to_string();

            // Try to detect version from directory structure
            if let Ok(entries) = fs::read_dir(base) {
                for entry in entries.filter_map(Result::ok) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("4.") {
                        let ver_lib = entry.path().join(WX_LIB_NAME);
                        if ver_lib.exists() {
                            install_version = name;
                            break;
                        }
                    }
                }
            }

            if install_version.is_empty() {
                let lib_path = base.join(WX_LIB_NAME);
                if lib_path.exists() {
                    install_version = detect_version_from_binary(&lib_path);
                }
            }

            if !install_version.is_empty() {
                break;
            }
        }
    }

    (install_location, install_version)
}

/// Try to detect WeChat version from binary strings
fn detect_version_from_binary(lib_path: &Path) -> String {
    if let Ok(data) = fs::read(lib_path) {
        let version_pattern = b"4.0.";
        for i in 0..data.len().saturating_sub(10) {
            if data[i..].starts_with(version_pattern) {
                let mut end = i + 4;
                while end < data.len() && end < i + 20 {
                    let c = data[end];
                    if c.is_ascii_digit() || c == b'.' {
                        end += 1;
                    } else {
                        break;
                    }
                }
                let ver = String::from_utf8_lossy(&data[i..end]).to_string();
                if ver.len() >= 5 {
                    return ver;
                }
            }
        }
    }
    String::from("4.0.0")
}

/// List all coexist files and their status
pub fn list_all() -> Result<Vec<CoexistFileInfo>, MyError> {
    let _ = WXINFO.get().ok_or(MyError::NeedInitFirst)?;
    list_by_name("", "")
}

/// Delete coexist files
pub fn del_corexist(files: &[CoexistFileInfo]) -> Result<(), MyError> {
    for x in files {
        if x.exe_file.exists() {
            fs::remove_file(&x.exe_file)?;
        }
        if x.lib_file.exists() {
            fs::remove_file(&x.lib_file)?;
        }
    }
    Ok(())
}

/// List coexist files by name filter
pub fn list_by_name(
    exe_filter_name: &str,
    lib_filter_name: &str,
) -> Result<Vec<CoexistFileInfo>, MyError> {
    let wx_info = WXINFO.get().ok_or(MyError::NeedInitFirst)?;
    let exe_files = walk_files(
        &wx_info.wx_path.exe_loc,
        NEW_WX_EXE_NAME,
        0,
        exe_filter_name,
    )?;
    let mut lib_files: Vec<CoexistFileInfo> = walk_files(
        &wx_info.wx_path.lib_loc,
        NEW_WX_LIB_NAME,
        1,
        lib_filter_name,
    )?;
    for x in lib_files.iter_mut() {
        for s in exe_files.iter() {
            if s.id == x.id {
                x.exe_name = s.exe_name.clone();
                x.exe_file = s.exe_file.clone();
                break;
            }
        }
    }
    read_file_status(&mut lib_files)?;
    Ok(lib_files)
}

/// Read patch status of files
pub fn read_file_status(files: &mut Vec<CoexistFileInfo>) -> Result<(), MyError> {
    let wx_info = WXINFO.get().ok_or(MyError::NeedInitFirst)?;
    for item in files {
        let lib_data: Vec<u8> = fs::read(&item.lib_file).map_err(|_| MyError::ReadFileError)?;
        let exe_data: Vec<u8> = fs::read(&item.exe_file).map_err(|_| MyError::ReadFileError)?;
        let patchs = &wx_info.patchs;
        let patchs_value = serde_json::to_value(patchs)?;
        if let Value::Object(map) = patchs_value {
            for (key, value) in map {
                let patch_option: Option<Patch> = serde_json::from_value(value)?;
                let mut status = false;
                let support = patch_option.is_some();
                if let Some(patch) = &patch_option {
                    if patch.config_item.is_search {
                        let data = if patch.config_item.which == "lib" {
                            &lib_data
                        } else {
                            &exe_data
                        };
                        let x = patch.loc[0];
                        if x.1 <= data.len() {
                            status = data[x.0..x.1] != patch.original;
                        }
                    }
                    item.patch_status.push(PatchStatus {
                        name: key,
                        support,
                        status,
                    });
                }
            }
        }
    }
    Ok(())
}

/// Walk directory for coexist files
fn walk_files(
    dir: &PathBuf,
    f_name: &str,
    typeed: usize,
    filter_name: &str,
) -> Result<Vec<CoexistFileInfo>, MyError> {
    let mut lists: Vec<CoexistFileInfo> = Vec::new();
    let r = &f_name.replace('#', "(\\d{0,1})");
    let re: Regex = Regex::new(r)?;
    let pr = &f_name.replace('#', ".{0,1}");
    let pre: Regex = Regex::new(&format!("^{}$", &pr))?;
    for entry in fs::read_dir(dir)?.filter_map(Result::ok) {
        let path = entry.path();
        if let Some(file_name) = path.file_name() {
            let name = file_name.to_string_lossy().to_string();
            if !pre.is_match(&name) || (!filter_name.is_empty() && name != filter_name) {
                continue;
            }
            if let Some(find) = re.captures(&name) {
                if let Some(v) = find.get(1) {
                    let id_str = v.as_str();
                    let id_str = if id_str.is_empty() { "-1" } else { id_str };
                    let id: i32 = id_str.parse().unwrap_or(-1);
                    let none_file = PathBuf::new();
                    let none_name = String::new();
                    let f = CoexistFileInfo {
                        id,
                        exe_name: if typeed == 1 {
                            none_name.clone()
                        } else {
                            name.clone()
                        },
                        exe_file: if typeed == 1 {
                            none_file.clone()
                        } else {
                            path.clone()
                        },
                        lib_name: if typeed == 0 { none_name } else { name.clone() },
                        lib_file: if typeed == 0 { none_file } else { path.clone() },
                        patch_status: Vec::new(),
                    };
                    lists.push(f);
                }
            }
        }
    }
    if lists.is_empty() {
        return Err(MyError::ReadDirError);
    }
    lists.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
    Ok(lists)
}

/// Apply patches and save files
pub fn do_patch(patch_info: Value) -> Result<Vec<CoexistFileInfo>, MyError> {
    let wx_info = WXINFO.get().ok_or(MyError::NeedInitFirst)?;
    let lib_loc = &wx_info.wx_path.lib_loc;
    let exe_loc = &wx_info.wx_path.exe_loc;
    let coexist_number = get_i64_from_value(&patch_info, "number");
    let is_coexist = (0..=9).contains(&coexist_number);
    let new_exe_name = get_new_exe_name(coexist_number);
    let new_lib_name = get_new_lib_name(coexist_number);

    let patchs = &wx_info.patchs;
    let patchs_value = serde_json::to_value(patchs)?;

    let mut new_exe_data = wx_info.wx_data.exe_data.clone();
    let mut new_lib_data = wx_info.wx_data.lib_data.clone();

    if let Value::Object(map) = patchs_value {
        let num_u8 = format!("{:X}", coexist_number).as_bytes()[0];
        for (_key, value) in map {
            let patch_option: Option<Patch> = serde_json::from_value(value)?;
            if let Some(mut patch) = patch_option {
                let mut is_patched = get_bool_from_value(&patch_info, patch.name.as_str());
                if is_coexist {
                    is_patched |= patch.config_item.is_force_patch;
                    if patch.config_item.is_replace_num && patch.replace_num_loc != 0 {
                        patch.patch[patch.replace_num_loc] = num_u8;
                    }
                }
                if patch.config_item.which == "exe" {
                    patched(&mut new_exe_data, &patch, is_patched)?;
                } else {
                    patched(&mut new_lib_data, &patch, is_patched)?;
                }
            }
        }
    }

    // Save files
    fs::write(exe_loc.join(&new_exe_name), &new_exe_data).map_err(|_| MyError::SaveFileError)?;
    // Make the new exe executable on Linux
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let exe_path = exe_loc.join(&new_exe_name);
        let mut perms = fs::metadata(&exe_path)
            .map_err(|_| MyError::SaveFileError)?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&exe_path, perms).map_err(|_| MyError::SaveFileError)?;
    }

    let new_lib_file = lib_loc.join(&new_lib_name);
    fs::write(&new_lib_file, &new_lib_data).map_err(|_| MyError::SaveFileError)?;

    list_by_name(&new_exe_name, &new_lib_name)
}

/// Apply or restore a patch
fn patched(data: &mut Vec<u8>, patch: &Patch, is_patch: bool) -> Result<(), MyError> {
    let patch_data = if is_patch {
        &patch.patch
    } else {
        &patch.original
    };
    for x in &patch.loc {
        data.splice(x.0..x.1, patch_data.to_owned());
    }
    Ok(())
}

/// Backup a file
pub fn backup(file: &PathBuf, backup_file: &PathBuf, replace: bool) -> Result<(), MyError> {
    let backup_exists = backup_file.exists();
    let file_exists = file.exists();
    if (!backup_exists && file_exists) || replace {
        fs::copy(file, backup_file)?;
    }
    Ok(())
}

/// Set up paths and create backups
fn set_path_and_backup(exe_loc: &str, version: &str) -> Result<WxPath, MyError> {
    let exe_loc = Path::new(exe_loc).to_path_buf();

    // On Linux, the lib may be in a version subdirectory or in the same directory
    let lib_loc_versioned = exe_loc.join(version);
    let lib_loc = if lib_loc_versioned.join(WX_LIB_NAME).exists() {
        lib_loc_versioned
    } else {
        exe_loc.clone()
    };

    let lib_file = lib_loc.join(WX_LIB_NAME);
    let exe_file = exe_loc.join(WX_EXE_NAME);

    if !lib_file.exists() || !exe_file.exists() {
        return Err(MyError::WXPathError);
    }

    // Create backups
    let lib_backup_file = lib_loc.join(WX_LIB_BAK_NAME);
    backup(&lib_file, &lib_backup_file, false)?;
    let exe_backup_file = exe_loc.join(WX_EXE_BAK_NAME);
    backup(&exe_file, &exe_backup_file, false)?;

    let wx_path = WxPath {
        exe_loc,
        lib_loc,
        exe_file,
        lib_file,
    };
    Ok(wx_path)
}

/// Load library and executable file data into memory
fn load_file(wx_path: &WxPath) -> Result<(WxData, String, String), MyError> {
    let lib_data: Vec<u8> = fs::read(&wx_path.lib_file).map_err(|_| MyError::ReadFileError)?;
    let lib_data_hex = hex_string(&lib_data);
    let exe_data: Vec<u8> = fs::read(&wx_path.exe_file).map_err(|_| MyError::ReadFileError)?;
    let exe_data_hex = hex_string(&exe_data);
    let wx_data = WxData { lib_data, exe_data };
    Ok((wx_data, lib_data_hex, exe_data_hex))
}

/// Search for all patch locations
fn search_patchs(lib_data_hex: &str, exe_data_hex: &str, version: &str) -> Result<Patchs, MyError> {
    let patch_config = PatchConfig::new(version)?;
    let patch_config_value = serde_json::to_value(patch_config)?;
    let mut json_obj = Map::new();
    if let Value::Object(map) = patch_config_value {
        for (key, value) in map {
            let config_item = serde_json::from_value(value)?;
            let p = search_patch(&key, lib_data_hex, exe_data_hex, &config_item)?;
            let p = serde_json::to_value(p)?;
            json_obj.insert(key, p);
        }
    }
    let patchs = serde_json::from_value(Value::Object(json_obj))?;
    Ok(patchs)
}

/// Search for a single patch location
fn search_patch(
    name: &str,
    lib_data: &str,
    exe_data: &str,
    config_item: &ConfigItem,
) -> Result<Option<Patch>, MyError> {
    let pattern = fix_blank(&config_item.pattern);
    if pattern.is_empty() {
        return Ok(None);
    }
    let data = if config_item.which == "exe" {
        exe_data
    } else {
        lib_data
    };
    let replace = if !config_item.replace.is_empty() {
        fix_blank(&config_item.replace)
    } else {
        pattern.clone()
    };
    let mut replace = fix_ellipsis(&replace, &pattern);
    let mut list = vec![&pattern];
    let mut replace_num_loc = 0;
    if !config_item.is_replace_num {
        list.push(&replace);
    } else {
        let r = fix_number_sign(&replace)?;
        replace_num_loc = r.0;
        replace = r.1;
    }
    for x in list {
        let r_fixed = fix_wildcard(x);
        let r = hex_search(data, &r_fixed)?;
        if r.0 {
            let patch = fix_patch_data(&replace, &r.2)?;
            let original = fix_patch_data(&pattern, &r.2)?;
            return Ok(Some(Patch {
                name: name.to_owned(),
                loc: r.1,
                replace_num_loc,
                original,
                patch,
                config_item: config_item.clone(),
            }));
        }
    }
    Err(MyError::SearchPatchLocError(name.to_owned()))
}

/// Search hex data for a pattern
type HexSearchResult = (bool, Vec<(usize, usize)>, String);

fn hex_search(data: &str, reg_text: &str) -> Result<HexSearchResult, MyError> {
    let reg = Regex::new(&reg_text.to_ascii_lowercase()).map_err(MyError::from)?;
    let mut locs: Vec<(usize, usize)> = Vec::new();
    let mut s = String::new();
    let mut isfind = false;
    if let Some(find) = reg.captures(data) {
        for x in find.iter().flatten() {
            locs.push((x.start() / 2, x.end() / 2));
            s = x.as_str().into();
            isfind = true;
        }
    }
    Ok((isfind, locs, s))
}

// ===== Utility functions =====

/// Remove whitespace
fn fix_blank(text: &str) -> String {
    let re_blank = Regex::new(r"\s").unwrap();
    re_blank.replace_all(text, "").into()
}

/// Find and replace ## with FF, return position
fn fix_number_sign(text: &str) -> Result<(usize, String), MyError> {
    let index = text.find("##").ok_or(MyError::PatternError)? / 2;
    Ok((index, text.replace("##", "FF")))
}

/// Replace # in file name with number
fn fix_corexist_file_name(text: &str, num: i64) -> String {
    text.replace('#', &format!("{}", num))
}

/// Replace ? with . for regex
fn fix_wildcard(text: &str) -> String {
    let re_blank = Regex::new(r"\?").unwrap();
    re_blank.replace_all(text, ".").into()
}

/// Handle ... ellipsis in patterns
fn fix_ellipsis(text: &str, text2: &str) -> String {
    if let Some(index) = text.find("...") {
        let l_text = &text[..index];
        let r_text = &text[index + 3..];
        let m_text = &text2[l_text.len()..text2.len() - r_text.len()];
        return Cow::from(format!("{}{}{}", l_text, m_text, r_text)).into();
    }
    text.into()
}

/// Process patch data, replacing ?? with actual bytes
fn fix_patch_data(text: &str, text2: &str) -> Result<Vec<u8>, MyError> {
    if text.len() != text2.len() {
        return Err(MyError::FixPatchDataError);
    }
    let mut u1 = text.as_bytes().to_owned();
    let u2 = text2.as_bytes();
    for (i, x) in u1.iter_mut().enumerate() {
        if *x == b'?' {
            *x = u2[i];
        }
    }
    let s = String::from_utf8(u1).map_err(|_| MyError::FixPatchDataError)?;
    let mut buf = vec![0u8; s.len() / 2];
    hex_decode(s.as_bytes(), &mut buf).map_err(|_| MyError::FixPatchDataError)?;
    Ok(buf)
}

/// Extract i64 from JSON value
fn get_i64_from_value(v: &Value, key: &str) -> i64 {
    if let Some(val) = v.get(key) {
        if let Some(n) = val.as_i64() {
            return n;
        }
        if let Some(s) = val.as_str() {
            return s.parse().unwrap_or(-1);
        }
    }
    -1
}

/// Extract bool from JSON value
fn get_bool_from_value(v: &Value, key: &str) -> bool {
    if let Some(val) = v.get(key) {
        if let Some(b) = val.as_bool() {
            return b;
        }
    }
    false
}

/// Generate new exe name for coexist instance
fn get_new_exe_name(num: i64) -> String {
    if (0..=9).contains(&num) {
        fix_corexist_file_name(NEW_WX_EXE_NAME, num)
    } else {
        WX_EXE_NAME.to_string()
    }
}

/// Generate new lib name for coexist instance
fn get_new_lib_name(num: i64) -> String {
    if (0..=9).contains(&num) {
        fix_corexist_file_name(NEW_WX_LIB_NAME, num)
    } else {
        WX_LIB_NAME.to_string()
    }
}
