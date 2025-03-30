use crate::config::*;
use crate::error::MyError;
use crate::structs::*;
use faster_hex::{hex_decode, hex_string};
use regex::Regex;
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use windows_registry::LOCAL_MACHINE;
//全局变量
static WXINFO: OnceLock<WxInfo> = OnceLock::new();

/**
 * init
 */

pub fn init(exe_loc: &str, version: &str) -> Result<(), MyError> {
    let wx_path: WxPath = set_path_and_backup(exe_loc, version)?;
    let (wx_data, dll_data_hex, exe_data_hex) = load_file(&wx_path)?;
    let patchs = search_patchs(&dll_data_hex, &exe_data_hex, version)?;
    WXINFO.get_or_init(|| WxInfo {
        wx_path,
        wx_data,
        patchs,
    });
    Ok(())
}

/**
 * 获取安装路径
 */
pub fn install_loc() -> (String, String) {
    let mut install_location = String::from("");
    let mut install_version = String::from("");
    if let Ok(key) = LOCAL_MACHINE
        .create("SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Weixin")
    {
        if let Ok(loc) = key.get_string("InstallLocation") {
            install_location = loc
        }
        if let Ok(ver) = key.get_string("DisplayVersion") {
            install_version = ver
        }
    }
    (install_location, install_version)
}

/**
 * 读取所有共存文件的  状态
 */
pub fn list_all() -> Result<Vec<CoexistFileInfo>, MyError> {
    let _ = WXINFO.get().ok_or(MyError::NeedInitFirst)?;
    list_by_name("", "")
}

/**
 * 删除除共存文件
 */
pub fn del_corexist(files: &Vec<CoexistFileInfo>) -> Result<(), MyError> {
    for x in files {
        fs::remove_file(&x.exe_file)?;
        fs::remove_file(&x.dll_file)?;
    }
    Ok(())
}

/**
 * 读取共存文件的状态
 */
pub fn list_by_name(
    exe_filter_name: &str,
    dll_filter_name: &str,
) -> Result<Vec<CoexistFileInfo>, MyError> {
    let wx_info = WXINFO.get().ok_or(MyError::NeedInitFirst)?;
    let exe_files = walk_files(
        &wx_info.wx_path.exe_loc,
        NEW_WX_EXE_NAME,
        0,
        exe_filter_name,
    )?;
    let mut dll_files: Vec<CoexistFileInfo> = walk_files(
        &wx_info.wx_path.dll_loc,
        NEW_WX_DLL_NAME,
        1,
        dll_filter_name,
    )?;
    //合并dll 和 exe
    for x in dll_files.iter_mut() {
        for s in (&exe_files).iter() {
            if &s.id == &x.id {
                (*x).exe_name = s.exe_name.clone();
                (*x).exe_file = s.exe_file.clone();
                break;
            }
        }
    }
    read_file_status(&mut dll_files)?;
    Ok(dll_files)
}

/**
 * 读取文件的 补丁 状态
 */
pub fn read_file_status(files: &mut Vec<CoexistFileInfo>) -> Result<(), MyError> {
    let wx_info = WXINFO.get().ok_or(MyError::NeedInitFirst)?;
    let patchs = &wx_info.patchs;
    for item in files {
        let dll_data: Vec<u8> = fs::read(&item.dll_file).map_err(|_| MyError::ReadFileError)?;
        item.unlock = if &patchs.unlock.is_none() == &true {
            item.unlock_unsupport = true;
            false
        } else {
            let unlock = patchs.unlock.clone().unwrap();
            let x = unlock.loc[0];
            if &dll_data[x.0..x.1] == unlock.original {
                false
            } else {
                true
            }
        };
        item.revoke = if patchs.revoke.is_none() {
            item.revoke_unsupport = true;
            false
        } else {
            let revoke = patchs.revoke.clone().unwrap();
            let x = revoke.loc[0];
            if &dll_data[x.0..x.1] == revoke.original {
                false
            } else {
                true
            }
        };
    }
    Ok(())
}

/**
 * 获取 共存文件列表
 */
fn walk_files(
    dir: &PathBuf,
    f_name: &str,
    typeed: usize,
    filter_name: &str,
) -> Result<Vec<CoexistFileInfo>, MyError> {
    let mut lists: Vec<CoexistFileInfo> = Vec::new();
    let r = &f_name.replace("#", "(\\d{0,1})");
    let re: Regex = Regex::new(&format!("{}", &r))?;
    let pr = &f_name.replace("#", ".{0,1}");
    let pre: Regex = Regex::new(&format!("{}{}{}", "^", &pr, "$"))?;
    for entry in fs::read_dir(dir)?.filter_map(Result::ok) {
        let path = &entry.path();
        if let Some(file_name) = path.file_name() {
            let name = &String::from(file_name.to_string_lossy());
            //长度一致
            if !pre.is_match(&name) || (&filter_name != &"" && &name != &filter_name) {
                continue;
            }
            //取出序号
            let find = re.captures(&name);
            let none_file = Path::new("").join("");
            let none_name = "".to_string();
            let exe_name = if typeed == 1 { &none_name } else { &name };
            let exe_file = if typeed == 1 { &none_file } else { &path };
            let dll_name = if typeed == 0 { &none_name } else { &name };
            let dll_file = if typeed == 0 { &none_file } else { &path };
            for item in find.iter() {
                if let Some(v) = item.get(1) {
                    let id = v.as_str();
                    let id = if id == "" { "-1" } else { id };
                    let id = (&id).parse().unwrap_or(-1);
                    let f = CoexistFileInfo {
                        id,
                        exe_name: exe_name.clone(),
                        exe_file: exe_file.clone(),
                        dll_name: dll_name.clone(),
                        dll_file: dll_file.clone(),
                        revoke: false,
                        revoke_unsupport: false,
                        unlock_unsupport: false,
                        unlock: false,
                    };
                    lists.push(f);
                }
            }
        }
    }
    if (&lists).len() < 1 {
        return Err(MyError::ReadDirRrror);
    }

    lists.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
    Ok(lists)
}

/**
 * 修补保存文件
 */
pub fn do_patch(
    is_unlock: bool,
    is_revoke: bool,
    coexist_number: i32,
) -> Result<Vec<CoexistFileInfo>, MyError> {
    println!("do_patch {} {} {}", is_unlock, is_revoke, coexist_number);
    let wx_info = WXINFO.get().ok_or(MyError::NeedInitFirst)?;
    let mut new_dll_data = (&wx_info.wx_data.dll_data).clone();
    println!("new_dll_data  {}", new_dll_data.len());
    let dll_loc = &wx_info.wx_path.dll_loc;
    let exe_loc = &wx_info.wx_path.exe_loc;
    let is_coexist = coexist_number <= 9 && coexist_number >= 0;
    let new_exe_name = if is_coexist {
        &fix_corexist_file_name(NEW_WX_EXE_NAME, coexist_number)
    } else {
        WX_EXE_NAME
    };
    println!("new_exe_name len {}", new_exe_name);
    let new_dll_name = if is_coexist {
        &fix_corexist_file_name(NEW_WX_DLL_NAME, coexist_number)
    } else {
        WX_DLL_NAME
    };
    println!("new_dll_name len {}", new_dll_name);
    //coexist_patch

    if is_coexist {
        let num_u8 = format!("{:X}", coexist_number).as_bytes()[0];
        //patch exe
        let mut new_exe_data = (&wx_info.wx_data.exe_data).clone();
        //修改 Weixin.exe 的 Weixin.dll 为 Weixin.dl{n}
        if let Some(_) = &wx_info.patchs.dllname {
            let mut exe_patch = wx_info.patchs.dllname.clone().unwrap();
            let patch = &exe_patch.patch;
            let l = patch.len();
            exe_patch.patch[l - 1] = num_u8;
            patched(&mut new_exe_data, &exe_patch, true)?;
            println!("patched dllname  {}", true);
        }

        // 修改 dll 的 config 为conf{n}g  COEXIST_CONFIG_PATTERN 为 69 COEXIST_CONFIG_REPLACE 为 FF
        // 把FF 修改为 num_u8
        if let Some(_) = &wx_info.patchs.config {
            let mut config_patch = (&wx_info.patchs.config).clone().unwrap();
            for (i, x) in config_patch.patch.iter_mut().enumerate() {
                if x == &(255 as u8) && config_patch.original[i] == 105 {
                    *x = num_u8;
                }
            }
            patched(&mut new_dll_data, &config_patch, true)?;
            println!("patched config_patch {}", true);
        }
        //host patch 和自动登入相关
        if let Some(_) = &wx_info.patchs.host {
            let mut host_patch = (&wx_info.patchs.host).clone().unwrap();
            let patch = &host_patch.patch;
            let l = patch.len();
            host_patch.patch[l - 1] = num_u8;
            patched(&mut new_dll_data, &host_patch, true)?;
            println!("patched host {}", true);
        }

        // 1.0.2 lock.ini
        //host patch 和自动登入相关
        if let Some(_) = &wx_info.patchs.lockini {
            let mut lockini_patch = (&wx_info.patchs.lockini).clone().unwrap();
            let patch = &lockini_patch.patch;
            let l = patch.len();
            lockini_patch.patch[l - 1] = num_u8;
            patched(&mut new_dll_data, &lockini_patch, true)?;
            println!("patched host {}", true);
        }
    }
    if let Some(unlock) = &wx_info.patchs.unlock {
        patched(
            &mut new_dll_data,
            unlock,
            (unlock.is_force_unlock && is_coexist) | is_unlock,
        )?;
        println!(
            "patched unlock {}",
            (unlock.is_force_unlock && is_coexist) | is_unlock
        );
    }

    //revoke_patch
    if let Some(revoke) = &wx_info.patchs.revoke {
        patched(
            &mut new_dll_data,
            revoke,
            (revoke.is_force_unlock && is_coexist) | is_revoke,
        )?;
        println!(
            "patched revoke {}",
            (revoke.is_force_unlock && is_coexist) | is_revoke
        );
    }

    //save
    // 保存 Weixin.exe 为 Weixin{n}.exe
    fs::write(exe_loc.join(&new_exe_name), &new_exe_data).map_err(|_| MyError::SaveFileError)?;
    let new_dll_file = &dll_loc.join(&new_dll_name);
    fs::write(&new_dll_file, &new_dll_data).map_err(|_| MyError::SaveFileError)?;
    list_by_name(new_exe_name, new_dll_name)
}

/**
 * patched
 */
fn patched(data: &mut Vec<u8>, patch: &Patch, is_patch: bool) -> Result<(), MyError> {
    let mut patch_data = &patch.patch;
    if !is_patch {
        patch_data = &patch.original;
    }
    for x in &patch.loc {
        data.splice(x.0..x.1, patch_data.to_owned());
    }
    Ok(())
}

/**
 * 备份文件
 */
pub fn backup(file: &PathBuf, backup_file: &PathBuf, replace: bool) -> Result<(), MyError> {
    let t = fs::exists(backup_file).map_err(|_| MyError::WXPathError)?;
    let t1 = fs::exists(file).map_err(|_| MyError::WXPathError)?;
    if !t && t1 || replace {
        fs::copy(file, backup_file)?;
    }
    Ok(())
}

/**
 * set_path_and_backup
 */
fn set_path_and_backup(exe_loc: &str, version: &str) -> Result<WxPath, MyError> {
    let exe_loc = Path::new(&exe_loc).join("");
    let dll_loc = exe_loc.join(&version);
    let dll_file = dll_loc.join(WX_DLL_NAME);
    let t1 = fs::exists(&dll_file).map_err(|_| MyError::WXPathError)?;
    let exe_file = exe_loc.join(WX_EXE_NAME);
    let t2 = fs::exists(&exe_file).map_err(|_| MyError::WXPathError)?;
    if t1 && t2 {
        //备份文件
        let dll_backup_file = dll_loc.join(WX_DLL_BAK_NAME);
        backup(&dll_file, &dll_backup_file, false)?;
        let exe_backup_file = exe_loc.join(WX_EXE_BAK_NAME);
        backup(&exe_file, &exe_backup_file, false)?;
        let wx_path: WxPath = WxPath {
            exe_loc,
            dll_loc,
            exe_file,
            dll_file,
        };
        return Ok(wx_path);
    }
    Err(MyError::WXPathError)
}

/**
 * 加载 DLL 和 EXE 文件数据到内存
 */
fn load_file(wx_path: &WxPath) -> Result<(WxData, String, String), MyError> {
    let dll_data: Vec<u8> = fs::read(&wx_path.dll_file).map_err(|_| MyError::ReadFileError)?;
    let dll_data_hex = hex_string(&dll_data);
    let exe_data: Vec<u8> = fs::read(&wx_path.exe_file).map_err(|_| MyError::ReadFileError)?;
    let exe_data_hex = hex_string(&exe_data);
    let wx_data = WxData { dll_data, exe_data };
    Ok((wx_data, dll_data_hex, exe_data_hex))
}

/**
 * 搜索 所有 patch 位置
 */
fn search_patchs(dll_data_hex: &str, exe_data_hex: &str, version: &str) -> Result<Patchs, MyError> {
    let patch_config = PatchConfig::new(version);
    let unlock = search_patch("unlock", &dll_data_hex, &exe_data_hex, &patch_config.unlock)?;
    println!("unlock   {:?}", unlock);
    let revoke = search_patch("revoke", &dll_data_hex, &exe_data_hex, &patch_config.revoke)?;
    println!("revoke   {:?}", revoke);
    let config = search_patch("config", &dll_data_hex, &exe_data_hex, &patch_config.config)?;
    println!("config   {:?}", config);
    let host = search_patch("host", &dll_data_hex, &exe_data_hex, &patch_config.host)?;
    println!("host   {:?}", host);
    let lockini = search_patch(
        "lockini",
        &dll_data_hex,
        &exe_data_hex,
        &patch_config.lockini,
    )?;
    println!("lockini   {:?}", lockini);
    let dllname = search_patch(
        "dllname",
        &dll_data_hex,
        &exe_data_hex,
        &patch_config.dllname,
    )?;
    println!("dllname   {:?}", dllname);
    let patchs = Patchs {
        unlock,
        revoke,
        config,
        host,
        dllname,
        lockini,
    };

    Ok(patchs)
}

/**
 * 搜索 patch 位置
 */
fn search_patch(
    name: &str,
    dll_data: &str,
    exe_data: &str,
    config_item: &ConfigItem,
) -> Result<Option<Patch>, MyError> {
    let data = if config_item.1 == "exe" {
        exe_data
    } else {
        dll_data
    };
    //去除空格
    let pattern = fix_blank(&config_item.2);
    if pattern == "" {
        return Ok(None);
    };
    let replace = if &config_item.3 != "" {
        &config_item.3
    } else {
        &config_item.2
    };
    //去除空格 修复省略
    let replace = fix_ellipsis(&fix_blank(replace), &pattern);
    //?? 转换 ..
    let list = [&pattern, &replace];
    for x in list {
        let r_fixed = fix_wildcard(&x);
        let r = hex_search(&data, &r_fixed)?;
        if r.0 {
            let patch = fix_patch_data(&replace, &r.2)?;
            let original = fix_patch_data(&pattern, &r.2)?;
            return Ok(Some(Patch {
                name: name.to_owned(),
                loc: r.1,
                original,
                patch,
                is_force_unlock: config_item.4,
            }));
        }
    }
    Err(MyError::SearchPatchLocError(name.to_owned()))
}

/**
 * hex_查找 位置
 */
fn hex_search(data: &str, reg_text: &str) -> Result<(bool, Vec<(usize, usize)>, String), MyError> {
    let reg = Regex::new(&reg_text.to_ascii_lowercase()).map_err(MyError::from)?;
    let mut locs: Vec<(usize, usize)> = Vec::new();
    let mut s = String::from("");
    let mut isfind = false;
    if let Some(find) = reg.captures(&data) {
        for item in find.iter() {
            if let Some(x) = item {
                locs.push((x.start() / 2, x.end() / 2));
                s = x.as_str().into();
                isfind = true;
            }
        }
    }
    return Ok((isfind, locs, s));
}

/**
 * 删除空格 换行
 */
fn fix_blank(text: &str) -> String {
    let re_blank = Regex::new(r"\s").unwrap();
    re_blank.replace_all(&text, "").into()
}

/**
 * 制作共存时 文件名 的 # 替换为 num
 */
fn fix_corexist_file_name(text: &str, num: i32) -> String {
    text.replace("#", &format!("{}", num))
}

/**
 * ? 替换为 .
 */
fn fix_wildcard(text: &str) -> String {
    let re_blank = Regex::new(r"\?").unwrap();
    re_blank.replace_all(&text, ".").into()
}

/**
 * 处理 ...
 */
fn fix_ellipsis(text: &str, text2: &str) -> String {
    if let Some(index) = text.find("...") {
        let l_text = &text[..index];
        let r_text = &text[index + 3..];
        let m_text = &text2[l_text.len()..&text2.len() - &r_text.len()];
        return Cow::from(format!("{}{}{}", l_text, m_text, r_text)).into();
    }
    text.into()
}

/**
 * 处理 patch data 的 ??
 */
fn fix_patch_data(text: &str, text2: &str) -> Result<Vec<u8>, MyError> {
    if text.len() != text2.len() {
        return Err(MyError::FixPatchDataError);
    }
    let mut u1 = text.as_bytes().to_owned();
    let u2 = text2.as_bytes();
    for (i, x) in u1.iter_mut().enumerate() {
        match x {
            63 => *x = u2[i],
            _ => {}
        }
    }
    let mut dst = vec![0; u1.len() / 2];
    hex_decode(&u1, &mut dst).map_err(|_| MyError::FixPatchDataError)?;
    Ok(dst)
}
