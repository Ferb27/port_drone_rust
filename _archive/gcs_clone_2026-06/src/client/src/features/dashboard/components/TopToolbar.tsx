import { useDroneStore } from '../../../store/droneStore'
import { protocolManager } from '../../../core/ProtocolManager'

interface Props {
  onLogout: () => void
}

export function TopToolbar({ onLogout }: Props) {
  const drones = useDroneStore((state) => state.drones)
  const selectedDroneId = useDroneStore((state) => state.selectedDroneId)
  const leaderId = useDroneStore((state) => state.leaderId)
  const setLeader = useDroneStore((state) => state.setLeader)
  const setSelectedDrone = useDroneStore((state) => state.setSelectedDrone)
  const swarmFormation = useDroneStore((state) => state.swarmFormation)
  const setSwarmFormation = useDroneStore((state) => state.setSwarmFormation)
  const isFlockMode = useDroneStore((state) => state.isFlockMode)
  const setFlockMode = useDroneStore((state) => state.setFlockMode)

  const droneList = Object.values(drones)
  const partyIds = droneList.map(d => d.partyID)
  const isSimulating = droneList.some(d => d.isSimulated)

  return (
    <div className="flex items-center justify-between px-6 py-3 bg-slate-900/60 backdrop-blur-md border-b border-slate-700/50 shadow-lg">
      
      {/* Left: Branding */}
      <div className="flex items-center gap-4">
        <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-cyan-400 to-blue-600 shadow-lg shadow-blue-500/30">
          <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
          </svg>
        </div>
        <div>
          <h1 className="text-sm font-bold tracking-wider text-white">QG-CLONE</h1>
          <p className="text-[10px] text-cyan-400 font-mono tracking-widest">ADVANCED SWARM GCS</p>
        </div>
      </div>

      {/* Center: Drone Selector & Leader Setup */}
      <div className="flex items-center gap-6 bg-slate-800/50 rounded-full px-6 py-1 border border-slate-600/30">
        <div className="flex items-center gap-2">
          <span className="text-xs text-slate-400 font-bold">SELECT DRONE:</span>
          {droneList.length === 0 ? (
            <span className="text-xs text-slate-500 italic">No drones connected</span>
          ) : (
            <div className="flex gap-1">
              {droneList.map(d => (
                <button
                  key={d.partyID}
                  onClick={() => setSelectedDrone(d.partyID)}
                  className={`px-3 py-1 rounded-full text-xs font-bold transition-all ${
                    selectedDroneId === d.partyID 
                      ? 'bg-cyan-500 text-white shadow-[0_0_10px_rgba(6,182,212,0.5)]' 
                      : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                  }`}
                >
                  {d.partyID.replace('sim_drone_', 'D')}
                </button>
              ))}
            </div>
          )}
        </div>

        <div className="w-[1px] h-6 bg-slate-600/50"></div>

        <div className="flex items-center gap-2">
          <button
            onClick={() => setFlockMode(!isFlockMode)}
            className={`px-3 py-1 rounded-full text-xs font-bold transition-all border ${
              isFlockMode 
                ? 'bg-amber-500/20 text-amber-400 border-amber-500/50 shadow-[0_0_10px_rgba(245,158,11,0.3)]' 
                : 'bg-slate-700/50 text-slate-400 border-slate-600/30 hover:bg-slate-600'
            }`}
            title="Enable Boids Organic Flocking (No Leader needed)"
          >
            FLOCK MODE
          </button>
        </div>

        {!isFlockMode && (
          <>
            <div className="w-[1px] h-6 bg-slate-600/50"></div>
            <div className="flex items-center gap-2">
              <span className="text-xs text-purple-400 font-bold">LEADER:</span>
              <select 
                className="bg-purple-900/40 text-purple-200 text-xs outline-none border border-purple-500/30 rounded-full px-3 py-1 font-bold focus:ring-1 focus:ring-purple-500 appearance-none"
                value={leaderId || ''}
                onChange={(e) => setLeader(e.target.value || null)}
                disabled={partyIds.length === 0}
              >
                <option value="">-- None --</option>
                {partyIds.map(id => <option key={id} value={id}>{id.replace('sim_drone_', 'D')}</option>)}
              </select>
            </div>

            <div className="w-[1px] h-6 bg-slate-600/50"></div>

            <div className="flex items-center gap-2">
              <span className="text-xs text-blue-400 font-bold">FORM:</span>
              <select 
                className="bg-blue-900/40 text-blue-200 text-xs outline-none border border-blue-500/30 rounded-full px-3 py-1 font-bold focus:ring-1 focus:ring-blue-500 appearance-none"
                value={swarmFormation}
                onChange={(e) => setSwarmFormation(e.target.value as any)}
              >
                <option value="V-Shape">V-Shape</option>
                <option value="Line">Line</option>
                <option value="Column">Column</option>
                <option value="Circle">Circle</option>
              </select>
            </div>
          </>
        )}
      </div>

      {/* Right: Actions */}
      <div className="flex items-center gap-4">
        <button
          onClick={() => protocolManager.toggleSimulator()}
          className={`px-4 py-1.5 rounded-full text-xs font-bold tracking-wider transition-all border ${
            isSimulating 
              ? 'bg-red-500/20 text-red-400 border-red-500/50 hover:bg-red-500/40' 
              : 'bg-emerald-500/20 text-emerald-400 border-emerald-500/50 hover:bg-emerald-500/40'
          }`}
        >
          {isSimulating ? 'STOP SIMULATOR' : 'START SWARM SIM'}
        </button>

        <button
          onClick={onLogout}
          className="rounded-full bg-slate-800 px-4 py-1.5 text-xs font-bold text-slate-300 hover:bg-slate-700 hover:text-white transition-colors"
        >
          LOGOUT
        </button>
      </div>

    </div>
  )
}
