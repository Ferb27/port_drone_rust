import { useDroneStore } from '../../../store/droneStore'
import { protocolManager } from '../../../core/ProtocolManager'

export function ActionPanel() {
  const drones = useDroneStore((state) => state.drones)
  const selectedDroneId = useDroneStore((state) => state.selectedDroneId)
  const settings = useDroneStore((state) => state.settings)
  const updateSettings = useDroneStore((state) => state.updateSettings)
  const isSimulating = Object.values(drones).some(d => d.isSimulated)
  
  const drone = selectedDroneId ? drones[selectedDroneId] : null

  return (
    <div className="w-80 flex flex-col gap-4">
      
      {/* Settings Panel */}
      <div className="bg-slate-900/80 backdrop-blur-md rounded-2xl border border-slate-700/50 shadow-2xl p-4 flex flex-col gap-3">
        <div className="text-xs font-bold text-slate-400 border-b border-slate-700/50 pb-2 flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          MISSION SETTINGS
        </div>
        
        <div className="flex gap-4">
          <div className="flex-1 flex flex-col gap-1">
            <span className="text-[10px] text-slate-400 font-bold">ALT STEP (m)</span>
            <input 
              type="number" 
              className="bg-slate-800 border border-slate-600 rounded px-2 py-1 text-xs text-white outline-none focus:border-cyan-500 transition-colors"
              value={settings.altStep}
              onChange={(e) => updateSettings({ altStep: Number(e.target.value) })}
            />
          </div>
          <div className="flex-1 flex flex-col gap-1">
            <span className="text-[10px] text-slate-400 font-bold">RADIUS (m)</span>
            <input 
              type="number" 
              className="bg-slate-800 border border-slate-600 rounded px-2 py-1 text-xs text-white outline-none focus:border-cyan-500 transition-colors"
              value={settings.radius}
              onChange={(e) => updateSettings({ radius: Number(e.target.value) })}
            />
          </div>
        </div>

        <div className="flex flex-col gap-1 mt-1">
          <div className="flex justify-between text-[10px] text-slate-400 font-bold">
            <span>SIM SPEED</span>
            <span className="text-cyan-400">{settings.simSpeed}x</span>
          </div>
          <input 
            type="range" 
            min="0.5" 
            max="3" 
            step="0.5"
            className="w-full h-1 bg-slate-700 rounded-lg appearance-none cursor-pointer accent-cyan-500"
            value={settings.simSpeed}
            onChange={(e) => updateSettings({ simSpeed: Number(e.target.value) })}
          />
        </div>
      </div>

      {/* Individual Controls */}
      {drone && (
        <div className="bg-slate-900/70 backdrop-blur-md rounded-2xl border border-slate-700/50 shadow-2xl p-4 flex flex-col gap-4">
          <div className="text-xs font-bold text-cyan-400 border-b border-slate-700/50 pb-2">
            INDIVIDUAL CONTROL: <span className="text-white">{drone.partyID}</span>
          </div>
          
          <div className="flex gap-2">
            <select 
              className="flex-1 bg-slate-800 border border-slate-600 rounded px-2 py-1 text-xs text-white"
              value={drone.flightMode || 'Stabilize'}
              onChange={(e) => protocolManager.setFlightMode(drone.partyID, e.target.value)}
            >
              <option value="Stabilize">STABILIZE</option>
              <option value="Loiter">LOITER</option>
              <option value="Guided">GUIDED</option>
              <option value="Auto">AUTO</option>
            </select>

            <button 
              onClick={() => drone.isArmed ? protocolManager.disarmDrone(drone.partyID) : protocolManager.armDrone(drone.partyID)}
              className={`px-3 py-1 rounded text-xs font-bold border transition-colors ${
                drone.isArmed 
                  ? 'bg-red-500/20 text-red-500 border-red-500/50 hover:bg-red-500/40' 
                  : 'bg-emerald-500/20 text-emerald-500 border-emerald-500/50 hover:bg-emerald-500/40'
              }`}
            >
              {drone.isArmed ? 'DISARM' : 'ARM'}
            </button>
          </div>

          <div className="flex gap-2">
            <button 
              onClick={() => protocolManager.takeoffDrone(drone.partyID)}
              className="flex-1 bg-slate-800 hover:bg-slate-700 text-emerald-400 border border-emerald-500/30 rounded py-2 text-xs font-bold transition-colors"
            >
              TAKEOFF
            </button>
            <button 
              onClick={() => protocolManager.rtlDrone(drone.partyID)}
              className="flex-1 bg-slate-800 hover:bg-slate-700 text-amber-500 border border-amber-500/30 rounded py-2 text-xs font-bold transition-colors"
            >
              RTL
            </button>
          </div>

          {/* MISION CONTROL FOR INDIVIDUAL DRONE */}
          {drone.missionWaypoints && drone.missionWaypoints.length > 0 && (
            <div className="bg-slate-800/80 rounded-xl p-3 border border-cyan-500/30 mt-2">
              <div className="flex justify-between items-center mb-2">
                <span className="text-[10px] font-bold text-cyan-400">MISSION WAYPOINTS: {drone.missionWaypoints.length}</span>
                <button 
                  onClick={() => useDroneStore.getState().clearWaypoints(drone.partyID)}
                  className="text-[10px] text-red-400 hover:text-red-300"
                >
                  CLEAR
                </button>
              </div>
              {useDroneStore.getState().leaderId !== null && useDroneStore.getState().leaderId !== drone.partyID ? (
                <div className="w-full bg-slate-700 text-slate-400 rounded py-2 text-xs font-bold text-center italic cursor-not-allowed">
                  FOLLOWER MODE LOCKED
                </div>
              ) : (
                <button 
                  onClick={() => protocolManager.startMission(drone.partyID)}
                  className="w-full bg-cyan-600 hover:bg-cyan-500 text-white rounded py-2 text-xs font-bold shadow-lg shadow-cyan-500/20 transition-colors"
                >
                  START MISSION
                </button>
              )}
            </div>
          )}
        </div>
      )}

      {/* Swarm Commands */}
      {isSimulating && (
        <div className="bg-purple-900/40 backdrop-blur-md rounded-2xl border border-purple-500/30 shadow-[0_0_20px_rgba(168,85,247,0.15)] p-4 flex flex-col gap-3">
          <div className="flex items-center gap-2 border-b border-purple-500/30 pb-2">
            <div className="w-2 h-2 rounded-full bg-purple-400 animate-pulse"></div>
            <div className="text-xs font-bold text-purple-300 tracking-widest">SWARM COMMANDS</div>
          </div>
          
          <div className="grid grid-cols-2 gap-2">
             <button onClick={() => protocolManager.armAll()} className="bg-orange-500 hover:bg-orange-600 text-white py-2 rounded-lg font-bold text-xs shadow-lg transition-colors">
               ARM ALL
             </button>
             <button onClick={() => protocolManager.takeoffAll()} className="bg-emerald-500 hover:bg-emerald-600 text-white py-2 rounded-lg font-bold text-xs shadow-lg transition-colors">
               TAKEOFF ALL
             </button>
             <button onClick={() => protocolManager.landAll()} className="bg-sky-500 hover:bg-sky-600 text-white py-2 rounded-lg font-bold text-xs shadow-lg transition-colors">
               LAND ALL
             </button>
             <button onClick={() => protocolManager.rtlAll()} className="bg-red-500 hover:bg-red-600 text-white py-2 rounded-lg font-bold text-xs shadow-lg transition-colors">
               RTL ALL
             </button>
          </div>
        </div>
      )}

    </div>
  )
}
