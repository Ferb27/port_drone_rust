# PATCHES: `de_fork` (Diff & Lý do)

Tổng hợp đầy đủ diff so với các commit upstream đã pin (không gồm `ssl_local/` tự ký cho AirGap TLS cục bộ và file `LICENSE` / `NOTES.md`).
B0 giữ nguyên — **không** thêm velocity feed-forward vào follower.

---

## 1. `de_mavlink` (pinned @ `be5d8a5`) — P1 & P2

- **P1:** Leader gắn `ms.q` (`uint32` sequence) và `ms.t` (`uint32` ms từ boot) vào msg `6503` (`SWARM_MAVLINK`); follower bỏ bản tin cũ/trùng theo `q` và reset trạng thái lọc sau **5 s** mất leader (`checkLeaderTimeout()` gọi ở 10 Hz).
- **P2:** Cài handler `FORMATION_VECTOR` (enum = 3) cho cả `CSwarmLeader` và `CSwarmFollower` (dùng chung `broadcastLeaderState()` và `resolveFormationBearing()`, giữ LPF B0, không velocity feed-forward).

```diff
diff --git a/src/fcb_facade.cpp b/src/fcb_facade.cpp
index efea2a9..c351bda 100644
--- a/src/fcb_facade.cpp
+++ b/src/fcb_facade.cpp
@@ -826,7 +826,7 @@ void CFCBFacade::sendMavlinkData_Packed(const std::string& target_party_id, cons
 }
 
 
-void CFCBFacade::sendSWARM_M(const std::string& target_party_id, const mavlink_message_t* mavlink_message, const uint16_t count) const
+void CFCBFacade::sendSWARM_M(const std::string& target_party_id, const mavlink_message_t* mavlink_message, const uint16_t count, const Json_de& message_cmd) const
 {
     if (count == 0) return;
 
@@ -855,7 +855,7 @@ void CFCBFacade::sendSWARM_M(const std::string& target_party_id, const mavlink_m
     }
 
     // Send the combined messages
-    m_module.sendBMSG(target_party_id, reinterpret_cast<char*>(buf.data()), len, TYPE_AndruavMessage_SWARM_MAVLINK, true, Json_de());
+    m_module.sendBMSG(target_party_id, reinterpret_cast<char*>(buf.data()), len, TYPE_AndruavMessage_SWARM_MAVLINK, true, message_cmd);
 
     return;
 }
diff --git a/src/fcb_facade.hpp b/src/fcb_facade.hpp
index 4b5e7d0..21eb574 100644
--- a/src/fcb_facade.hpp
+++ b/src/fcb_facade.hpp
@@ -119,7 +119,7 @@ namespace fcb
             void requestFromUnitToFollowMe(const std::string&target_party_id, const int follower_index) const;
             void requestFromUnitToChangeFormation(const std::string&target_party_id, const int follower_index) const;
             void requestFromUnitToUnFollowMe(const std::string&target_party_id) const;
-            void sendSWARM_M(const std::string&target_party_id, const mavlink_message_t* mavlink_message, const uint16_t count)  const;
+            void sendSWARM_M(const std::string&target_party_id, const mavlink_message_t* mavlink_message, const uint16_t count, const Json_de& message_cmd = Json_de())  const;
             // END SWARM API
             
             void requestUdpProxyTelemetry(const bool enable, const std::string&udp_ip1, const int& udp_port1, const std::string&udp_ip2, const int& udp_port2);
diff --git a/src/fcb_main.cpp b/src/fcb_main.cpp
index 4b9ab15..b2be704 100644
--- a/src/fcb_main.cpp
+++ b/src/fcb_main.cpp
@@ -24,6 +24,7 @@
 #include "./geofence/fcb_geo_fence_base.hpp"
 #include "./geofence/fcb_geo_fence_manager.hpp"
 #include "./swarm/fcb_swarm_leader.hpp"
+#include "./swarm/fcb_swarm_follower.hpp"
 
 #include "./geofence/fcb_geo_fence_base.hpp"
 #include "./geofence/fcb_geo_fence_manager.hpp"
@@ -579,6 +580,7 @@ void CFCBMain::loopScheduler() {
 
     if (m_counter % 10 == 0) { // each 100 msec
       fcb_swarm_leader.handleSwarmsAsLeader();
+      swarm::CSwarmFollower::getInstance().checkLeaderTimeout();
 
       // Update DRONEENGAGE_PILOT operations
       if (de::fcb::depilot::CDEPilotManager::getInstance().getActive()) {
diff --git a/src/swarm/fcb_swarm_follower.cpp b/src/swarm/fcb_swarm_follower.cpp
index 3fd4c15..913bd01 100644
--- a/src/swarm/fcb_swarm_follower.cpp
+++ b/src/swarm/fcb_swarm_follower.cpp
@@ -18,6 +18,7 @@ de::fcb::swarm::CSwarmManager& fcb_swarm_manager = de::fcb::swarm::CSwarmManager
 #define HDG_DEADZONE_DEG 5.0                  // ignore compass heading changes below this (degrees)
 #define CROSS_TRACK_DEADZONE_M 3.0            // deadzone in meters for dynamic side switching
 #define TARGET_JUMP_THRESHOLD_M 5.0           // if raw target jumps more than this, check direction before chasing
+#define SWARM_LEADER_TIMEOUT_USEC 5000000ULL  // P1: 5s leader-loss timeout
 
 
 /**
@@ -304,6 +305,102 @@ void CSwarmFollower::updateFollowerInArrowFormation(const bool is_dynamic)
  * @brief Update me as a follower based on the active formation.
  * 
  */
+bool CSwarmFollower::resolveFormationBearing(const bool is_dynamic, double& out_bearing)
+{
+    double speed_sq = m_leader_gpos_new.vx * m_leader_gpos_new.vx + m_leader_gpos_new.vy * m_leader_gpos_new.vy;
+
+    if (speed_sq >= SPEED_STATIONARY_THRESHOLD_SQ)
+    {
+        double formation_bearing = getBearingOfVector(m_leader_gpos_new.vx, m_leader_gpos_new.vy);
+        if (m_has_last_bearing)
+        {
+            double bearing_diff = formation_bearing - m_last_leader_bearing;
+            while (bearing_diff > M_PI) bearing_diff -= 2.0 * M_PI;
+            while (bearing_diff < -M_PI) bearing_diff += 2.0 * M_PI;
+
+            const double max_slew = MAX_BEARING_SLEW_RATE_DEG * M_PI / 180.0;
+            if (fabs(bearing_diff) > max_slew)
+            {
+                formation_bearing = m_last_leader_bearing + (bearing_diff > 0 ? max_slew : -max_slew);
+            }
+        }
+        m_last_leader_bearing = formation_bearing;
+        m_has_last_bearing = true;
+        out_bearing = formation_bearing;
+        return true;
+    }
+
+    if (is_dynamic)
+    {
+        if (!m_has_last_bearing) return false;
+        out_bearing = m_last_leader_bearing;
+        return true;
+    }
+
+    if (m_leader_gpos_new.hdg != 65535 && m_leader_gpos_new.hdg < 36000)
+    {
+        double hdg_rad = m_leader_gpos_new.hdg / 100.0 * M_PI / 180.0;
+        if (m_has_last_bearing)
+        {
+            double hdg_diff = hdg_rad - m_last_leader_bearing;
+            while (hdg_diff > M_PI) hdg_diff -= 2.0 * M_PI;
+            while (hdg_diff < -M_PI) hdg_diff += 2.0 * M_PI;
+
+            if (fabs(hdg_diff) > HDG_DEADZONE_DEG * M_PI / 180.0)
+            {
+                m_last_leader_bearing = hdg_rad;
+            }
+        }
+        else
+        {
+            m_last_leader_bearing = hdg_rad;
+            m_has_last_bearing = true;
+        }
+        out_bearing = m_last_leader_bearing;
+        return true;
+    }
+
+    if (!m_has_last_bearing) return false;
+    out_bearing = m_last_leader_bearing;
+    return true;
+}
+
+void CSwarmFollower::updateFollowerInVectorFormation()
+{
+    double formation_bearing = 0.0;
+    if (!resolveFormationBearing(false, formation_bearing)) return;
+
+    const double leader_lat = m_leader_gpos_new.lat / 1e7;
+    const double leader_lon = m_leader_gpos_new.lon / 1e7;
+    const int follower_index = fcb_swarm_manager.getFollowerIndex();
+    const double base_distance = (follower_index + 1) * m_min_horizontal_distance;
+
+    POINT_2D p = get_point_at_bearing(leader_lat, leader_lon, formation_bearing + M_PI, base_distance);
+
+    if (m_has_smoothed_target)
+    {
+        m_smoothed_target_lat = TARGET_SMOOTHING_ALPHA * p.latitude + (1.0 - TARGET_SMOOTHING_ALPHA) * m_smoothed_target_lat;
+        m_smoothed_target_lon = TARGET_SMOOTHING_ALPHA * p.longitude + (1.0 - TARGET_SMOOTHING_ALPHA) * m_smoothed_target_lon;
+    }
+    else
+    {
+        m_smoothed_target_lat = p.latitude;
+        m_smoothed_target_lon = p.longitude;
+        m_has_smoothed_target = true;
+    }
+
+    mavlinksdk::CMavlinkCommand::getInstance().gotoGuidedPoint(
+        m_smoothed_target_lat, m_smoothed_target_lon,
+        (m_leader_gpos_new.relative_alt + (follower_index + 1) * m_min_vertical_distance * 1000) / 1000.0f);
+
+    CFCBFacade::getInstance().sendFCBTargetLocation(
+        "", m_smoothed_target_lat, m_smoothed_target_lon,
+        (double)m_leader_gpos_new.relative_alt, DESTINATION_SWARM_MY_LOCATION);
+
+    m_leader_last_access = get_time_usec_monotonic();
+    m_leader_gpos_old = m_leader_gpos_new;
+}
+
 void CSwarmFollower::updateFollower()
 {
     de::fcb::swarm::CSwarmManager& fcb_swarm_manager = de::fcb::swarm::CSwarmManager::getInstance();
@@ -320,6 +417,7 @@ void CSwarmFollower::updateFollower()
             updateFollowerInArrowFormation(true);
             break;
         case FORMATION_VECTOR:
+            updateFollowerInVectorFormation();
             break;
         
         default:
@@ -338,7 +436,7 @@ void CSwarmFollower::updateFollower()
  * @param full_message 
  * @param full_message_length 
  */
-void CSwarmFollower::handle_leader_traffic(const std::string & leader_sender, const char * full_message, const int & full_message_length)
+void CSwarmFollower::handle_leader_traffic(const std::string & leader_sender, const char * full_message, const int & full_message_length, const Json_de& message_cmd)
 {
 
     // suggest: send unfollow to it. but take care of message rate.
@@ -348,6 +446,18 @@ void CSwarmFollower::handle_leader_traffic(const std::string & leader_sender, co
     // This traffic is not from my leader.
     if (!fcb_swarm_manager.isMyLeader(leader_sender)) return ;
 
+    // P1: Drop stale/out-of-order messages when sequence number is present
+    if (message_cmd.is_object() && message_cmd.contains("q") && message_cmd["q"].is_number_unsigned())
+    {
+        const uint32_t seq = message_cmd["q"].get<uint32_t>();
+        if (m_has_leader_seq && static_cast<int32_t>(seq - m_last_leader_seq) <= 0)
+        {
+            return;
+        }
+        m_last_leader_seq = seq;
+        m_has_leader_seq = true;
+    }
+
 
     // this is a binary message
     // search for char '0' and then binary message is the next byte after it.
@@ -399,6 +509,23 @@ void CSwarmFollower::handle_leader_traffic(const std::string & leader_sender, co
     
     if (valid) 
     {
+        m_last_leader_rx_usec = get_time_usec_monotonic();
+        m_leader_rx_active = true;
         updateFollower();
     }
+}
+
+void CSwarmFollower::checkLeaderTimeout()
+{
+    if (!fcb_swarm_manager.isFollower() || !m_leader_rx_active) return;
+
+    if ((get_time_usec_monotonic() - m_last_leader_rx_usec) > SWARM_LEADER_TIMEOUT_USEC)
+    {
+        std::cout << _ERROR_CONSOLE_BOLD_TEXT_ << "SWARM: Leader traffic timeout (5s). Holding last target." << _NORMAL_CONSOLE_TEXT_ << std::endl;
+        m_leader_rx_active = false;
+        m_has_leader_seq = false;
+        m_has_smoothed_target = false;
+        m_has_last_bearing = false;
+        m_has_prev_raw_target = false;
+    }
 }
\ No newline at end of file
diff --git a/src/swarm/fcb_swarm_follower.hpp b/src/swarm/fcb_swarm_follower.hpp
index 32a4855..dc63b4b 100644
--- a/src/swarm/fcb_swarm_follower.hpp
+++ b/src/swarm/fcb_swarm_follower.hpp
@@ -49,7 +49,8 @@ namespace swarm
 
         public:
 
-            void handle_leader_traffic(const std::string & leader_sender, const char * full_message, const int & full_message_length);
+            void handle_leader_traffic(const std::string & leader_sender, const char * full_message, const int & full_message_length, const Json_de& message_cmd = Json_de());
+            void checkLeaderTimeout();
 
         public: 
 
@@ -75,6 +76,8 @@ namespace swarm
             void updateFollower();
             void updateFollowerInThreadFormation();
             void updateFollowerInArrowFormation(const bool is_dynamic);
+            void updateFollowerInVectorFormation();
+            bool resolveFormationBearing(const bool is_dynamic, double& out_bearing);
 
         private:
             
@@ -104,6 +107,12 @@ namespace swarm
             bool m_has_prev_raw_target = false;
             int m_consecutive_skips = 0;
 
+            // P1: stale message filtering (sequence) and 5s leader timeout tracking
+            uint32_t m_last_leader_seq = 0;
+            bool m_has_leader_seq = false;
+            uint64_t m_last_leader_rx_usec = 0;
+            bool m_leader_rx_active = false;
+
         private:
             
             int m_min_vertical_distance = KNODE_LENGTH;
diff --git a/src/swarm/fcb_swarm_leader.cpp b/src/swarm/fcb_swarm_leader.cpp
index b57ce41..0e6292a 100644
--- a/src/swarm/fcb_swarm_leader.cpp
+++ b/src/swarm/fcb_swarm_leader.cpp
@@ -8,24 +8,28 @@ using namespace de::fcb::swarm;
 #define DEF_SWARM_LEADER_LOCATION_UPDATE_RATE 1000000
 
 
-/**
- * @brief Send target location for followers that forming thread formation.
- * 
- */
-void CSwarmLeader::updateFollowersThreadFormation()
+void CSwarmLeader::broadcastLeaderState()
 {
-    mavlinksdk::CVehicle &vehicle =  mavlinksdk::CVehicle::getInstance();
-        
+    mavlinksdk::CVehicle &vehicle = mavlinksdk::CVehicle::getInstance();
+
     // get my attitude and position to share with followers.
     const mavlink_attitude_t& attitude = vehicle.getMsgAttitude();
-    const mavlink_global_position_int_t&  my_gpos = vehicle.getMsgGlobalPositionInt();
+    const mavlink_global_position_int_t& my_gpos = vehicle.getMsgGlobalPositionInt();
+
+    const uint32_t ts_boot_ms = (my_gpos.time_boot_ms != 0)
+        ? my_gpos.time_boot_ms
+        : static_cast<uint32_t>(get_time_usec_monotonic() / 1000ULL);
+    const Json_de ms = {
+        {"q", ++m_swarm_seq},
+        {"t", ts_boot_ms}
+    };
 
     // loop on followers unit
     const std::vector<ANDRUAV_UNIT_FOLLOWER>& follower_units = m_fcb_swarm_manager.getFollowerUnits();
-    for (const auto& item : follower_units) 
+    for (const auto& item : follower_units)
     {
         mavlink_message_t mavlink_message[2];
-        
+
         const int sys_id = vehicle.getSysId();
         const int comp_id = vehicle.getCompId();
 
@@ -33,38 +37,26 @@ void CSwarmLeader::updateFollowersThreadFormation()
         mavlink_msg_global_position_int_encode(sys_id, comp_id, &mavlink_message[0], &my_gpos);
         // roll - pitch - yaw - rollspeed - pitchspeed - yawspeed
         mavlink_msg_attitude_encode(sys_id, comp_id, &mavlink_message[1], &attitude);
-            
-        // forward info to followers.
-        de::fcb::CFCBFacade::getInstance().sendSWARM_M (item.party_id, mavlink_message, 2);
+
+        // forward info to followers with sequence and boot timestamp.
+        de::fcb::CFCBFacade::getInstance().sendSWARM_M(item.party_id, mavlink_message, 2, ms);
     }
-} 
+}
 
 
-void CSwarmLeader::updateFollowerInArrowFormation()
+/**
+ * @brief Send target location for followers that forming thread formation.
+ * 
+ */
+void CSwarmLeader::updateFollowersThreadFormation()
 {
-    mavlinksdk::CVehicle &vehicle =  mavlinksdk::CVehicle::getInstance();
-        
-    // get my attitude and position to share with followers.
-    const mavlink_attitude_t& attitude = vehicle.getMsgAttitude();
-    const mavlink_global_position_int_t&  my_gpos = vehicle.getMsgGlobalPositionInt();
+    broadcastLeaderState();
+} 
 
-    // loop on followers unit
-    const std::vector<ANDRUAV_UNIT_FOLLOWER>& follower_units = m_fcb_swarm_manager.getFollowerUnits();
-    for (const auto& item : follower_units) 
-    {
-        mavlink_message_t mavlink_message[2];
-        
-        const int sys_id = vehicle.getSysId();
-        const int comp_id = vehicle.getCompId();
 
-        // time_boot_ms - lat - lon - alt - relative - alt - vx - vy - vz - hdg
-        mavlink_msg_global_position_int_encode(sys_id, comp_id, &mavlink_message[0], &my_gpos);
-        // roll - pitch - yaw - rollspeed - pitchspeed - yawspeed
-        mavlink_msg_attitude_encode(sys_id, comp_id, &mavlink_message[1], &attitude);
-            
-        // forward info to followers.
-        de::fcb::CFCBFacade::getInstance().sendSWARM_M (item.party_id, mavlink_message, 2);
-    }
+void CSwarmLeader::updateFollowerInArrowFormation()
+{
+    broadcastLeaderState();
 }
 
 /**
@@ -84,6 +76,7 @@ void CSwarmLeader::updateFollowers()
             updateFollowerInArrowFormation();
             break;
         case FORMATION_VECTOR:
+            broadcastLeaderState();
             break;
         
         default:
diff --git a/src/swarm/fcb_swarm_leader.hpp b/src/swarm/fcb_swarm_leader.hpp
index 7e2b59a..5d6bd26 100644
--- a/src/swarm/fcb_swarm_leader.hpp
+++ b/src/swarm/fcb_swarm_leader.hpp
@@ -75,6 +75,7 @@ namespace swarm
 
         private:
 
+            void broadcastLeaderState();
             void updateFollowers();
             void updateFollowersThreadFormation();
             void updateFollowerInArrowFormation();
@@ -83,6 +84,7 @@ namespace swarm
             
             mavlink_global_position_int_t m_leader_gpos_latest;
             u_int64_t m_leader_last_access;
+            uint32_t m_swarm_seq = 0;
 
         private:
             
diff --git a/src/swarm/fcb_swarm_manager.cpp b/src/swarm/fcb_swarm_manager.cpp
index c36ee8b..9e789ed 100644
--- a/src/swarm/fcb_swarm_manager.cpp
+++ b/src/swarm/fcb_swarm_manager.cpp
@@ -510,8 +510,11 @@ void CSwarmManager::handleFollowHimRequest(const Json_de& andruav_message, const
 void CSwarmManager::handleSwarmMavlink(const Json_de& andruav_message, const char * full_message, const int & full_message_length)
 {
     const std::string leader_sender = andruav_message[ANDRUAV_PROTOCOL_SENDER].get<std::string>();
+    const Json_de message_cmd = andruav_message.contains(ANDRUAV_PROTOCOL_MESSAGE_CMD)
+        ? andruav_message[ANDRUAV_PROTOCOL_MESSAGE_CMD]
+        : Json_de();
     de::fcb::swarm::CSwarmFollower& swarm_follower = de::fcb::swarm::CSwarmFollower::getInstance();
-    swarm_follower.handle_leader_traffic(leader_sender, full_message, full_message_length);
+    swarm_follower.handle_leader_traffic(leader_sender, full_message, full_message_length, message_cmd);
 }
```

---

## 2. `de_server` (pinned @ `53a092f`, v5.6.2) — P4 & Rebrand

- **P4:** Đọc `process.env.de_public_port` kiểu `int` (`parseInt`) trong `js_andruav_comm_server.js` để chạy sau delay proxy; fallback `de_comm_server_port` → `server_port` như upstream.
- **Rebrand & pnpm:** Đổi `name` → `de_server`, `version` → `5.7.0-fork`, `git+ssh://` → `https://` cho `droneengage_server_common`, và cấu hình AirGap mặc định trong `server.config` (`0.0.0.0`, `127.0.0.1`, `enable_storage_server: false`).

```diff
diff --git a/package.json b/package.json
index f814acb..d433f0b 100644
--- a/package.json
+++ b/package.json
@@ -1,6 +1,6 @@
 {
-  "name": "andruavserver",
-  "version": "5.6.2",
+  "name": "de_server",
+  "version": "5.7.0-fork",
   "author": {
     "name": "Mohammad Said Hefny",
     "email": "mohammad.hefny@gmail.com"
@@ -38,6 +38,6 @@
     "udp-packet": "^2.0.0",
     "uuid": "^9.0.0",
     "ws": "^8.16.0",
-    "droneengage_server_common": "git+ssh://git@github.com:DroneEngage/droneengage_server_common.git#v1.0.0"
+    "droneengage_server_common": "https://github.com/DroneEngage/droneengage_server_common.git#v1.0.0"
   }
 }
diff --git a/server.config b/server.config
index 98a9ea1..5660e39 100755
--- a/server.config
+++ b/server.config
@@ -1,7 +1,7 @@
 {
     "server_id"                     : "DE_CommSrv",
-    "server_ip"                     : "::",
-    "public_host"                   : "airgap.local",
+    "server_ip"                     : "0.0.0.0",
+    "public_host"                   : "127.0.0.1",
     "server_port"                   : 9966,
     "s2s_ws_target_ip"              : "127.0.0.1",
     "s2s_ws_target_port"            : "19001",
@@ -26,7 +26,7 @@
     // s2s_auth_enabled is managed by the authenticator only; comm server always attempts S2S auth.
     "s2s_my_private_key"            : "./ssl_local/DE_ServerComm_private.pem",
     // Storage server endpoint for direct connection
-    "enable_storage_server"         : true,
+    "enable_storage_server"         : false,
     "storage_server_host"           : "127.0.0.1",
     "storage_server_port"           : 9000,
     // logging
diff --git a/src/comm_server/js_andruav_comm_server.js b/src/comm_server/js_andruav_comm_server.js
index 34d20d9..ea17dcb 100644
--- a/src/comm_server/js_andruav_comm_server.js
+++ b/src/comm_server/js_andruav_comm_server.js
@@ -125,7 +125,7 @@ function fn_generateLoginRequestReply(p_cmd) {
     c_reply.d[c_CONSTANTS.CONST_CS_REQUEST_ID.toString()] = p_cmd.d[c_CONSTANTS.CONST_CS_REQUEST_ID.toString()];
     c_reply.d[c_CONSTANTS.CONST_CS_ERROR.toString()] = c_CONSTANTS.CONST_ERROR_NON;
     c_reply.d[c_CONSTANTS.CONST_CS_SERVER_PUBLIC_HOST.toString()] = global.m_serverconfig.m_configuration.public_host;
-    c_reply.d[c_CONSTANTS.CONST_CS_SERVER_PORT.toString()] = process.env.de_comm_server_port || global.m_serverconfig.m_configuration.server_port;
+    c_reply.d[c_CONSTANTS.CONST_CS_SERVER_PORT.toString()] = (process.env.de_public_port ? parseInt(process.env.de_public_port, 10) : null) || process.env.de_comm_server_port || global.m_serverconfig.m_configuration.server_port;
     c_reply.d[c_CONSTANTS.CONST_CS_LOGIN_TEMP_KEY.toString()] = p_cmd.d[c_CONSTANTS.CONST_CS_LOGIN_TEMP_KEY.toString()];
 
     return c_reply;
@@ -273,7 +273,7 @@ function fn_updateServerWatchdog() {
             'version': v_version,
             'serverId': global.m_serverconfig.m_configuration.server_id,
             'public_host': global.m_serverconfig.m_configuration.public_host, // this is the ip that is listening to the connections.
-            'serverPort': process.env.de_comm_server_port || global.m_serverconfig.m_configuration.server_port,
+            'serverPort': (process.env.de_public_port ? parseInt(process.env.de_public_port, 10) : null) || process.env.de_comm_server_port || global.m_serverconfig.m_configuration.server_port,
             // this is why we send this message with every connection.
             // it is crucial to Auth server to know the exact unique list of keys
             // so that it can rout correctly in case of multiple communication_servers
```

---

## 3. `de_authenticator` (pinned @ `584d842`) — P3 & Rebrand

- **P3:** Nâng `RATE_LIMIT_MAX_REQUESTS` từ `10` lên `1000` req/phút trong `js_router_agent.js` để chạy batch thí nghiệm SITL.
- **Rebrand & pnpm:** Đổi `name` → `de_authenticator`, `version` → `5.7.0-fork`, `git+ssh://` → `https://`, thêm `body-parser` cho pnpm strict mode, và đặt `altcha_enabled: false` cho AirGap cục bộ (`account_storage_type: "single"`).

```diff
diff --git a/package.json b/package.json
index 2f5d596..8a66d8f 100644
--- a/package.json
+++ b/package.json
@@ -1,6 +1,6 @@
 {
-  "name": "DE_AUTH",
-  "version": "7.7.0",
+  "name": "de_authenticator",
+  "version": "5.7.0-fork",
   "author": {
     "name": "Mohammad Said Hefny",
     "email": "mohammad.hefny@gmail.com"
@@ -24,11 +24,12 @@
     "altcha-lib": "^1.4.1",
     "bcrypt": "^6.0.0",
     "bcryptjs": "^3.0.3",
+    "body-parser": "^1.20.2",
     "commander": "^2.11.0",
     "cookie-parser": "^1.3.5",
     "cors": "^2.8.5",
     "csurf": "^1.11.0",
-    "droneengage_server_common": "git+ssh://git@github.com:DroneEngage/droneengage_server_common.git#v1.0.1",
+    "droneengage_server_common": "https://github.com/DroneEngage/droneengage_server_common.git#v1.0.1",
     "ejs": "^3.1.8",
     "express": "^4.19.2",
     "express-ejs-layouts": "^2.5.0",
diff --git a/server.config b/server.config
index e688850..3057e23 100644
--- a/server.config
+++ b/server.config
@@ -74,7 +74,7 @@
     // Set altcha_enabled=false for trusted/LAN installs. Leave
     // altcha_hmac_secret empty to use an ephemeral per-boot secret (in-flight
     // challenges die on restart); set a fixed secret to survive restarts.
-    "altcha_enabled"            : true,
+    "altcha_enabled"            : false,
     "altcha_hmac_secret"        : "",
     "altcha_max_number"         : 100000,
     "altcha_ttl_sec"            : 300,
diff --git a/src/routes/js_router_agent.js b/src/routes/js_router_agent.js
index 2d64fa2..6ba6d16 100755
--- a/src/routes/js_router_agent.js
+++ b/src/routes/js_router_agent.js
@@ -6,7 +6,7 @@ const C = global.c_CONSTANTS;
 // Simple in-memory rate limiter
 const rateLimitMap = new Map();
 const RATE_LIMIT_WINDOW = 60000; // 1 minute
-const RATE_LIMIT_MAX_REQUESTS = 10; // Max 10 requests per minute per IP
+const RATE_LIMIT_MAX_REQUESTS = 1000; // Max 10 requests per minute per IP
 
 const accountRateLimitMap = new Map();
 const ACCOUNT_RATE_LIMIT_WINDOW = 60000; // 1 minute
```

---

## 4. `de_webclient_react` (pinned @ `0171010`) — Rebrand

- Đổi `name` → `de_webclient_react`, `version` → `5.7.0-fork`.

```diff
diff --git a/package.json b/package.json
index 80fb29c7..6fb9b6ea 100644
--- a/package.json
+++ b/package.json
@@ -1,6 +1,6 @@
 {
-  "name": "droneengage_webclient",
-  "version": "3.1.0",
+  "name": "de_webclient_react",
+  "version": "5.7.0-fork",
   "build_number": "20260918-01",
   "description": "DroneEngage WebClient GCS Control Station.",
   "author": {
```

---

## 5. Rebrand & `DE_FORK=1` (5.7.0-fork) — Cả 6 repo (2026-11-19)

- **Author:** `"K. Pham"`, email bỏ trống.
- **URL base:** `https://github.com/dn-nckh/de_fork`
- **Version:** `5.7.0-fork`, kèm macro/biến `DE_FORK=1`.
- **License:** The Unlicense (`LICENSE` ở thư mục gốc và từng repo).

### 5.1 `de_mavlink` (version/CMake/README)
```diffdiff --git a/.version b/.version index ac5f40c..d7b273d 100644 --- a/.version +++ b/.version @@ -1 +1 @@ -5.6.8.1 \ No newline at end of file +5.7.0-fork diff --git a/CMakeLists.txt b/CMakeLists.txt index 5baa2bc..cefc07f 100644 --- a/CMakeLists.txt +++ b/CMakeLists.txt @@ -30,10 +30,10 @@ endif()  # Auto-increment version on each build  set(VERSION_FILE "${CMAKE_SOURCE_DIR}/.version")   -# Manually set major, minor, and bugfix versions (you can change these) -set(MAJOR_VERSION 7) -set(MINOR_VERSION 4) -set(BUGFIX_VERSION 5) +# Phien ban fork NCKH (5.7.0-fork) - giu version so cho CMake project() +set(MAJOR_VERSION 5) +set(MINOR_VERSION 7) +set(BUGFIX_VERSION 0)    if(EXISTS ${VERSION_FILE})      file(READ ${VERSION_FILE} CURRENT_VERSION) @@ -69,15 +69,14 @@ ELSE()      message(STATUS "Non-release build - keeping current build version")  ENDIF()   -# Create new version string -set(__APP__VERSION__ "${MAJOR_VERSION}.${MINOR_VERSION}.${BUGFIX_VERSION}.${BUILD_VERSION}") +# Chuoi phien ban va co DE_FORK=1 cho ban fork NCKH +set(__APP__VERSION__ "5.7.0-fork")    message(STATUS "Building version: ${__APP__VERSION__}") -message(STATUS "To change major/minor/bugfix versions, edit MAJOR_VERSION, MINOR_VERSION, and BUGFIX_VERSION above")    SET (__APP__VERSION__ "${__APP__VERSION__}") -add_definitions( -D__APP__VERSION__="${__APP__VERSION__}") -project(DE_MAVLINK_PLUGIN VERSION "${__APP__VERSION__}") +add_definitions( -D__APP__VERSION__="${__APP__VERSION__}" -DDE_FORK=1) +project(de_fork_mavlink VERSION "${MAJOR_VERSION}.${MINOR_VERSION}.${BUGFIX_VERSION}.${BUILD_VERSION}")       diff --git a/README.md b/README.md index a42a51c..f60c02c 100644 --- a/README.md +++ b/README.md @@ -1,12 +1,8 @@ -[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/DroneEngage/droneengage_mavlink) +# de_fork_mavlink (v5.7.0-fork)   -[![Ardupilot Cloud EcoSystem](https://cloud.ardupilot.org/_static/ardupilot_logo.png "Ardupilot Cloud EcoSystem")](https://cloud.ardupilot.org "Ardupilot Cloud EcoSystem") **Drone Engage** is part of Ardupilot Cloud Eco System - ------------- - -![Drone Engage Mavlink Module](resources/de_logo_title.png)  +> Bản fork NCKH từ `droneengage_mavlink` — Tác giả fork: **K. Pham** | Repo: <https://github.com/dn-nckh/de_fork> | Giấy phép: The Unlicense.   -#Drone Engage Mavlink Module +# Drone Engage Mavlink Module    # What is Drone-Mavlink ?   @@ -27,8 +23,8 @@ For a quick build with default settings:    ```bash  # Clone the repository -git clone https://github.com/DroneEngage/droneengage_mavlink.git -cd droneengage_mavlink +git clone https://github.com/dn-nckh/de_fork.git +cd de_fork/de_mavlink    # Build using the provided script  ./build.sh @@ -42,8 +38,8 @@ cd droneengage_mavlink    ```bash  # Clone the repository -git clone https://github.com/DroneEngage/droneengage_mavlink.git -cd droneengage_mavlink +git clone https://github.com/dn-nckh/de_fork.git +cd de_fork/de_mavlink    # Create build directory  mkdir build && cd build diff --git a/src/version.h b/src/version.h index a67dd44..679c90e 100644 --- a/src/version.h +++ b/src/version.h @@ -6,5 +6,14 @@  #include <stdio.h>     +// Phien ban fork NCKH - K. Pham +#ifndef DE_FORK +#define DE_FORK 1 +#endif + +#ifndef __APP__VERSION__ +#define __APP__VERSION__ "5.7.0-fork" +#endif +  static std::string version_string = __APP__VERSION__;   #endif \ No newline at end of file
```

### 5.2 `de_communication`
```diffdiff --git a/CMakeLists.txt b/CMakeLists.txt index 4f5c309..0617c3a 100644 --- a/CMakeLists.txt +++ b/CMakeLists.txt @@ -30,10 +30,10 @@ endif()  # Auto-increment version on each build  set(VERSION_FILE "${CMAKE_SOURCE_DIR}/.version")   -# Manually set major, minor, and bugfix versions (you can change these) -set(MAJOR_VERSION 4) -set(MINOR_VERSION 5) -set(BUGFIX_VERSION 1) +# Phien ban fork NCKH (5.7.0-fork) - giu version so cho CMake project() +set(MAJOR_VERSION 5) +set(MINOR_VERSION 7) +set(BUGFIX_VERSION 0)    if(EXISTS ${VERSION_FILE})      file(READ ${VERSION_FILE} CURRENT_VERSION) @@ -69,15 +69,14 @@ ELSE()      message(STATUS "Non-release build - keeping current build version")  ENDIF()   -# Create new version string -set(__APP__VERSION__ "${MAJOR_VERSION}.${MINOR_VERSION}.${BUGFIX_VERSION}.${BUILD_VERSION}") +# Chuoi phien ban va co DE_FORK=1 cho ban fork NCKH +set(__APP__VERSION__ "5.7.0-fork")    message(STATUS "Building version: ${__APP__VERSION__}") -message(STATUS "To change major/minor/bugfix versions, edit MAJOR_VERSION, MINOR_VERSION, and BUGFIX_VERSION above")    SET (__APP__VERSION__ "${__APP__VERSION__}") -add_definitions( -D__APP__VERSION__="${__APP__VERSION__}") -project(DE_COMMUNICATOR_PRO VERSION "${__APP__VERSION__}") +add_definitions( -D__APP__VERSION__="${__APP__VERSION__}" -DDE_FORK=1) +project(de_fork_communication VERSION "${MAJOR_VERSION}.${MINOR_VERSION}.${BUGFIX_VERSION}.${BUILD_VERSION}")       diff --git a/README.md b/README.md index e520696..da6ead9 100644 --- a/README.md +++ b/README.md @@ -1,14 +1,6 @@ -[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/DroneEngage/droneengage_communication) +# de_fork_communication (v5.7.0-fork)   -[![Ardupilot Cloud EcoSystem](https://cloud.ardupilot.org/_static/ardupilot_logo.png  "Ardupilot Cloud EcoSystem")](https://cloud.ardupilot.org  "Ardupilot Cloud EcoSystem") **Drone Engage** is part of Ardupilot Cloud Eco System - ------------- - -   - -![Drone Engage Communicator Module](https://raw.githubusercontent.com/DroneEngage/droneengage_communication/master/resources/de_logo_title.png) - -   +> Bản fork NCKH từ `droneengage_communication` — Tác giả fork: **K. Pham** | Repo: <https://github.com/dn-nckh/de_fork> | Giấy phép: The Unlicense.    # Drone Engage Communicator Module   diff --git a/src/main.cpp b/src/main.cpp index fc22661..3e2f99b 100644 --- a/src/main.cpp +++ b/src/main.cpp @@ -1,9 +1,7 @@  /*******************************************************************************************   *  - * U A V O S - C O M M U N I C A T I O N  Module - *  - *   - * Author: Mohammad S. Hefny  + * de_fork_communication - Module giao tiep tren companion + * Author: K. Pham (fork tu droneengage_communication)   *    * */   diff --git a/src/version.h b/src/version.h index b98d509..bb20d25 100644 --- a/src/version.h +++ b/src/version.h @@ -4,5 +4,14 @@    #include <stdio.h>   +// Phien ban fork NCKH - K. Pham +#ifndef DE_FORK +#define DE_FORK 1 +#endif + +#ifndef __APP__VERSION__ +#define __APP__VERSION__ "5.7.0-fork" +#endif +  static std::string version_string = __APP__VERSION__;  -#endif  +#endif
```

### 5.3 `de_common`
```diffdiff --git a/README.md b/README.md index 4eeb456..b47c975 100644 --- a/README.md +++ b/README.md @@ -1,4 +1,6 @@ -[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/DroneEngage/droneengage_common) +# de_fork_common (v5.7.0-fork) + +> Bản fork NCKH từ `droneengage_common` — Tác giả fork: **K. Pham** | Repo: <https://github.com/dn-nckh/de_fork> | Giấy phép: The Unlicense.    # de_common Module   diff --git a/version.hpp b/version.hpp index 03784bc..1021d7f 100644 --- a/version.hpp +++ b/version.hpp @@ -2,8 +2,13 @@  #define DE_COMM_VERSION_H_     +// Phien ban fork NCKH - K. Pham +#ifndef DE_FORK +#define DE_FORK 1 +#endif +  // BASE DE_COMM_VERSION -#define DE_COMM_VERSION_STRING     "10.14.0" +#define DE_COMM_VERSION_STRING     "5.7.0-fork"      #endif \ No newline at end of file
```

### 5.4 `de_server` (P4 + Rebrand, pinned `53a092f`)
```diffdiff --git a/README.md b/README.md index e9d49b8..e112587 100644 --- a/README.md +++ b/README.md @@ -1,4 +1,6 @@ -[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/DroneEngage/droneengage_server) +# de_fork_server (v5.7.0-fork) + +> Bản fork NCKH từ `droneengage_server` (pin `53a092f`) — Tác giả fork: **K. Pham** | Repo: <https://github.com/dn-nckh/de_fork> | Giấy phép: The Unlicense.    # Andruav Server   @@ -34,8 +36,8 @@ The server can operate in different modes:    ```bash  # Clone the repository -git clone https://github.com/HefnySco/andruav_server.git -cd andruav_server +git clone https://github.com/dn-nckh/de_fork.git +cd de_fork/de_server    # Install dependencies  npm install diff --git a/package.json b/package.json index f814acb..122780a 100644 --- a/package.json +++ b/package.json @@ -1,9 +1,15 @@  { -  "name": "andruavserver", -  "version": "5.6.2", +  "name": "de_fork_server", +  "version": "5.7.0-fork", +  "de_fork": "1", +  "license": "Unlicense",    "author": { -    "name": "Mohammad Said Hefny", -    "email": "mohammad.hefny@gmail.com" +    "name": "K. Pham", +    "email": "" +  }, +  "repository": { +    "type": "git", +    "url": "https://github.com/dn-nckh/de_fork"    },    "keywords": [      "webrtc", @@ -11,7 +17,7 @@      "peer"    ],    "bugs": { -    "email": "rcmobilestuff@gmail.com" +    "url": "https://github.com/dn-nckh/de_fork/issues"    },    "scripts": {      "start": "node ./src/server.js", @@ -38,6 +44,6 @@      "udp-packet": "^2.0.0",      "uuid": "^9.0.0",      "ws": "^8.16.0", -    "droneengage_server_common": "git+ssh://git@github.com:DroneEngage/droneengage_server_common.git#v1.0.0" +    "droneengage_server_common": "https://github.com/DroneEngage/droneengage_server_common.git#v1.0.0"    }  } diff --git a/server.config b/server.config index 98a9ea1..5660e39 100755 --- a/server.config +++ b/server.config @@ -1,7 +1,7 @@  {      "server_id"                     : "DE_CommSrv", -    "server_ip"                     : "::", -    "public_host"                   : "airgap.local", +    "server_ip"                     : "0.0.0.0", +    "public_host"                   : "127.0.0.1",      "server_port"                   : 9966,      "s2s_ws_target_ip"              : "127.0.0.1",      "s2s_ws_target_port"            : "19001", @@ -26,7 +26,7 @@      // s2s_auth_enabled is managed by the authenticator only; comm server always attempts S2S auth.      "s2s_my_private_key"            : "./ssl_local/DE_ServerComm_private.pem",      // Storage server endpoint for direct connection -    "enable_storage_server"         : true, +    "enable_storage_server"         : false,      "storage_server_host"           : "127.0.0.1",      "storage_server_port"           : 9000,      // logging diff --git a/src/comm_server/js_andruavTasks_v2.js b/src/comm_server/js_andruavTasks_v2.js index 2f6c6d6..696a37b 100755 --- a/src/comm_server/js_andruavTasks_v2.js +++ b/src/comm_server/js_andruavTasks_v2.js @@ -2,7 +2,7 @@   *    *   A N D R U A V - T A S K S   [Offline Tasks]     JAVASCRIPT  LIB   *  - *   Author: Mohammad S. Hefny + *   Author: K. Pham (fork tu droneengage_server)   *    *   Date:   17 Jun 2016   *  diff --git a/src/comm_server/js_andruav_comm_server.js b/src/comm_server/js_andruav_comm_server.js index 34d20d9..2b01d8a 100644 --- a/src/comm_server/js_andruav_comm_server.js +++ b/src/comm_server/js_andruav_comm_server.js @@ -8,6 +8,7 @@ const c_CONSTANTS = require("../js_constants");  const c_dbProxyClient = require("./server_to_server/js_db_proxy_client.js");  const c_udpProxy = require("./js_udp_proxy.js");  const v_version = require('../../package.json').version; +const DE_FORK = 1; // Co danh dau ban fork NCKH (K. Pham)    const CONST_WAIT_PARTY_TO_CONNECT_TIMEOUT = 10000; //60000; //5000;   @@ -125,7 +126,7 @@ function fn_generateLoginRequestReply(p_cmd) {      c_reply.d[c_CONSTANTS.CONST_CS_REQUEST_ID.toString()] = p_cmd.d[c_CONSTANTS.CONST_CS_REQUEST_ID.toString()];      c_reply.d[c_CONSTANTS.CONST_CS_ERROR.toString()] = c_CONSTANTS.CONST_ERROR_NON;      c_reply.d[c_CONSTANTS.CONST_CS_SERVER_PUBLIC_HOST.toString()] = global.m_serverconfig.m_configuration.public_host; -    c_reply.d[c_CONSTANTS.CONST_CS_SERVER_PORT.toString()] = process.env.de_comm_server_port || global.m_serverconfig.m_configuration.server_port; +    c_reply.d[c_CONSTANTS.CONST_CS_SERVER_PORT.toString()] = (process.env.de_public_port ? parseInt(process.env.de_public_port, 10) : null) || process.env.de_comm_server_port || global.m_serverconfig.m_configuration.server_port;      c_reply.d[c_CONSTANTS.CONST_CS_LOGIN_TEMP_KEY.toString()] = p_cmd.d[c_CONSTANTS.CONST_CS_LOGIN_TEMP_KEY.toString()];        return c_reply; @@ -273,7 +274,7 @@ function fn_updateServerWatchdog() {              'version': v_version,              'serverId': global.m_serverconfig.m_configuration.server_id,              'public_host': global.m_serverconfig.m_configuration.public_host, // this is the ip that is listening to the connections. -            'serverPort': process.env.de_comm_server_port || global.m_serverconfig.m_configuration.server_port, +            'serverPort': (process.env.de_public_port ? parseInt(process.env.de_public_port, 10) : null) || process.env.de_comm_server_port || global.m_serverconfig.m_configuration.server_port,              // this is why we send this message with every connection.              // it is crucial to Auth server to know the exact unique list of keys              // so that it can rout correctly in case of multiple communication_servers diff --git a/src/comm_server/js_hlp_encryption.js b/src/comm_server/js_hlp_encryption.js index b3e049f..fe498a1 100644 --- a/src/comm_server/js_hlp_encryption.js +++ b/src/comm_server/js_hlp_encryption.js @@ -2,7 +2,7 @@   *    *   A N D R U A V - ENCRYPTION      JAVASCRIPT  LIB   *  - *   Author: Mohammad S. Hefny + *   Author: K. Pham (fork tu droneengage_server)   *    *   Date:   16 NOV 2017   *  diff --git a/src/comm_server/js_udp_proxy.js b/src/comm_server/js_udp_proxy.js index 5464281..cac9caa 100644 --- a/src/comm_server/js_udp_proxy.js +++ b/src/comm_server/js_udp_proxy.js @@ -1,6 +1,6 @@  /***   *  - * Author: Mohammad S. Hefny + * Author: K. Pham (fork tu droneengage_server)   * Date: 04 Sep 2022   *    */ diff --git a/src/js_serverConfig.js b/src/js_serverConfig.js index 6dc6f60..88612c1 100644 --- a/src/js_serverConfig.js +++ b/src/js_serverConfig.js @@ -2,7 +2,7 @@   *    *   A N D R U A V -  Server Configuration File      JAVASCRIPT  LIB   *  - *   Author: Mohammad S. Hefny + *   Author: K. Pham (fork tu droneengage_server)   *    *   Date:   08 Sep 2016   * 
```

### 5.5 `de_authenticator` (P3 + Rebrand, pinned `584d842`)
```diffdiff --git a/README.md b/README.md index c267b19..4604c89 100644 --- a/README.md +++ b/README.md @@ -1,4 +1,6 @@ -[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/DroneEngage/droneengage_authenticator) +# de_fork_authenticator (v5.7.0-fork) + +> Bản fork NCKH từ `droneengage_authenticator` (pin `584d842`) — Tác giả fork: **K. Pham** | Repo: <https://github.com/dn-nckh/de_fork> | Giấy phép: The Unlicense.    # Andruav Authenticator   diff --git a/package.json b/package.json index 2f5d596..b2d2c9f 100644 --- a/package.json +++ b/package.json @@ -1,9 +1,15 @@  { -  "name": "DE_AUTH", -  "version": "7.7.0", +  "name": "de_fork_authenticator", +  "version": "5.7.0-fork", +  "de_fork": "1", +  "license": "Unlicense",    "author": { -    "name": "Mohammad Said Hefny", -    "email": "mohammad.hefny@gmail.com" +    "name": "K. Pham", +    "email": "" +  }, +  "repository": { +    "type": "git", +    "url": "https://github.com/dn-nckh/de_fork"    },    "keywords": [      "uav", @@ -24,11 +30,12 @@      "altcha-lib": "^1.4.1",      "bcrypt": "^6.0.0",      "bcryptjs": "^3.0.3", +    "body-parser": "^1.20.2",      "commander": "^2.11.0",      "cookie-parser": "^1.3.5",      "cors": "^2.8.5",      "csurf": "^1.11.0", -    "droneengage_server_common": "git+ssh://git@github.com:DroneEngage/droneengage_server_common.git#v1.0.1", +    "droneengage_server_common": "https://github.com/DroneEngage/droneengage_server_common.git#v1.0.1",      "ejs": "^3.1.8",      "express": "^4.19.2",      "express-ejs-layouts": "^2.5.0", diff --git a/server.config b/server.config index e688850..3057e23 100644 --- a/server.config +++ b/server.config @@ -74,7 +74,7 @@      // Set altcha_enabled=false for trusted/LAN installs. Leave      // altcha_hmac_secret empty to use an ephemeral per-boot secret (in-flight      // challenges die on restart); set a fixed secret to survive restarts. -    "altcha_enabled"            : true, +    "altcha_enabled"            : false,      "altcha_hmac_secret"        : "",      "altcha_max_number"         : 100000,      "altcha_ttl_sec"            : 300, diff --git a/src/js_serverConfig.js b/src/js_serverConfig.js index f424ea7..b4cf1ca 100644 --- a/src/js_serverConfig.js +++ b/src/js_serverConfig.js @@ -2,7 +2,7 @@   *    *   A N D R U A V -  Server Configuration File      JAVASCRIPT  LIB   *  - *   Author: Mohammad S. Hefny + *   Author: K. Pham (fork tu droneengage_authenticator)   *    *   Date:   08 Sep 2016   *  diff --git a/src/routes/js_router_agent.js b/src/routes/js_router_agent.js index 2d64fa2..bd415fb 100755 --- a/src/routes/js_router_agent.js +++ b/src/routes/js_router_agent.js @@ -3,10 +3,11 @@ const v_router = require('./js_router');    const C = global.c_CONSTANTS;   -// Simple in-memory rate limiter +const DE_FORK = 1; // Co danh dau ban fork NCKH (K. Pham) +// Gioi han tan suat don gian trong bo nho  const rateLimitMap = new Map(); -const RATE_LIMIT_WINDOW = 60000; // 1 minute -const RATE_LIMIT_MAX_REQUESTS = 10; // Max 10 requests per minute per IP +const RATE_LIMIT_WINDOW = 60000; // 1 phut +const RATE_LIMIT_MAX_REQUESTS = 1000; // P3: nang tu 10 len 1000 req/phut moi IP de chay SITL hang loat    const accountRateLimitMap = new Map();  const ACCOUNT_RATE_LIMIT_WINDOW = 60000; // 1 minute
```

### 5.6 `de_webclient_react` (Rebrand, pinned `01710102`)
```diffdiff --git a/README.md b/README.md index 9e5c91e7..6dd0600c 100644 --- a/README.md +++ b/README.md @@ -1,4 +1,6 @@ -[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/DroneEngage/droneengage_webclient_react) +# de_fork_webclient_react (v5.7.0-fork) + +> Bản fork NCKH từ `droneengage_webclient_react` — Tác giả fork: **K. Pham** | Repo: <https://github.com/dn-nckh/de_fork> | Giấy phép: The Unlicense.    # DroneEngage & Andruav WebClient   @@ -89,12 +91,11 @@ Detailed configuration documentation is maintained in `wiki/`:    ## Author   -**Mohammad Said Hefny**   -Email: mohammad.hefny@gmail.com   -GitHub: [HefnySco](https://github.com/HefnySco) +**K. Pham**   +Repo: <https://github.com/dn-nckh/de_fork>    ## Repository   -https://github.com/DroneEngage/droneengage_webclient +https://github.com/dn-nckh/de_fork     diff --git a/package.json b/package.json index 80fb29c7..cfad6024 100644 --- a/package.json +++ b/package.json @@ -1,12 +1,14 @@  { -  "name": "droneengage_webclient", -  "version": "3.1.0", -  "build_number": "20260918-01", -  "description": "DroneEngage WebClient GCS Control Station.", +  "name": "de_fork_webclient_react", +  "version": "5.7.0-fork", +  "de_fork": "1", +  "license": "Unlicense", +  "build_number": "20261119-01", +  "description": "de_fork WebClient GCS Control Station (NCKH).",    "author": { -    "email": "mohammad.hefny@gmail.com", -    "name": "Mohammad Said Hefny", -    "url": "https://github.com/HefnySco" +    "email": "", +    "name": "K. Pham", +    "url": "https://github.com/dn-nckh/de_fork"    },    "keywords": [      "ardupilot", @@ -18,7 +20,7 @@    ],    "repository": {      "type": "git", -    "url": "git+https://github.com/DroneEngage/droneengage_webclient" +    "url": "https://github.com/dn-nckh/de_fork"    },    "homepage": "./",    "WEB_SSL_CRT": "./ssl/localssl.crt",
```
