/*
 * Purpose: Quản lý trạng thái băng thông và cấu hình Telemetry (UDP Proxy, Rate Control).
 * Dependencies: zustand
 * Author: Antigravity
 * Date: 2026-06-08
 */

import { create } from 'zustand'

interface TelemetryState {
  /** Mức độ băng thông: 0 (Full) tới 3 (Lowest) */
  telemetryLevel: number
  /** Cờ dừng khẩn cấp luồng UDP Proxy */
  isUdpPaused: boolean
  /** Cổng cố định để forward telemetry qua UDP (cho Mission Planner) */
  udpProxyPort: number
}

interface TelemetryActions {
  setTelemetryLevel: (level: number) => void
  setUdpPaused: (isPaused: boolean) => void
  setUdpProxyPort: (port: number) => void
}

export const useTelemetryStore = create<TelemetryState & TelemetryActions>((set) => ({
  telemetryLevel: 0,
  isUdpPaused: false,
  udpProxyPort: 15412,

  setTelemetryLevel: (level) => set({ telemetryLevel: level }),
  setUdpPaused: (isPaused) => set({ isUdpPaused: isPaused }),
  setUdpProxyPort: (port) => set({ udpProxyPort: port }),
}))
