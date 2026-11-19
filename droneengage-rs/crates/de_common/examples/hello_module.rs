//! Minimal module for live interop tests with a real de_comm.
//!
//! usage: hello_module <comm_ip> <comm_port> <listen_port> [module_id]
//!
//! Registers as a generic module subscribed to RemoteExecute (1005) and
//! DUMMY (9999), prints everything it receives, and every 5 s sends an
//! Error (1008) message to the group so it shows up at the comm server.

use de_common::messages::*;
use de_common::module::{MODULE_CLASS_GENERIC, MODULE_FEATURE_SENDING_TELEMETRY};
use de_common::{DatabusConfig, Module, ModuleDefinition};
use serde_json::json;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: hello_module <comm_ip> <comm_port> <listen_port> [module_id]");
        std::process::exit(2);
    }
    let id = args.get(4).cloned().unwrap_or_else(|| "HELLO_RS".into());
    let module = Module::new(ModuleDefinition {
        class: MODULE_CLASS_GENERIC.into(),
        id: id.clone(),
        key: format!("{}", de_common::helpers::get_time_usec()),
        version: de_common::VERSION.into(),
        message_filter: json!([TYPE_AndruavMessage_RemoteExecute, TYPE_AndruavMessage_DUMMY]),
    });
    module.add_module_feature(MODULE_FEATURE_SENDING_TELEMETRY);
    // de_comm asks auth to validate the hardware serial; an empty one is
    // rejected ("Module License INVALID") and the module gets no traffic.
    // "NORPI" is what de_mavlink reports off a Raspberry Pi.
    module.set_hardware("NORPI", de_common::module::HARDWARE_TYPE_CPU);
    let cfg = DatabusConfig {
        target_ip: args[1].clone(),
        target_port: args[2].parse().expect("comm_port"),
        host: "0.0.0.0".into(),
        listen_port: args[3].parse().expect("listen_port"),
        chunk_size: 0,
        use_unix_socket: false,
        unix_socket_dir: String::new(),
    };
    let mut rx = module.init(&cfg, &json!({})).await.expect("databus init");
    println!("hello_module {id} key {} started", module.module_key());

    let sender = module.clone();
    tokio::spawn(async move {
        let mut n = 0;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            n += 1;
            sender
                .send_error_message("", 0, 0, 6, &format!("hello from rust module #{n}"))
                .await;
            println!("TX 1008 #{n} (party {})", sender.party_id());
        }
    });

    while let Some(msg) = rx.recv().await {
        let text = String::from_utf8_lossy(&msg.raw[..msg.raw.len().saturating_sub(1)]);
        println!("RX {}", text.chars().take(300).collect::<String>());
    }
}
