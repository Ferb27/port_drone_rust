// Ported from droneengage_mavlink@be5d8a5: src/main.cpp
//
// de_ardupilot: DroneEngage flight-controller module (Rust).

// Message constants keep their C++ names (TYPE_AndruavMessage_*), also in match patterns.
#![allow(non_upper_case_globals)]

mod defines;
mod fcb;
mod gps;
mod mav;
mod modes;
mod parser;
mod swarm;
mod vehicle;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use de_common::config::{ConfigFile, LocalConfigFile};
use de_common::json::{validate_field, JsonType};
use de_common::messages::*;
use de_common::module::{
    HARDWARE_TYPE_CPU, MODULE_CLASS_FCB, MODULE_FEATURE_RECEIVING_TELEMETRY,
    MODULE_FEATURE_SENDING_TELEMETRY,
};
use de_common::parser::ModuleParser;
use de_common::{DatabusConfig, Module, ModuleDefinition};
use serde_json::{json, Value};
use tokio::sync::Mutex as AsyncMutex;

/// Version reported in the module ID ("v").
const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "-rs");

#[derive(Parser, Debug)]
#[command(name = "de_ardupilot", about = "DroneEngage FCB module (Rust)")]
struct Args {
    /// configuration file
    #[arg(
        short = 'c',
        long = "config",
        default_value = "de_mavlink.config.module.json"
    )]
    config: String,
    /// local (device specific) configuration file
    #[arg(short = 'b', long = "bconfig", default_value = "de_mavlink.local")]
    bconfig: String,
    /// print the hardware serial number and exit
    #[arg(short = 's', long = "serial")]
    serial: bool,
    /// print the version and exit
    #[arg(short = 'o', long = "versiononly")]
    version_only: bool,
}

/// C++ MESSAGE_FILTER, same order.
fn message_filter() -> Value {
    json!([
        TYPE_AndruavMessage_RemoteExecute,
        TYPE_AndruavMessage_FlightControl,
        TYPE_AndruavMessage_GeoFence,
        TYPE_AndruavMessage_ExternalGeoFence,
        TYPE_AndruavMessage_Arm,
        TYPE_AndruavMessage_ChangeAltitude,
        TYPE_AndruavMessage_Land,
        TYPE_AndruavMessage_GuidedPoint,
        TYPE_AndruavMessage_CirclePoint,
        TYPE_AndruavMessage_DoYAW,
        TYPE_AndruavMessage_DistinationLocation,
        TYPE_AndruavMessage_ChangeSpeed,
        TYPE_AndruavMessage_TrackingTarget_ACTION,
        TYPE_AndruavMessage_TrackingTargetLocation,
        TYPE_AndruavMessage_TrackingTarget_STATUS,
        TYPE_AndruavMessage_UploadWayPoints,
        TYPE_AndruavMessage_RemoteControlSettings,
        TYPE_AndruavMessage_SET_HOME_LOCATION,
        TYPE_AndruavMessage_RemoteControl2,
        TYPE_AndruavMessage_LightTelemetry,
        TYPE_AndruavMessage_ServoChannel,
        TYPE_AndruavMessage_Sync_EventFire,
        TYPE_AndruavMessage_MAVLINK,
        TYPE_AndruavMessage_SWARM_MAVLINK,
        TYPE_AndruavMessage_Make_Swarm,
        TYPE_AndruavMessage_FollowHim_Request,
        TYPE_AndruavMessage_FollowMe_Guided,
        TYPE_AndruavMessage_UpdateSwarm,
        TYPE_AndruavMessage_UDPProxy_Info,
        TYPE_AndruavSystem_UDPProxy,
        TYPE_AndruavMessage_P2P_ACTION,
        TYPE_AndruavMessage_P2P_STATUS,
        TYPE_AndruavMessage_Upload_DE_Mission,
        TYPE_AndruavMessage_DEPilot_Control,
        TYPE_AndruavMessage_PRECLAND_TARGET,
        TYPE_AndruavMessage_PRECLAND_STATUS,
        TYPE_AndruavMessage_CONFIG_ACTION,
        TYPE_AndruavMessage_DUMMY,
    ])
}

/// RPi CPU serial (first 16 chars after "Serial : ") or "NORPI", plus /etc/machine-id.
fn hardware_serial() -> String {
    let mut s = "NORPI".to_string();
    if let Ok(info) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in info.lines() {
            if line.starts_with("Serial") {
                if let Some((_, v)) = line.split_once(':') {
                    s = v.trim_start().chars().take(16).collect();
                }
            }
        }
    }
    if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
        s.push_str(id.lines().next().unwrap_or(""));
    }
    s
}

fn str_config<'a>(cfg: &'a Value, key: &str) -> anyhow::Result<&'a str> {
    cfg.get(key)
        .and_then(Value::as_str)
        .with_context(|| format!("config field {key} missing or not a string"))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let serial = hardware_serial();
    if args.version_only {
        println!("{VERSION}");
        return Ok(());
    }
    if args.serial {
        println!("Serial Number: {serial}");
        return Ok(());
    }
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    tracing::info!(version = VERSION, "Drone-Engage FCB Module (Rust)");

    let mut config = ConfigFile::load(&args.config)?;
    let mut local = LocalConfigFile::load(&args.bconfig)?;
    config.apply_local_overrides(&local.json().clone());
    let cfg = config.json().clone();

    let mut module_key = local.get_string("module_key");
    if module_key.is_empty() {
        module_key = de_common::helpers::get_time_usec().to_string();
        local.set_string("module_key", &module_key);
        local.apply()?;
    }

    let module = Module::new(ModuleDefinition {
        class: MODULE_CLASS_FCB.into(),
        id: str_config(&cfg, "module_id")?.trim().to_string(),
        key: module_key,
        version: VERSION.into(),
        message_filter: message_filter(),
    });
    module.add_module_feature(MODULE_FEATURE_SENDING_TELEMETRY);
    module.add_module_feature(MODULE_FEATURE_RECEIVING_TELEMETRY);
    module.set_hardware(&serial, HARDWARE_TYPE_CPU);
    module.configure_memory_status(300.0, 15.0);

    let link = mav::FcLink::new(mav::address_from_config(&cfg)?);
    let fcb = Arc::new(AsyncMutex::new(fcb::Fcb::new(
        module.clone(),
        link.clone(),
        cfg.clone(),
    )));
    let config = Arc::new(Mutex::new(config));

    // FC frames in
    {
        let fcb = fcb.clone();
        let link = link.clone();
        tokio::spawn(async move {
            loop {
                let (h, m) = link.recv().await;
                fcb.lock().await.on_fc_frame(h, m).await;
            }
        });
    }
    // 10 ms scheduler
    {
        let fcb = fcb.clone();
        tokio::spawn(async move {
            let mut t = tokio::time::interval(Duration::from_millis(10));
            t.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                t.tick().await;
                fcb.lock().await.tick().await;
            }
        });
    }

    // databus last (C++: "should be last")
    let chunk_size = if validate_field(&cfg, "s2s_udp_packet_size", JsonType::String) {
        str_config(&cfg, "s2s_udp_packet_size")?
            .parse()
            .unwrap_or(0)
    } else {
        tracing::warn!("s2s_udp_packet_size missing, auto-detect");
        0
    };
    let databus = DatabusConfig {
        target_ip: str_config(&cfg, "s2s_udp_target_ip")?.to_string(),
        target_port: str_config(&cfg, "s2s_udp_target_port")?.parse()?,
        host: str_config(&cfg, "s2s_udp_listening_ip")?.to_string(),
        listen_port: str_config(&cfg, "s2s_udp_listening_port")?.parse()?,
        chunk_size,
        use_unix_socket: cfg
            .get("use_unix_socket")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        unix_socket_dir: cfg
            .get("unix_socket_dir")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
    };
    let mut rx = module.init(&databus, &cfg).await?;
    let mut parser = parser::FcbParser {
        module: module.clone(),
        config,
        fcb: fcb.clone(),
    };

    loop {
        tokio::select! {
            msg = rx.recv() => {
                let Some(msg) = msg else { break };
                let j = &msg.json;
                let is_uv = j.get(INTERMODULE_ROUTING_TYPE).and_then(Value::as_str) == Some(CMD_TYPE_INTERMODULE);
                let mt = j.get(ANDRUAV_PROTOCOL_MESSAGE_TYPE).and_then(de_common::json::as_cpp_int);
                if is_uv && mt == Some(TYPE_AndruavModule_ID) {
                    let cmd = j.get(ANDRUAV_PROTOCOL_MESSAGE_CMD).cloned().unwrap_or(Value::Null);
                    fcb.lock().await.on_module_id(&cmd).await;
                    continue;
                }
                parser.parse_message(&msg).await;
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("TERMINATING AT USER REQUEST");
                break;
            }
        }
    }
    module.shutdown();
    Ok(())
}
