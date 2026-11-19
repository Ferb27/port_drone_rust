// Ported from droneengage_common@5fb11c2: de_databus/de_module.cpp, de_module.hpp,
// udpClient.cpp (InternelSenderIDEntry), unixDgramClient.cpp (InternelSenderIDEntry)
//
// C++ `CModule` is a singleton with a receive thread, an ID-sender thread and
// a health thread. Here `Module` is a cheap clonable handle; `init` spawns the
// same three loops as tokio tasks and hands received messages out through an
// mpsc channel instead of a function-pointer callback.
//
// Not ported yet: Phase-3 capability adverts (setCapabilities, onInvoke,
// publishState, fireEvent). No C++ module we port uses them. Capability
// requests from de_comm are still answered the way a C++ module without
// adverts answers them.

use std::collections::BTreeMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Map, Value};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::chunk::{DEFAULT_UDP_DATABUS_PACKET_SIZE, SAFE_REMOTE_UDP_PAYLOAD};
use crate::cpu::{self, CpuBusyState, CpuMonitor, CpuMonitorConfig, CpuTransition, ProcCpuTimes};
use crate::health::HealthMonitor;
use crate::helpers::{is_localhost, unix_socket_base_dir, unix_time_sec};
use crate::json::dump;
use crate::messages::*;
use crate::transport::Transport;

pub const MODULE_FEATURE_RECEIVING_TELEMETRY: &str = "R";
pub const MODULE_FEATURE_SENDING_TELEMETRY: &str = "T";
pub const MODULE_FEATURE_CAPTURE_IMAGE: &str = "C";
pub const MODULE_FEATURE_CAPTURE_VIDEO: &str = "V";
pub const MODULE_FEATURE_GPIO: &str = "G";
pub const MODULE_FEATURE_AI_RECOGNITION: &str = "A";
pub const MODULE_FEATURE_TRACKING: &str = "K";
pub const MODULE_FEATURE_P2P: &str = "P";
pub const MODULE_FEATURE_PRECISION_LANDING: &str = "L";

pub const MODULE_CLASS_COMM: &str = "comm";
pub const MODULE_CLASS_FCB: &str = "fcb";
pub const MODULE_CLASS_VIDEO: &str = "camera";
pub const MODULE_CLASS_P2P: &str = "p2p";
pub const MODULE_CLASS_GENERIC: &str = "gen";
pub const MODULE_CLASS_GPIO: &str = "gpio";
pub const MODULE_CLASS_A_RECOGNITION: &str = "ai_rec";
pub const MODULE_CLASS_TRACKING: &str = "trk";
pub const MODULE_CLASS_VIEWLINK: &str = "vlk";
pub const MODULE_CLASS_IR_MI48_CAMERA: &str = "ir";
pub const MODULE_CLASS_TELNET: &str = "tel";
pub const MODULE_CLASS_PRECISION_LANDING: &str = "pl";

pub const HARDWARE_TYPE_UNDEFINED: i32 = 0;
pub const HARDWARE_TYPE_CPU: i32 = 1;

const MODULE_HEALTH_INTERVAL_SEC: u32 = 30;
const INCOMING_QUEUE: usize = 256;

/// What `defineModule` sets.
#[derive(Debug, Clone)]
pub struct ModuleDefinition {
    pub class: String,
    pub id: String,
    pub key: String,
    pub version: String,
    /// Array of message type numbers this module wants to receive.
    pub message_filter: Value,
}

/// Arguments of C++ `CModule::init`.
#[derive(Debug, Clone)]
pub struct DatabusConfig {
    /// de_comm address.
    pub target_ip: String,
    pub target_port: u16,
    /// Module listening address.
    pub host: String,
    pub listen_port: u16,
    /// <= 0: auto (8192 on loopback, 1472 otherwise).
    pub chunk_size: i32,
    pub use_unix_socket: bool,
    pub unix_socket_dir: String,
}

/// One message received from de_comm.
#[derive(Debug, Clone)]
pub struct Incoming {
    /// Full reassembled buffer, including the trailing NUL the transport adds
    /// (binary messages: `json \0 payload \0`).
    pub raw: Vec<u8>,
    /// The JSON header (text up to the first NUL).
    pub json: Value,
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleError {
    #[error("module already initialised")]
    AlreadyInitialised,
    #[error("cannot resolve address {0}")]
    Resolve(String),
    #[error("databus I/O: {0}")]
    Io(#[from] std::io::Error),
}

/// Handle to the module's databus connection. Clone freely.
#[derive(Clone)]
pub struct Module {
    inner: Arc<Inner>,
}

struct Inner {
    def: Mutex<ModuleDefinition>,
    features: Mutex<Vec<String>>,
    hardware: Mutex<(String, i32)>,
    extra_fields: Mutex<BTreeMap<String, Value>>,
    instance_time_stamp: i64,
    party: Mutex<(String, String)>, // (party_id, group_id)
    first_id_received: AtomicBool,
    json_id: Mutex<String>,
    transport: Mutex<Option<Arc<Transport>>>,
    tasks: Mutex<Vec<JoinHandle<()>>>,
    /// Serialises sendJMSG/sendBMSG/sendMREMSG like C++ `m_lock`.
    send_lock: tokio::sync::Mutex<()>,
    pub(crate) health: Mutex<HealthMonitor>,
    event_counter: AtomicU64,
}

impl Module {
    pub fn new(def: ModuleDefinition) -> Self {
        Self {
            inner: Arc::new(Inner {
                def: Mutex::new(def),
                features: Mutex::new(Vec::new()),
                hardware: Mutex::new((String::new(), HARDWARE_TYPE_UNDEFINED)),
                extra_fields: Mutex::new(BTreeMap::new()),
                instance_time_stamp: unix_time_sec(),
                party: Mutex::new((String::new(), String::new())),
                first_id_received: AtomicBool::new(false),
                json_id: Mutex::new(String::new()),
                transport: Mutex::new(None),
                tasks: Mutex::new(Vec::new()),
                send_lock: tokio::sync::Mutex::new(()),
                health: Mutex::new(HealthMonitor::default()),
                event_counter: AtomicU64::new(0),
            }),
        }
    }

    // ---- definition / identity -------------------------------------------------

    pub fn add_module_feature(&self, feature: &str) {
        lock(&self.inner.features).push(feature.to_string());
    }

    pub fn set_hardware(&self, serial: &str, hardware_type: i32) {
        *lock(&self.inner.hardware) = (serial.to_string(), hardware_type);
    }

    pub fn set_module_class(&self, class: &str) {
        lock(&self.inner.def).class = class.to_string();
    }

    pub fn set_module_id(&self, id: &str) {
        lock(&self.inner.def).id = id.to_string();
    }

    pub fn set_module_key(&self, key: &str) {
        lock(&self.inner.def).key = key.to_string();
    }

    pub fn module_key(&self) -> String {
        lock(&self.inner.def).key.clone()
    }

    pub fn module_id(&self) -> String {
        lock(&self.inner.def).id.clone()
    }

    /// Party ID assigned by de_comm (empty until the first ID reply).
    pub fn party_id(&self) -> String {
        lock(&self.inner.party).0.clone()
    }

    pub fn group_id(&self) -> String {
        lock(&self.inner.party).1.clone()
    }

    pub(crate) fn health_monitor(&self) -> std::sync::MutexGuard<'_, HealthMonitor> {
        lock(&self.inner.health)
    }

    /// Extra key in the ID message's "ms" (C++ appendExtraField).
    pub fn append_extra_field(&self, name: &str, value: Value) {
        lock(&self.inner.extra_fields).insert(name.to_string(), value);
    }

    /// Build (and cache for the 1 Hz sender) the module ID message.
    /// `resend = true` asks de_comm to answer with its own ID.
    pub fn create_json_id(&self, resend: bool) -> String {
        let def = lock(&self.inner.def).clone();
        let (serial, hw_type) = lock(&self.inner.hardware).clone();
        let mut ms = Map::new();
        ms.insert(JSON_INTERMODULE_MODULE_ID.into(), def.id.into());
        ms.insert(JSON_INTERMODULE_MODULE_CLASS.into(), def.class.into());
        ms.insert(
            JSON_INTERMODULE_MODULE_MESSAGES_LIST.into(),
            def.message_filter,
        );
        ms.insert(
            JSON_INTERMODULE_MODULE_FEATURES.into(),
            Value::from(lock(&self.inner.features).clone()),
        );
        ms.insert(JSON_INTERMODULE_MODULE_KEY.into(), def.key.into());
        ms.insert(JSON_INTERMODULE_HARDWARE_ID.into(), serial.into());
        ms.insert(JSON_INTERMODULE_HARDWARE_TYPE.into(), hw_type.into());
        ms.insert(JSON_INTERMODULE_VERSION.into(), def.version.into());
        ms.insert(JSON_INTERMODULE_RESEND.into(), resend.into());
        ms.insert(
            JSON_INTERMODULE_TIMESTAMP_INSTANCE.into(),
            self.inner.instance_time_stamp.into(),
        );
        for (k, v) in lock(&self.inner.extra_fields).iter() {
            ms.insert(k.clone(), v.clone());
        }
        let msg = json!({
            INTERMODULE_ROUTING_TYPE: CMD_TYPE_INTERMODULE,
            ANDRUAV_PROTOCOL_MESSAGE_TYPE: TYPE_AndruavModule_ID,
            ANDRUAV_PROTOCOL_MESSAGE_CMD: Value::Object(ms),
        });
        let text = dump(&msg);
        *lock(&self.inner.json_id) = text.clone();
        text
    }

    // ---- lifecycle -------------------------------------------------------------

    /// Open the databus and start the receive, ID and health loops.
    /// `module_config` is the module's JSON config (for "cpu_monitor").
    pub async fn init(
        &self,
        cfg: &DatabusConfig,
        module_config: &Value,
    ) -> Result<mpsc::Receiver<Incoming>, ModuleError> {
        if lock(&self.inner.transport).is_some() {
            return Err(ModuleError::AlreadyInitialised);
        }
        let chunk_size = if cfg.chunk_size > 0 {
            cfg.chunk_size as usize
        } else if is_localhost(&cfg.target_ip) {
            DEFAULT_UDP_DATABUS_PACKET_SIZE
        } else {
            SAFE_REMOTE_UDP_PAYLOAD
        };

        let mut transport = None;
        if cfg.use_unix_socket && is_localhost(&cfg.target_ip) {
            transport = self.try_unix(cfg, chunk_size);
        }
        let transport = match transport {
            Some(t) => t,
            None => {
                let target = resolve(&cfg.target_ip, cfg.target_port)?;
                let bind = resolve(&cfg.host, cfg.listen_port)?;
                Transport::bind_udp(target, bind, chunk_size).await?
            }
        };
        let transport = Arc::new(transport);
        *lock(&self.inner.transport) = Some(transport.clone());
        self.create_json_id(true);

        let (tx, rx) = mpsc::channel(INCOMING_QUEUE);
        let tasks = [
            tokio::spawn(receive_loop(self.clone(), transport.clone(), tx)),
            tokio::spawn(id_sender_loop(self.clone(), transport)),
            tokio::spawn(health_loop(
                self.clone(),
                cpu::parse_cpu_monitor_config(module_config),
            )),
        ];
        lock(&self.inner.tasks).extend(tasks);
        Ok(rx)
    }

    fn try_unix(&self, cfg: &DatabusConfig, chunk_size: usize) -> Option<Transport> {
        let base = unix_socket_base_dir(&cfg.unix_socket_dir);
        let Some(base) = base else {
            tracing::error!("unix socket dir rejected; module bus falls back to UDP");
            return None;
        };
        let def = lock(&self.inner.def).clone();
        let broker = PathBuf::from(format!("{base}/de_comm_broker.sock"));
        let own = PathBuf::from(format!("{base}/de_comm_{}_{}.sock", def.id, def.key));
        // sun_path is 108 bytes including the NUL.
        if broker.as_os_str().len() >= 108 || own.as_os_str().len() >= 108 {
            tracing::error!(path = %own.display(), "unix socket path too long; falling back to UDP");
            return None;
        }
        match Transport::bind_unix(&broker, &own, chunk_size) {
            Ok(t) => Some(t),
            Err(e) => {
                tracing::error!(error = %e, "unix socket bind failed; falling back to UDP");
                None
            }
        }
    }

    /// Stop all loops and the transport.
    pub fn shutdown(&self) {
        if let Some(t) = lock(&self.inner.transport).as_ref() {
            t.stop();
        }
        for h in lock(&self.inner.tasks).drain(..) {
            h.abort();
        }
    }

    // ---- sending ---------------------------------------------------------------

    fn transport(&self) -> Option<Arc<Transport>> {
        lock(&self.inner.transport).clone()
    }

    /// Send raw bytes as one databus message.
    pub async fn send_raw(&self, msg: &[u8]) {
        if let Some(t) = self.transport() {
            t.send(msg).await;
        }
    }

    /// Forward a message received on another channel unchanged (C++ forwardMSG).
    pub async fn forward_msg(&self, msg: &[u8]) {
        self.send_raw(msg).await;
    }

    /// Routing type for a message the way C++ decides it.
    fn routing(target: &str, internal: bool) -> &'static str {
        if internal {
            CMD_TYPE_INTERMODULE
        } else if !target.is_empty() {
            CMD_COMM_INDIVIDUAL
        } else {
            CMD_COMM_GROUP
        }
    }

    /// Build the JSON envelope of sendJMSG/sendBMSG.
    pub fn envelope(&self, target: &str, msg_cmd: Value, message_id: i32, internal: bool) -> Value {
        json!({
            INTERMODULE_MODULE_KEY: self.module_key(),
            ANDRUAV_PROTOCOL_TARGET_ID: target,
            INTERMODULE_ROUTING_TYPE: Self::routing(target, internal),
            ANDRUAV_PROTOCOL_MESSAGE_TYPE: message_id,
            ANDRUAV_PROTOCOL_MESSAGE_CMD: msg_cmd,
        })
    }

    /// JSON message. `internal = true` keeps it inside the unit (de_comm and other modules).
    pub async fn send_jmsg(&self, target: &str, jmsg: Value, message_id: i32, internal: bool) {
        let _g = self.inner.send_lock.lock().await;
        let text = dump(&self.envelope(target, jmsg, message_id, internal));
        self.send_raw(text.as_bytes()).await;
    }

    /// Binary message: JSON header, NUL, then `payload`. `message_cmd` goes in
    /// "ms" (pass `Value::Null` for none, which serialises as `null` like C++).
    pub async fn send_bmsg(
        &self,
        target: &str,
        payload: &[u8],
        message_id: i32,
        internal: bool,
        message_cmd: Value,
    ) {
        let _g = self.inner.send_lock.lock().await;
        let mut buf = dump(&self.envelope(target, message_cmd, message_id, internal)).into_bytes();
        buf.push(0);
        buf.extend_from_slice(payload);
        self.send_raw(&buf).await;
    }

    /// System message to the comm server (`ty: "s"`, `tg: "_SYS_"`).
    pub async fn send_sysmsg(&self, jmsg: Value, message_id: i32) {
        let msg = json!({
            ANDRUAV_PROTOCOL_TARGET_ID: ANDRUAV_PROTOCOL_SENDER_COMM_SERVER,
            INTERMODULE_ROUTING_TYPE: CMD_COMM_SYSTEM,
            ANDRUAV_PROTOCOL_MESSAGE_TYPE: message_id,
            ANDRUAV_PROTOCOL_MESSAGE_CMD: jmsg,
        });
        self.send_raw(dump(&msg).as_bytes()).await;
    }

    /// Module-to-module remote execute (TYPE_AndruavModule_RemoteExecute, `{"C": command}`).
    pub async fn send_mremsg(&self, command_type: i32) {
        let _g = self.inner.send_lock.lock().await;
        let msg = json!({
            INTERMODULE_MODULE_KEY: self.module_key(),
            INTERMODULE_ROUTING_TYPE: CMD_TYPE_INTERMODULE,
            ANDRUAV_PROTOCOL_MESSAGE_TYPE: TYPE_AndruavModule_RemoteExecute,
            ANDRUAV_PROTOCOL_MESSAGE_CMD: {"C": command_type},
        });
        self.send_raw(dump(&msg).as_bytes()).await;
    }

    /// Fresh nonce in the C++ fireEvent format: "<epoch ms>-<counter>".
    pub fn next_event_nonce(&self) -> String {
        let n = self.inner.event_counter.fetch_add(1, Ordering::SeqCst) + 1;
        format!("{}-{}", crate::helpers::get_time_usec() / 1000, n)
    }

    // ---- receiving ---------------------------------------------------------------

    /// C++ `CModule::onReceive`. Returns the message to hand to the
    /// application, or `None` when it was consumed or invalid.
    pub(crate) async fn on_receive(&self, raw: Vec<u8>) -> Option<Incoming> {
        let json = match crate::json::parse_c_string(&raw) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "databus: bad JSON");
                return None;
            }
        };
        json.get(ANDRUAV_PROTOCOL_MESSAGE_TYPE)?;
        // C++ get<std::string>() throws on a non-string -> message dropped.
        let routing = json.get(INTERMODULE_ROUTING_TYPE)?.as_str()?.to_string();

        if routing == CMD_TYPE_INTERMODULE {
            let cmd = json.get(ANDRUAV_PROTOCOL_MESSAGE_CMD)?.clone();
            let message_type = crate::json::as_cpp_int(json.get(ANDRUAV_PROTOCOL_MESSAGE_TYPE)?)?;
            match message_type {
                TYPE_AndruavModule_ID => {
                    let rec = cmd.get(JSON_INTERMODULE_PARTY_RECORD)?;
                    let party = rec.get(ANDRUAV_PROTOCOL_SENDER)?.as_str()?.to_string();
                    let group = rec.get(ANDRUAV_PROTOCOL_GROUP_ID)?.as_str()?.to_string();
                    *lock(&self.inner.party) = (party.clone(), group.clone());
                    if !self.inner.first_id_received.swap(true, Ordering::SeqCst) {
                        tracing::info!(%party, %group, "communicator server found");
                        self.create_json_id(false);
                    }
                    return Some(Incoming { raw, json });
                }
                TYPE_AndruavMessage_MODULE_CAPABILITIES => {
                    // A module with no adverts still answers {"caps":[],"ch":""}.
                    if cmd.get("r").and_then(Value::as_bool).unwrap_or(false) {
                        self.send_jmsg(
                            "",
                            json!({"caps": [], "ch": ""}),
                            TYPE_AndruavMessage_MODULE_CAPABILITIES,
                            true,
                        )
                        .await;
                    }
                    return None;
                }
                TYPE_AndruavMessage_CAPABILITY_INVOKE => {
                    let id = cmd.get("id").and_then(Value::as_str).unwrap_or("");
                    if !id.is_empty() {
                        let ns = cmd.get("ns").and_then(Value::as_str).unwrap_or("");
                        let act = cmd.get("act").and_then(Value::as_str).unwrap_or("");
                        let result = json!({"id": id, "ok": false, "err": format!("unknown action {ns}.{act}")});
                        self.send_jmsg("", result, TYPE_AndruavMessage_CAPABILITY_RESULT, true)
                            .await;
                    }
                    return None;
                }
                TYPE_AndruavMessage_DUMMY => {
                    tracing::info!("TYPE_AndruavMessage_DUMMY received");
                }
                _ => {}
            }
        }
        Some(Incoming { raw, json })
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    // A panic while holding one of these short locks must not take the
    // whole databus down: recover the data.
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn resolve(host: &str, port: u16) -> Result<SocketAddr, ModuleError> {
    (host, port)
        .to_socket_addrs()
        .ok()
        .and_then(|mut it| it.find(SocketAddr::is_ipv4))
        .ok_or_else(|| ModuleError::Resolve(format!("{host}:{port}")))
}

async fn receive_loop(module: Module, transport: Arc<Transport>, tx: mpsc::Sender<Incoming>) {
    loop {
        match transport.recv().await {
            Ok(raw) => {
                if let Some(msg) = module.on_receive(raw).await {
                    if tx.send(msg).await.is_err() {
                        return; // application dropped the receiver
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "databus: recv failed");
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

async fn id_sender_loop(module: Module, transport: Arc<Transport>) {
    let mut tick = tokio::time::interval(Duration::from_secs(1));
    loop {
        tick.tick().await;
        let id = lock(&module.inner.json_id).clone();
        if !id.is_empty() {
            transport.send(id.as_bytes()).await;
        }
    }
}

/// 1 Hz: board CPU sample (+ 6547 report on edge / every report_s);
/// every 30 s: memory health (6535).
async fn health_loop(module: Module, cpu_cfg: CpuMonitorConfig) {
    let mut cpu_mon = CpuMonitor::new(cpu_cfg);
    let board = cpu::board_id();
    let start = Instant::now();
    let mut prev_proc: Option<(ProcCpuTimes, u64)> = None;
    let mut last_report_ms: u64 = 0;
    let mut tick: u32 = 0;
    let mut interval = tokio::time::interval_at(
        tokio::time::Instant::now() + Duration::from_secs(1),
        Duration::from_secs(1),
    );
    loop {
        interval.tick().await;
        if cpu_cfg.enabled {
            let now_ms = start.elapsed().as_millis() as u64 + 1; // never 0 (0 means "never reported")
            if let Some(times) = cpu::read_proc_stat_cpu() {
                let transition = cpu_mon.update_from_times(times, now_ms);
                let usage = cpu_mon.last_usage_percent();
                if usage >= 0.0 {
                    let mut process_usage = 0;
                    if let Some(p) = cpu::read_proc_self_stat_cpu() {
                        if let Some((pp, pms)) = prev_proc {
                            if now_ms > pms {
                                // DIVERGENCE: USER_HZ assumed 100 (C++ asks sysconf, which is 100 on Linux).
                                let pct =
                                    cpu::process_cpu_usage_percent(&pp, &p, now_ms - pms, 100);
                                if pct >= 0.0 {
                                    process_usage = (pct + 0.5) as i32;
                                }
                            }
                        }
                        prev_proc = Some((p, now_ms));
                    }
                    let event = match transition {
                        CpuTransition::BecameSuperBusy => Some(CPU_REPORT_EVENT_BECAME_SUPER_BUSY),
                        CpuTransition::BecameFree => Some(CPU_REPORT_EVENT_BECAME_FREE),
                        CpuTransition::None
                            if last_report_ms == 0
                                || now_ms.saturating_sub(last_report_ms)
                                    >= cpu_cfg.report_s as u64 * 1000 =>
                        {
                            Some(CPU_REPORT_EVENT_PERIODIC)
                        }
                        CpuTransition::None => None,
                    };
                    if let Some(ev) = event {
                        last_report_ms = now_ms;
                        let state = if cpu_mon.state() == CpuBusyState::SuperBusy {
                            CPU_BOARD_STATE_SUPER_BUSY
                        } else {
                            CPU_BOARD_STATE_FREE
                        };
                        module
                            .send_cpu_status(&board, (usage + 0.5) as i32, process_usage, state, ev)
                            .await;
                    }
                }
            }
        }
        tick += 1;
        if tick >= MODULE_HEALTH_INTERVAL_SEC {
            tick = 0;
            module.send_memory_status("").await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fcb() -> Module {
        let m = Module::new(ModuleDefinition {
            class: MODULE_CLASS_FCB.into(),
            id: "FCB_1".into(),
            key: "1795085454675320".into(),
            version: "7.4.5.1".into(),
            message_filter: json!([1005, 1010]),
        });
        m.add_module_feature(MODULE_FEATURE_SENDING_TELEMETRY);
        m.add_module_feature(MODULE_FEATURE_RECEIVING_TELEMETRY);
        m.set_hardware("NORPI", HARDWARE_TYPE_CPU);
        m
    }

    #[test]
    fn id_message_matches_cpp_layout() {
        let m = fcb();
        let ts = m.inner.instance_time_stamp;
        assert_eq!(
            m.create_json_id(true),
            format!(
                r#"{{"ms":{{"a":"FCB_1","b":"fcb","c":[1005,1010],"d":["T","R"],"e":"1795085454675320","s":"NORPI","t":1,"u":{ts},"v":"7.4.5.1","z":true}},"mt":9100,"ty":"uv"}}"#
            )
        );
    }

    #[test]
    fn envelope_routing_and_key_order() {
        let m = fcb();
        assert_eq!(
            dump(&m.envelope("", json!({"d":"0"}), 1061, false)),
            r#"{"GU":"1795085454675320","ms":{"d":"0"},"mt":1061,"tg":"","ty":"g"}"#
        );
        assert_eq!(
            dump(&m.envelope("_AGN_", json!({"d":"0"}), 1061, false)),
            r#"{"GU":"1795085454675320","ms":{"d":"0"},"mt":1061,"tg":"_AGN_","ty":"i"}"#
        );
        assert_eq!(
            dump(&m.envelope("16657964", Value::Null, 6503, true)),
            r#"{"GU":"1795085454675320","ms":null,"mt":6503,"tg":"16657964","ty":"uv"}"#
        );
    }

    #[tokio::test]
    async fn id_reply_sets_party_and_stops_resend_flag() {
        let m = fcb();
        m.create_json_id(true);
        let reply = br#"{"ms":{"a":"COMM_1","b":"comm","c":"","d":null,"e":"3303982186","f":{"gr":"1","sd":"15647541"},"g":6,"u":1795085442,"z":false},"mt":9100,"ty":"uv"}"#;
        let mut raw = reply.to_vec();
        raw.push(0);
        let got = m.on_receive(raw).await.expect("ID is forwarded to the app");
        assert_eq!(got.json["mt"], 9100);
        assert_eq!(m.party_id(), "15647541");
        assert_eq!(m.group_id(), "1");
        assert!(lock(&m.inner.json_id).contains(r#""z":false"#));
    }

    #[tokio::test]
    async fn invalid_messages_dropped() {
        let m = fcb();
        assert!(m.on_receive(b"not json\0".to_vec()).await.is_none());
        assert!(m.on_receive(br#"{"ty":"i"}"#.to_vec()).await.is_none());
        assert!(m.on_receive(br#"{"mt":1,"ty":5}"#.to_vec()).await.is_none());
        let ok = m
            .on_receive(br#"{"mt":1005,"ty":"i","ms":{"C":1004}}"#.to_vec())
            .await;
        assert!(ok.is_some());
    }
}
