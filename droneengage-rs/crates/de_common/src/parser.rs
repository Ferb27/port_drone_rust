// Ported from droneengage_common@5fb11c2: de_databus/de_message_parser_base.cpp,
// de_message_parser_base.hpp
//
// C++ uses an abstract base class with member flags (m_is_binary, ...).
// Here the flags travel in `MessageMeta` and modules implement `ModuleParser`.

use std::sync::{Arc, Mutex};

use serde_json::Value;

use crate::config::ConfigFile;
use crate::json::{as_cpp_int, validate_field, JsonType};
use crate::messages::*;
use crate::module::{Incoming, Module};

/// Facts about an incoming message, computed once by `parse_message`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageMeta {
    pub message_type: i32,
    /// Heuristic from C++: text messages end with '}' (before the added NUL).
    pub is_binary: bool,
    /// Sender is the comm server (`sd == "_SYS_"`).
    pub is_system: bool,
    /// `ty == "uv"`.
    pub is_inter_module: bool,
    /// "p" field, 0 when missing or not an unsigned number.
    pub permission: u32,
}

impl MessageMeta {
    /// None when "mt" is missing or not a number (C++ would throw and drop).
    pub fn of(msg: &Incoming) -> Option<Self> {
        let j = &msg.json;
        let message_type = as_cpp_int(j.get(ANDRUAV_PROTOCOL_MESSAGE_TYPE)?)?;
        let r = &msg.raw;
        let n = r.len();
        let is_binary = !((n >= 1 && r[n - 1] == b'}') || (n >= 2 && r[n - 2] == b'}'));
        let permission = if validate_field(
            j,
            ANDRUAV_PROTOCOL_MESSAGE_PERMISSION,
            JsonType::NumberUnsigned,
        ) {
            as_cpp_int(&j[ANDRUAV_PROTOCOL_MESSAGE_PERMISSION]).unwrap_or(0) as u32
        } else {
            0
        };
        let is_system = j.get(ANDRUAV_PROTOCOL_SENDER).and_then(Value::as_str)
            == Some(ANDRUAV_PROTOCOL_SENDER_COMM_SERVER);
        let is_inter_module =
            j.get(INTERMODULE_ROUTING_TYPE).and_then(Value::as_str) == Some(CMD_TYPE_INTERMODULE);
        Some(Self {
            message_type,
            is_binary,
            is_system,
            is_inter_module,
            permission,
        })
    }
}

/// What a module plugs into the shared dispatch.
#[allow(async_fn_in_trait)]
pub trait ModuleParser {
    fn module(&self) -> &Module;
    fn config(&self) -> &Arc<Mutex<ConfigFile>>;

    /// TYPE_AndruavMessage_RemoteExecute (1005).
    async fn parse_remote_execute(&mut self, msg: &Incoming, meta: &MessageMeta);

    /// Every other message, after the shared default handling.
    async fn parse_command(&mut self, msg: &Incoming, meta: &MessageMeta);

    /// Called after CONFIG_ACTION_APPLY_CONFIG changed the config.
    async fn on_config_updated(&mut self, _cmd: &Value) {}

    /// C++ `CAndruavMessageParserBase::parseMessage`.
    async fn parse_message(&mut self, msg: &Incoming) {
        let Some(meta) = MessageMeta::of(msg) else {
            return;
        };
        if meta.message_type == TYPE_AndruavMessage_RemoteExecute {
            self.parse_remote_execute(msg, &meta).await;
            return;
        }
        if meta.message_type == TYPE_AndruavMessage_CONFIG_ACTION {
            let cmd = msg
                .json
                .get(ANDRUAV_PROTOCOL_MESSAGE_CMD)
                .cloned()
                .unwrap_or(Value::Null);
            self.handle_config_action(msg, &cmd).await;
        }
        self.parse_command(msg, &meta).await;
    }

    /// TYPE_AndruavMessage_CONFIG_ACTION handling shared by all modules.
    async fn handle_config_action(&mut self, msg: &Incoming, cmd: &Value) {
        if !validate_field(cmd, "a", JsonType::NumberUnsigned) {
            return;
        }
        let mut module_key = String::new();
        if validate_field(cmd, "b", JsonType::String) {
            module_key = self.module().module_key();
            if cmd["b"].as_str() != Some(module_key.as_str()) {
                return;
            }
        }
        match as_cpp_int(&cmd["a"]).unwrap_or(-1) {
            CONFIG_ACTION_Restart => {
                tracing::warn!("CONFIG_ACTION_Restart: exiting");
                std::process::exit(0);
            }
            CONFIG_ACTION_APPLY_CONFIG => {
                if !validate_field(cmd, "c", JsonType::Object) {
                    return;
                }
                let text = crate::json::dump(&cmd["c"]);
                self.config()
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .update_json(&text);
                self.on_config_updated(cmd).await;
            }
            CONFIG_REQUEST_FETCH_CONFIG_TEMPLATE => {
                let Some(sender) = msg
                    .json
                    .get(ANDRUAV_PROTOCOL_SENDER)
                    .and_then(Value::as_str)
                else {
                    return;
                };
                let sender = sender.to_string();
                let module = self.module().clone();
                match std::fs::read_to_string("template.json")
                    .ok()
                    .and_then(|t| serde_json::from_str::<Value>(&t).ok())
                {
                    Some(template) => {
                        module
                            .api_send_config_template(&sender, &module_key, template, true)
                            .await;
                    }
                    None => {
                        tracing::error!("cannot read template.json");
                        module
                            .send_error_message(
                                "",
                                0,
                                ERROR_3DR,
                                NOTIFICATION_TYPE_ERROR,
                                "cannot read template.json",
                            )
                            .await;
                        module
                            .api_send_config_template(&sender, &module_key, Value::Null, true)
                            .await;
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn incoming(raw: &[u8]) -> Incoming {
        let json = crate::json::parse_c_string(raw).unwrap();
        Incoming {
            raw: raw.to_vec(),
            json,
        }
    }

    #[test]
    fn meta_text_vs_binary_and_permission() {
        let t = incoming(b"{\"mt\":1005,\"sd\":\"_SYS_\",\"ty\":\"i\",\"p\":4294967295}\0");
        let m = MessageMeta::of(&t).unwrap();
        assert!(!m.is_binary && m.is_system && !m.is_inter_module);
        assert_eq!(m.permission, 0xFFFF_FFFF);

        let b = incoming(b"{\"mt\":6503,\"ty\":\"uv\"}\0\x01\x02\x03\0");
        let m = MessageMeta::of(&b).unwrap();
        assert!(m.is_binary && m.is_inter_module);
        assert_eq!(m.permission, 0);
    }
}
