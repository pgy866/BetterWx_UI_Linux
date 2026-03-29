use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::MyError;

use super::config::{CONFIG, HOST, LIBNAME, LOCKINI, REVOKE, UNLOCK};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Patch {
    pub name: String,
    pub loc: Vec<(usize, usize)>,
    pub original: Vec<u8>,
    pub patch: Vec<u8>,
    pub replace_num_loc: usize,
    pub config_item: ConfigItem,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Patchs {
    pub unlock: Option<Patch>,
    pub revoke: Option<Patch>,
    pub config: Option<Patch>,
    pub host: Option<Patch>,
    pub libname: Option<Patch>,
    pub lockini: Option<Patch>,
}

#[derive(Debug)]
pub struct WxPath {
    pub exe_loc: PathBuf,
    pub lib_loc: PathBuf,
    pub exe_file: PathBuf,
    pub lib_file: PathBuf,
}

#[derive(Debug)]
pub struct WxData {
    pub exe_data: Vec<u8>,
    pub lib_data: Vec<u8>,
}

#[derive(Debug)]
pub struct WxInfo {
    pub wx_path: WxPath,
    pub wx_data: WxData,
    pub patchs: Patchs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchStatus {
    pub name: String,
    pub support: bool,
    pub status: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoexistFileInfo {
    pub id: i32,
    pub lib_name: String,
    pub lib_file: PathBuf,
    pub exe_name: String,
    pub exe_file: PathBuf,
    pub patch_status: Vec<PatchStatus>,
}

// ConfigType tuple:
// (version, which_file, pattern, replace, is_force_patch, is_replace_num, is_search)
pub type ConfigType = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    bool,
    bool,
    bool,
);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigItem {
    pub version: String,
    pub which: String,
    pub pattern: String,
    pub replace: String,
    pub is_force_patch: bool,
    pub is_replace_num: bool,
    pub is_search: bool,
}

impl ConfigItem {
    pub fn new(version: &str, config: &[ConfigType]) -> Result<Self, MyError> {
        for item in config {
            if version >= item.0 {
                let r = Self {
                    version: item.0.to_string(),
                    which: item.1.to_string(),
                    pattern: item.2.to_string(),
                    replace: item.3.to_string(),
                    is_force_patch: item.4,
                    is_replace_num: item.5,
                    is_search: item.6,
                };
                return Ok(r);
            }
        }
        Err(MyError::UnsupportedVersion)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchConfig {
    pub unlock: ConfigItem,
    pub revoke: ConfigItem,
    pub config: ConfigItem,
    pub host: ConfigItem,
    pub libname: ConfigItem,
    pub lockini: ConfigItem,
}

impl PatchConfig {
    pub fn new(version: &str) -> Result<Self, MyError> {
        let unlock = ConfigItem::new(version, &UNLOCK)?;
        let revoke = ConfigItem::new(version, &REVOKE)?;
        let config = ConfigItem::new(version, &CONFIG)?;
        let host = ConfigItem::new(version, &HOST)?;
        let libname = ConfigItem::new(version, &LIBNAME)?;
        let lockini = ConfigItem::new(version, &LOCKINI)?;
        Ok(Self {
            unlock,
            revoke,
            config,
            host,
            libname,
            lockini,
        })
    }
}
