use std::io::Write;
use std::thread;
use std::time::Duration;

mod config;
mod constants;
mod cs2;
mod os;

use config::{parse_config, write_config, CONFIG_PATH, DEFAULT_CONFIG_NAME};
use cs2::CS2;

#[cfg(not(target_os = "linux"))]
compile_error!("only linux is supported.");

fn main() {
    let env = env_logger::Env::new();
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .filter_level(log::LevelFilter::Off)
        .filter_module("cs2_skin_changer", log::LevelFilter::Info)
        .parse_env(env)
        .init();

    log::info!("CS2 Skin Changer v1.0.0");

    if let Ok(username) = std::env::var("USER")
        && username == "root"
    {
        log::error!("start without sudo, and add your user to the input group.");
        return;
    }

    // Load config
    let config_path = CONFIG_PATH.join(DEFAULT_CONFIG_NAME);
    let mut config = parse_config(&config_path);

    log::info!("Skin changer enabled: {}", config.enabled);
    log::info!("Config path: {:?}", config_path);

    // Print configured skins
    let mut configured_skins = 0;
    for (weapon, skin_config) in &config.skins {
        if skin_config.enabled && skin_config.paint_kit > 0 {
            log::info!(
                "  {:?}: PaintKit={}, Seed={}, Wear={:.4}, StatTrak={}",
                weapon,
                skin_config.paint_kit,
                skin_config.seed,
                skin_config.wear,
                skin_config.stattrak
            );
            configured_skins += 1;
        }
    }

    if configured_skins == 0 {
        log::warn!("No skins configured! Edit the config file to add skins.");
        log::info!("Example: Set enabled=true and paint_kit to a valid skin ID for any weapon.");
        
        // Enable skin changer by default for demonstration
        config.enabled = true;
        write_config(&config, &config_path);
        log::info!("Enabled skin changer in config. Edit the config file to add specific skins.");
    }

    // Main loop
    let mut cs2 = CS2::new();
    let loop_duration = Duration::from_millis(10); // 100Hz update rate

    log::info!("Starting main loop...");
    log::info!("Press Ctrl+C to exit");

    loop {
        if !cs2.is_valid() {
            log::info!("Waiting for CS2 process...");
            cs2.setup();
            if !cs2.is_valid() {
                thread::sleep(Duration::from_secs(5));
                continue;
            }
            log::info!("CS2 found!");
        }

        cs2.run(&config);
        thread::sleep(loop_duration);
    }
}
