use crate::cs2::offsets::Offsets;
use serde::Deserialize;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct DumperOutput {
    #[serde(rename = "client.dll")]
    client: Option<HashMap<String, u64>>,
    #[serde(rename = "engine2.dll")]
    engine: Option<HashMap<String, u64>>,
    #[serde(rename = "libclient.so")]
    libclient: Option<HashMap<String, u64>>,
    #[serde(rename = "libengine2.so")]
    libengine: Option<HashMap<String, u64>>,
}

pub fn update_offsets_from_dumper(offsets: &mut Offsets) -> bool {
    let urls = [
        "https://raw.githubusercontent.com/sezzyaep/CS2-OFFSETS/main/offsets.json",
        "https://raw.githubusercontent.com/hxuanyu/cs2-dumper/main/output/linux/offsets.json",
        "https://raw.githubusercontent.com/a2x/cs2-dumper/main/output/linux/offsets.json",
    ];

    let mut content = String::new();
    let mut found = false;

    // First check local file "offsets.json" in current directory
    if let Ok(c) = std::fs::read_to_string("offsets.json") {
        log::info!("found local offsets.json");
        content = c;
        found = true;
    } else {
        for url in urls {
            log::info!("fetching offsets from {}", url);
            match ureq::get(url).call() {
                Ok(response) => {
                    if let Ok(c) = response.into_body().read_to_string() {
                        content = c;
                        found = true;
                        break;
                    } else {
                        log::warn!("failed to read response body from {}", url);
                    }
                },
                Err(e) => {
                    log::warn!("failed to fetch offsets from {}: {}", url, e);
                }
            }
        }
    }

    if !found {
        return false;
    }

    let dumper_output: DumperOutput = match serde_json::from_str(&content) {
        Ok(o) => o,
        Err(e) => {
            log::warn!("failed to parse offsets.json: {}", e);
            return false;
        }
    };

    // Try to get values from linux keys first, then windows keys as fallback
    let engine_map = dumper_output.libengine.or(dumper_output.engine);
    
    if let Some(map) = engine_map {
        if let Some(val) = map.get("dwNetworkGameClient") {
             offsets.direct.network_client = offsets.library.engine + *val;
             log::info!("updated dwNetworkGameClient from dumper: 0x{:X}", val);
        }
        
        if let Some(val) = map.get("dwNetworkGameClient_deltaTick") {
            offsets.network_client.delta_tick = *val;
            log::info!("updated delta_tick from dumper: 0x{:X}", val);
        } else if let Some(val) = map.get("dwNetworkGameClient_clientTickCount") {
             log::info!("found clientTickCount: 0x{:X} (not using as delta_tick yet)", val);
        }
    }

    true
}
