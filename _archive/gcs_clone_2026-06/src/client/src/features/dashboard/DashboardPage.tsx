import { useEffect } from 'react'
import { useAuthStore } from '../../store/authStore'
import { useNavigate } from 'react-router-dom'
import { protocolManager } from '../../core/ProtocolManager'
import { DroneMap } from './DroneMap'

import { TopToolbar } from './components/TopToolbar'
import { FloatingHUD } from './components/FloatingHUD'
import { ActionPanel } from './components/ActionPanel'
import { TelemetryControlPanel } from '../telemetry/TelemetryControlPanel'
import { DebugConsole } from './components/DebugConsole'

export function DashboardPage() {
  const { clearSession } = useAuthStore()
  const navigate = useNavigate()

  useEffect(() => {
    protocolManager.connect()
    return () => {
      protocolManager.disconnect()
    }
  }, [])

  function handleLogout() {
    protocolManager.disconnect()
    clearSession()
    navigate('/login', { replace: true })
  }

  return (
    <main className="relative h-screen w-screen overflow-hidden bg-black font-sans text-slate-100">
      {/* Lớp dưới cùng: Bản đồ Fullscreen */}
      <DroneMap />

      {/* Lớp bên trên: Các UI điều khiển nổi (Glassmorphism) */}
      <div className="absolute inset-0 pointer-events-none z-[500] flex flex-col justify-between">
        
        {/* Thanh công cụ trên cùng */}
        <div className="pointer-events-auto">
          <TopToolbar onLogout={handleLogout} />
        </div>

        {/* Khung dưới (HUD trái, Action phải) */}
        <div className="flex justify-between items-end p-6 pointer-events-none">
          <div className="pointer-events-auto">
            <FloatingHUD />
          </div>
          <div className="flex flex-col gap-4 items-end pointer-events-auto">
            <TelemetryControlPanel />
            <ActionPanel />
          </div>
        </div>

        <DebugConsole />
      </div>
    </main>
  )
}
