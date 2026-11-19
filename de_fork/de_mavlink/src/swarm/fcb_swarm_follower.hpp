
#ifndef FCB_SWARM_FOLLOWER_H_
#define FCB_SWARM_FOLLOWER_H_

#include <iostream>
#include <mavlink_sdk.h>
#include "fcb_swarm_manager.hpp"

// S1/B1 — velocity feed-forward cho follower (params DE_FF_ENABLE / DE_FF_ALPHA /
// DE_FF_MAX_TAU_S), default ON. Giữ N=20 mẫu position gần nhất từ leader (6503),
// ước vel bằng LPF α trên delta/t, clamp ±12 m/s mỗi trục, rồi predict target =
// pos + v*τ_est trước khi push qua gotoGuidedPoint.

#define DE_FF_N_SAMPLES 20              // ring buffer: N=20 mẫu position gần nhất từ leader (6503)
#define DE_FF_MIN_SAMPLES 3             // < 3 mẫu -> fallback LPF cũ B0 (target alpha 0.3)
#define DE_FF_V_CLAMP_M 12.0            // clamp velocity ±12 m/s / trục sau LPF
#define DE_FF_TURN_DELTA_M 4.0          // |Δv| L∞ > 4 m/s trong window 1.5s -> reset buffer vel + log warning
#define DE_FF_TURN_WINDOW_MS 1500ULL    // cửa sổ Δv [ms, time_boot_ms]
#define DE_FF_DT_MAX_MS 500              // delta/t: bỏ mẫu mới nếu dt không nằm (0, 500] ms
#define DE_FF_ALPHA_DEFAULT 0.2          // param DE_FF_ALPHA
#define DE_FF_MAX_TAU_S_DEFAULT 8.0      // param DE_FF_MAX_TAU_S [s]

typedef struct
{
    uint32_t t_ms;   // time_boot_ms từ leader (ms)
    double lat, lon; // deg (đã /1e7)
    int32_t alt;     // relative_alt, cm

} FFPosSample;


namespace de
{
namespace fcb
{
namespace swarm
{
    class CSwarmFollower
    {
        public:

            
            static CSwarmFollower& getInstance()
            {
                static CSwarmFollower instance;

                return instance;
            }

            CSwarmFollower(CSwarmFollower const&)            = delete;
            void operator=(CSwarmFollower const&)            = delete;

        
            // Note: Scott Meyers mentions in his Effective Modern
            //       C++ book, that deleted functions should generally
            //       be public as it results in better error messages
            //       due to the compilers behavior to check accessibility
            //       before deleted status

        private:

            CSwarmFollower()
            {
                
            }

            
        public:
            
            ~CSwarmFollower ()
            {

            }

        public:

            void handle_leader_traffic(const std::string & leader_sender, const char * full_message, const int & full_message_length, const Json_de& message_cmd = Json_de());
            void checkLeaderTimeout();

        public: 

            void setMinVerticalDistance (const int min_vertical_distance)
            {
                m_min_vertical_distance = min_vertical_distance;
            };
            void setMinHorizontalDistance (const int min_horizontal_distance)
            {
                m_min_horizontal_distance = min_horizontal_distance;
            };
            int getMinVerticalDistance ()
            {
                return m_min_vertical_distance;
            };
            int getMinHorizontalDistance ()
            {
                return m_min_horizontal_distance;
            };

        public: 

            // S1/B1 params (GCS set qua FollowHim_Request: ff / fa / ft), default ON
            void setFFEnabled(const bool enabled)
            {
                m_ff_enabled = (enabled ? true : false);
            }
            void setFFAlpha(double alpha)   // param DE_FF_ALPHA, default 0.2
            {
                if (alpha < 0.001) alpha = 0.001;
                else if (alpha > 0.95) alpha = 0.95;
                m_ff_alpha = alpha;
            }
            void setFFMaxTauS(double seconds)   // param DE_FF_MAX_TAU_S, default 8 [s]
            {
                if (seconds < 0.1) seconds = 0.1;
                else if (seconds > 32.0) seconds = 32.0;
                m_ff_max_tau_s = seconds;
            }

        private:

            void updateFollower();
            void updateFollowerInThreadFormation();
            void updateFollowerInArrowFormation(const bool is_dynamic);
            void updateFollowerInVectorFormation();
            bool resolveFormationBearing(const bool is_dynamic, double& out_bearing);

            // S1/B1: velocity feed-forward
            void sampleLeaderFF(const mavlink_global_position_int_t& g);   // mỗi 6503 GPOS mới
            bool applyVelocityFeedForward(double& target_lat, double& target_lon) const; // predict trước gotoGuidedPoint
            void resetFF();                                                  // zero toàn bộ state FF

        private:
            
            mavlink_global_position_int_t m_leader_gpos_new;
            mavlink_global_position_int_t m_leader_gpos_old;
            u_int64_t m_leader_last_access;

            // Bearing latching for arrow formation — prevents violent swings
            double m_last_leader_bearing = 0.0;
            bool m_has_last_bearing = false;

            // Target position smoothing — low-pass filter on goto target
            double m_smoothed_target_lat = 0.0;
            double m_smoothed_target_lon = 0.0;
            bool m_has_smoothed_target = false;

            // Leader attitude (parsed from ATTITUDE message)
            float m_leader_yaw = 0.0f;
            float m_leader_yawspeed = 0.0f;

            // Dynamic side assignment for arrow formation (hysteresis)
            int m_last_side = 0;  // -1 = left, +1 = right, 0 = unassigned

            // Target jump detection — skip chasing target when it jumps toward follower
            double m_prev_raw_target_lat = 0.0;
            double m_prev_raw_target_lon = 0.0;
            bool m_has_prev_raw_target = false;
            int m_consecutive_skips = 0;

            // P1: stale message filtering (sequence) and 5s leader timeout tracking
            uint32_t m_last_leader_seq = 0;
            bool m_has_leader_seq = false;
            uint64_t m_last_leader_rx_usec = 0;
            bool m_leader_rx_active = false;

            // S1/B1: velocity feed-forward (default ON, params DE_FF_*)
            bool   m_ff_enabled = true;
            double m_ff_alpha = 0.2;                  // param DE_FF_ALPHA [0..1]
            double m_ff_max_tau_s = 8.0;              // param DE_FF_MAX_TAU_S [s]

            FFPosSample m_ff_pos[DE_FF_N_SAMPLES];   // ring: N mẫu gần nhất (head = slot ghi tiếp theo)
            int          m_ff_head  = 0;
            int          m_ff_count = 0;              // 0..N samples đã push

            bool        m_ff_have_clock_offset = false;  // pin offset ở sample đầu: now - t_boot*1e3
            long long   m_ff_clock_offset_us = 0;

            double       m_ff_vx = 0.0, m_ff_vy = 0.0, m_ff_vz = 0.0; // LPF'd velocity [m/s], clamp ±12
            bool         m_ff_have_vel = false;

            uint32_t     m_ff_turn_start_ms = 0;        // window start (t_ms mẫu mở cửa sổ / sau reset)

        private:
            
            int m_min_vertical_distance = KNODE_LENGTH;
            int m_min_horizontal_distance = KNODE_LENGTH;
    };
}
}
}
#endif
