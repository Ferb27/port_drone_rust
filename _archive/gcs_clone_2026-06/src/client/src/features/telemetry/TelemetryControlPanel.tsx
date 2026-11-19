import React from 'react'
import { useTelemetryStore } from '../../store/telemetryStore'
import { useDroneStore } from '../../store/droneStore'
import { protocolManager } from '../../core/ProtocolManager'

export const TelemetryControlPanel: React.FC = () => {
  const { telemetryLevel, isUdpPaused, udpProxyPort, setTelemetryLevel, setUdpPaused } = useTelemetryStore()
  // Lấy ra Drone đang được chọn, nếu không chọn mặc định áp dụng cho tất cả (_AGN_)
  const selectedDroneId = useDroneStore((state) => state.selectedDroneId)
  const targetId = selectedDroneId || '_AGN_'

  const handleLevelChange = (level: number) => {
    setTelemetryLevel(level)
    protocolManager.setTelemetryLevel(targetId, level)
  }

  const handleTogglePause = () => {
    const newState = !isUdpPaused
    setUdpPaused(newState)
    protocolManager.toggleUdpTelemetry(targetId, newState)
  }

  const handleCopyPort = () => {
    navigator.clipboard.writeText(udpProxyPort.toString())
  }

  return (
    <div className="bg-white/10 backdrop-blur-md border border-white/20 p-5 rounded-2xl shadow-2xl max-w-sm w-full text-white font-sans">
      <div className="flex justify-between items-center mb-6">
        <h3 className="text-xl font-semibold tracking-wide">
          Telemetry & Safety
        </h3>
        {targetId === '_AGN_' ? (
          <span className="bg-blue-500/30 text-blue-200 text-xs px-2 py-1 rounded-full border border-blue-500/50">ALL SWARM</span>
        ) : (
          <span className="bg-purple-500/30 text-purple-200 text-xs px-2 py-1 rounded-full border border-purple-500/50">{targetId}</span>
        )}
      </div>

      {/* Mức độ Telemetry */}
      <div className="mb-6">
        <div className="flex justify-between items-center mb-2">
          <label className="text-sm text-gray-300">Rate Optimization Level</label>
          <span className="text-xs font-mono font-bold text-gray-400">LVL {telemetryLevel}</span>
        </div>
        <div className="flex space-x-2 bg-black/30 p-1 rounded-lg">
          {[0, 1, 2, 3].map((lvl) => (
            <button
              key={lvl}
              onClick={() => handleLevelChange(lvl)}
              className={`flex-1 py-1.5 rounded-md text-sm font-medium transition-all duration-300 ${
                telemetryLevel === lvl
                  ? lvl === 0 
                    ? 'bg-red-500/80 text-white shadow-[0_0_10px_rgba(239,68,68,0.5)]' // Level 0: Cảnh báo đỏ (Tốn băng thông)
                    : 'bg-indigo-500/80 text-white shadow-[0_0_10px_rgba(99,102,241,0.5)]' // Level 1-3: Xanh nhạt tới đậm
                  : 'text-gray-400 hover:text-white hover:bg-white/5'
              }`}
            >
              L{lvl}
            </button>
          ))}
        </div>
        <p className="text-[10px] mt-2 text-gray-400 italic">
          {telemetryLevel === 0 && 'Warning: L0 sends full MAVLink rate. High bandwidth usage!'}
          {telemetryLevel === 1 && 'L1: Light optimization for stable 4G.'}
          {telemetryLevel === 2 && 'L2: Moderate optimization. Recommended for Swarm.'}
          {telemetryLevel === 3 && 'L3: Maximum optimization. Lowest bandwidth.'}
        </p>
      </div>

      {/* Nút dừng khẩn cấp */}
      <div className="mb-6">
        <button
          onClick={handleTogglePause}
          className={`w-full flex items-center justify-center py-3 rounded-xl font-bold uppercase tracking-wider transition-all duration-300 ${
            isUdpPaused
              ? 'bg-red-600 hover:bg-red-500 text-white shadow-[0_0_15px_rgba(220,38,38,0.7)] animate-pulse'
              : 'bg-green-600 hover:bg-green-500 text-white shadow-[0_0_15px_rgba(22,163,74,0.5)]'
          }`}
        >
          {isUdpPaused ? '⚠ UDP PAUSED' : 'UDP STREAM ACTIVE'}
        </button>
        <p className="text-[10px] mt-2 text-center text-gray-400">
          Emergency power button to pause web UDP stream.
        </p>
      </div>

      {/* UDPCI Info Box */}
      <div className="bg-black/40 rounded-xl p-3 border border-white/5">
        <label className="text-xs text-gray-400 uppercase tracking-wide block mb-1">UDPCI Proxy Server</label>
        <div className="flex items-center justify-between bg-black/50 p-2 rounded-lg border border-white/10">
          <div className="font-mono text-sm text-green-400">
            udp_proxy_fixed_port: {udpProxyPort}
          </div>
          <button 
            onClick={handleCopyPort}
            className="text-gray-400 hover:text-white bg-white/5 hover:bg-white/10 px-2 py-1 rounded text-xs transition-colors"
          >
            Copy
          </button>
        </div>
        <p className="text-[10px] mt-2 text-gray-500 leading-tight">
          Connect Mission Planner via UDP to `127.0.0.1:{udpProxyPort}` for local telemetry forwarding.
        </p>
      </div>
    </div>
  )
}
