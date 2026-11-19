// Ported from droneengage_mavlink@be5d8a5: src/fcb_andruav_message_parser.cpp (R2a subset),
// src/swarm/fcb_swarm_manager.cpp (handleMakeSwarm, handleFollowHimRequest, handleUpdateSwarm)

use std::sync::{Arc, Mutex};

use de_common::config::ConfigFile;
use de_common::json::{as_cpp_int, validate_field, JsonType};
use de_common::messages::*;
use de_common::parser::{MessageMeta, ModuleParser};
use de_common::{Incoming, Module};
use serde_json::Value;
use tokio::sync::Mutex as AsyncMutex;

use crate::fcb::Fcb;
use crate::swarm::{self, KNODE_LENGTH};

pub struct FcbParser {
    pub module: Module,
    pub config: Arc<Mutex<ConfigFile>>,
    pub fcb: Arc<AsyncMutex<Fcb>>,
}

/// nlohmann is_number_integer(): signed or unsigned integer.
fn int_field(cmd: &Value, k: &str) -> Option<i32> {
    let v = cmd.get(k)?;
    if v.is_i64() || v.is_u64() {
        as_cpp_int(v)
    } else {
        None
    }
}

fn str_field<'a>(cmd: &'a Value, k: &str) -> Option<&'a str> {
    cmd.get(k).and_then(Value::as_str)
}

fn sender(msg: &Incoming) -> String {
    str_field(&msg.json, ANDRUAV_PROTOCOL_SENDER)
        .unwrap_or_default()
        .to_string()
}

impl ModuleParser for FcbParser {
    fn module(&self) -> &Module {
        &self.module
    }

    fn config(&self) -> &Arc<Mutex<ConfigFile>> {
        &self.config
    }

    async fn parse_remote_execute(&mut self, msg: &Incoming, meta: &MessageMeta) {
        let cmd = msg
            .json
            .get(ANDRUAV_PROTOCOL_MESSAGE_CMD)
            .cloned()
            .unwrap_or(Value::Null);
        if !validate_field(&cmd, "C", JsonType::NumberUnsigned) {
            return;
        }
        let fcb = self.fcb.lock().await;
        match as_cpp_int(&cmd["C"]).unwrap_or(-1) {
            TYPE_AndruavMessage_MAVLINK => {
                if int_field(&cmd, "Act") == Some(0) {
                    fcb.send_heartbeat(&sender(msg)).await;
                }
            }
            TYPE_AndruavMessage_HomeLocation => {
                if !fcb.info.is_gcs_blocked {
                    fcb.send_home_location(&sender(msg)).await;
                }
            }
            TYPE_AndruavMessage_UDPProxy_Info => {
                if fcb.info.is_gcs_blocked {
                    return;
                }
                if !meta.is_system
                    && (meta.permission & PERMISSION_ALLOW_GCS_FULL_CONTROL as u32)
                        != PERMISSION_ALLOW_GCS_FULL_CONTROL as u32
                {
                    return;
                }
                fcb.send_udp_proxy_status_of_me().await;
            }
            other => tracing::debug!(command = other, "remote execute not ported yet (R2b)"),
        }
    }

    async fn parse_command(&mut self, msg: &Incoming, meta: &MessageMeta) {
        let cmd = msg
            .json
            .get(ANDRUAV_PROTOCOL_MESSAGE_CMD)
            .cloned()
            .unwrap_or(Value::Null);
        let mut fcb = self.fcb.lock().await;
        match meta.message_type {
            TYPE_AndruavMessage_SWARM_MAVLINK => {
                if fcb.info.is_gcs_blocked {
                    return;
                }
                if !meta.is_system
                    && (meta.permission & PERMISSION_ALLOW_SWARM as u32)
                        != PERMISSION_ALLOW_SWARM as u32
                {
                    return;
                }
                let leader = sender(msg);
                fcb.handle_leader_traffic(&leader, &msg.raw).await;
            }
            TYPE_AndruavMessage_Make_Swarm => {
                if !fcb.info.is_gcs_blocked {
                    handle_make_swarm(&mut fcb, &cmd).await;
                }
            }
            TYPE_AndruavMessage_FollowHim_Request => {
                if !fcb.info.is_gcs_blocked {
                    handle_follow_him_request(&mut fcb, msg, &cmd, meta).await;
                }
            }
            TYPE_AndruavMessage_UpdateSwarm => {
                if !fcb.info.is_gcs_blocked {
                    handle_update_swarm(&mut fcb, &cmd).await;
                }
            }
            TYPE_AndruavSystem_UDPProxy => {
                if !meta.is_system {
                    return;
                }
                let (Some(s1), Some(s2)) = (cmd.get("socket1"), cmd.get("socket2")) else {
                    return;
                };
                if !(validate_field(s1, "address", JsonType::String)
                    && validate_field(s1, "port", JsonType::NumberUnsigned)
                    && validate_field(s2, "address", JsonType::String)
                    && validate_field(s2, "port", JsonType::NumberUnsigned)
                    && validate_field(&cmd, "en", JsonType::Boolean))
                {
                    return;
                }
                let en = cmd["en"].as_bool().unwrap_or(false);
                let ip1 = s1["address"].as_str().unwrap_or_default().to_string();
                let ip2 = s2["address"].as_str().unwrap_or_default().to_string();
                let p1 = as_cpp_int(&s1["port"]).unwrap_or(0);
                let p2 = as_cpp_int(&s2["port"]).unwrap_or(0);
                fcb.update_udp_proxy(en, &ip1, p1, &ip2, p2).await;
            }
            TYPE_AndruavModule_ID if meta.is_inter_module => {
                fcb.on_module_id(&cmd).await;
            }
            TYPE_AndruavMessage_Sync_EventFire
            | TYPE_AndruavMessage_UDPProxy_Info
            | TYPE_AndruavMessage_CONFIG_ACTION => {
                // mission event triggers: mission manager is R2b
            }
            other => tracing::debug!(message_type = other, "message not ported yet (R2b)"),
        }
    }
}

/// CSwarmManager::handleMakeSwarm (leader side).
async fn handle_make_swarm(fcb: &mut Fcb, cmd: &Value) {
    let Some(formation) = int_field(cmd, "a") else {
        return;
    };
    let Some(b) = str_field(cmd, "b") else { return };
    if fcb.info.party_id != b {
        return;
    }
    fcb.swarm.leader_min_h = int_field(cmd, "h").unwrap_or(KNODE_LENGTH);
    fcb.swarm.leader_min_v = int_field(cmd, "v").unwrap_or(KNODE_LENGTH);
    if fcb.swarm.formation_as_leader == formation {
        return;
    }
    fcb.swarm.formation_as_leader = formation;
    if formation == swarm::FORMATION_NO_SWARM {
        fcb.swarm.is_leader = false;
        fcb.release_followers().await;
    } else if fcb.swarm.is_leader {
        let fl: Vec<(String, i32)> = fcb
            .swarm
            .followers
            .iter()
            .map(|f| (f.party_id.clone(), f.follower_index))
            .collect();
        for (p, i) in fl {
            fcb.request_from_unit_to_change_formation(&p, i).await;
        }
    } else {
        fcb.swarm.is_leader = true;
    }
    fcb.api_ic_send_id("").await;
}

/// CSwarmManager::handleFollowHimRequest (follower side).
async fn handle_follow_him_request(fcb: &mut Fcb, msg: &Incoming, cmd: &Value, meta: &MessageMeta) {
    let Some(action) = int_field(cmd, "f") else {
        return;
    };
    let Some(c) = str_field(cmd, "c") else { return };
    fcb.swarm.follower_min_h = int_field(cmd, "h").unwrap_or(KNODE_LENGTH);
    fcb.swarm.follower_min_v = int_field(cmd, "v").unwrap_or(KNODE_LENGTH);
    if fcb.info.party_id != c {
        return; // info message for someone else
    }
    let requester = if meta.is_inter_module {
        fcb.module.party_id()
    } else {
        sender(msg)
    };
    match action {
        SWARM_UNFOLLOW => {
            let leader = str_field(cmd, "b")
                .map(str::to_string)
                .unwrap_or_else(|| fcb.swarm.leader_party_id.clone());
            // C++ passes the raw "sd" here, even for intermodule
            let by = sender(msg);
            fcb.unfollow_leader(&leader, &by).await;
        }
        SWARM_FOLLOW => {
            let Some(idx) = int_field(cmd, "a") else {
                return;
            };
            let Some(leader) = str_field(cmd, "b") else {
                return;
            };
            let formation = int_field(cmd, "d").unwrap_or(swarm::FORMATION_NO_SWARM);
            fcb.follow_leader(leader, idx, formation, &requester).await;
        }
        SWARM_CHANGE_FORMATION => {
            if requester != fcb.swarm.leader_party_id {
                return;
            }
            let Some(formation) = int_field(cmd, "d") else {
                return;
            };
            fcb.swarm.formation_as_follower = formation;
            fcb.api_ic_send_id("").await;
            fcb.module
                .send_error_message(
                    ANDRUAV_PROTOCOL_SENDER_ALL_GCS,
                    0,
                    ERROR_TYPE_LO7ETTA7AKOM,
                    NOTIFICATION_TYPE_WARNING,
                    &format!("SWAM Formation changed to {formation}"),
                )
                .await;
        }
        _ => {}
    }
}

/// CSwarmManager::handleUpdateSwarm (leader side).
async fn handle_update_swarm(fcb: &mut Fcb, cmd: &Value) {
    let Some(action) = int_field(cmd, "a") else {
        return;
    };
    let Some(c) = str_field(cmd, "c") else { return };
    let Some(d) = str_field(cmd, "d") else { return };
    if fcb.info.party_id != c {
        return;
    }
    let follower = d.to_string();
    if action == SWARM_ADD {
        fcb.add_follower(&follower).await;
    } else if action == SWARM_DELETE {
        fcb.release_single_follower(&follower).await;
    }
}
