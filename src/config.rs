use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::cs2::entity::weapon::Weapon;

pub const DEFAULT_CONFIG_NAME: &str = "cs2-skin-changer.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SkinChangerConfig {
    pub enabled: bool,
    pub skins: HashMap<Weapon, WeaponSkinConfig>,
}

impl Default for SkinChangerConfig {
    fn default() -> Self {
        let mut skins = HashMap::new();
        for weapon in Weapon::iter() {
            if weapon == Weapon::Unknown {
                continue;
            }
            skins.insert(weapon, WeaponSkinConfig::default());
        }
        Self {
            enabled: false,
            skins,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WeaponSkinConfig {
    pub enabled: bool,
    pub paint_kit: i32,
    pub seed: i32,
    pub wear: f32,
    pub stattrak: i32,
}

impl Default for WeaponSkinConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            paint_kit: 0,
            seed: 0,
            wear: 0.0,
            stattrak: -1,
        }
    }
}

pub static BASE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = std::env::var_os("XDG_CONFIG_HOME")
        .and_then(|p| {
            if p.is_empty() {
                None
            } else {
                Some(PathBuf::from(p))
            }
        })
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|base| base.join("cs2-skin-changer"))
        .unwrap_or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
        });
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path
});

pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = BASE_PATH.clone();
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path
});

pub fn parse_config(path: &Path) -> SkinChangerConfig {
    if !path.exists() || path.is_dir() {
        return SkinChangerConfig::default();
    }

    let config_string = read_to_string(path).unwrap();
    let config = toml::from_str(&config_string);
    if config.is_err() {
        log::warn!("config file invalid");
    }
    log::info!("loaded config {:?}", path.file_name().unwrap());
    config.unwrap_or_default()
}

pub fn write_config(config: &SkinChangerConfig, path: &Path) {
    let out = toml::to_string(&config).unwrap();
    std::fs::write(path, out).unwrap();
}
