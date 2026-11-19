// Ported from droneengage_common@5fb11c2: de_databus/de_facade_base.cpp, de_facade_base.hpp
//
// C++ CFacade_Base is a singleton wrapping CModule. In Rust these are just
// more methods on `Module`; module-specific facades (de_mavlink's) build on them.

use serde_json::{json, Value};

use crate::messages::*;
use crate::module::Module;

impl Module {
    /// Ask a party (or everyone, `""`) to send its ID (RemoteExecute {C: 1004}, intermodule).
    pub async fn request_id(&self, target_party_id: &str) {
        self.send_jmsg(
            target_party_id,
            json!({"C": TYPE_AndruavMessage_ID}),
            TYPE_AndruavMessage_RemoteExecute,
            true,
        )
        .await;
    }

    /// TYPE_AndruavMessage_Error to GCS: EN error number, IT info type, NT severity, DS text.
    pub async fn send_error_message(
        &self,
        target_party_id: &str,
        error_number: i32,
        info_type: i32,
        notification_type: i32,
        description: &str,
    ) {
        let msg = json!({"EN": error_number, "IT": info_type, "NT": notification_type, "DS": description});
        self.send_jmsg(target_party_id, msg, TYPE_AndruavMessage_Error, false)
            .await;
        tracing::info!(%description, "sendErrorMessage");
    }

    /// Memory ceiling and growth thresholds used by `send_memory_status`.
    pub fn configure_memory_status(&self, max_rss_mb: f64, max_growth_mb_per_hour: f64) {
        self.health_monitor()
            .configure(max_rss_mb, max_growth_mb_per_hour, 40);
    }

    /// Sample own memory and send TYPE_AndruavMessage_MODULE_HEALTH_STATUS.
    /// Called every 30 s by the health loop; no-op without /proc.
    pub async fn send_memory_status(&self, target_party_id: &str) {
        let Some(h) = self.health_monitor().sample() else {
            return;
        };
        let msg = json!({
            "a": MODULE_HEALTH_ACTION_STATUS,
            "k": self.module_key(),
            "rs": h.rss_mb,
            "pk": h.peak_rss_mb,
            "sw": h.vmswap_mb,
            "th": h.threads,
            "sl": h.slope_mb_h,
            "tr": h.trend,
            "hs": h.status,
            "up": h.uptime_sec,
        });
        self.send_jmsg(
            target_party_id,
            msg,
            TYPE_AndruavMessage_MODULE_HEALTH_STATUS,
            false,
        )
        .await;
    }

    /// Board CPU report to de_comm only (6547, intermodule).
    pub async fn send_cpu_status(
        &self,
        board_id: &str,
        board_usage_percent: i32,
        process_usage_percent: i32,
        board_state: i32,
        report_event: i32,
    ) {
        let msg = crate::cpu::build_cpu_status_json(
            board_id,
            board_usage_percent,
            process_usage_percent,
            board_state,
            report_event,
        );
        self.send_jmsg("", msg, TYPE_AndruavMessage_MODULE_CPU_STATUS, true)
            .await;
    }

    /// Reply with this module's config template (TYPE_AndruavMessage_CONFIG_STATUS).
    pub async fn api_send_config_template(
        &self,
        target_party_id: &str,
        module_key: &str,
        template: Value,
        reply: bool,
    ) {
        let msg = json!({
            "a": CONFIG_STATUS_FETCH_CONFIG_TEMPLATE,
            "b": template,
            "k": module_key,
            "R": reply,
        });
        self.send_jmsg(
            target_party_id,
            msg,
            TYPE_AndruavMessage_CONFIG_STATUS,
            false,
        )
        .await;
    }
}
