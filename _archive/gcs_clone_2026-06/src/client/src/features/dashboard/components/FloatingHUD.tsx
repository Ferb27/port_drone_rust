import { useDroneStore } from '../../../store/droneStore'

export function FloatingHUD() {
  const drones = useDroneStore((state) => state.drones)
  const selectedDroneId = useDroneStore((state) => state.selectedDroneId)
  
  const drone = selectedDroneId ? drones[selectedDroneId] : null

  if (!drone) return <div className="w-64"></div> // Placeholder để giữ layout

  const heading = drone.heading || 0
  const isArmed = drone.isArmed

  return (
    <div className="w-72 bg-slate-900/70 backdrop-blur-md rounded-2xl border border-slate-700/50 shadow-2xl overflow-hidden flex flex-col">
      
      {/* Header */}
      <div className="bg-slate-800/80 px-4 py-2 flex justify-between items-center border-b border-slate-700/50">
        <span className="font-bold text-sm text-cyan-400">{drone.partyID}</span>
        <span className={`px-2 py-0.5 rounded text-[10px] font-bold ${isArmed ? 'bg-red-500/20 text-red-500 border border-red-500/50' : 'bg-emerald-500/20 text-emerald-500 border border-emerald-500/50'}`}>
          {isArmed ? 'ARMED' : 'DISARMED'}
        </span>
      </div>

      {/* Main HUD */}
      <div className="p-4 flex flex-col gap-4">
        
        {/* Telemetry Row */}
        <div className="grid grid-cols-3 gap-2">
          <div className="bg-slate-800/50 rounded-lg p-2 text-center border border-slate-700/30">
            <div className="text-[10px] text-slate-400 font-bold mb-1">ALT (m)</div>
            <div className="text-xl font-mono text-white">{drone.alt?.toFixed(1) || '0.0'}</div>
          </div>
          <div className="bg-slate-800/50 rounded-lg p-2 text-center border border-slate-700/30">
            <div className="text-[10px] text-slate-400 font-bold mb-1">SPD (m/s)</div>
            <div className="text-xl font-mono text-white">{drone.speed?.toFixed(1) || '0.0'}</div>
          </div>
          <div className="bg-slate-800/50 rounded-lg p-2 text-center border border-slate-700/30">
            <div className="text-[10px] text-slate-400 font-bold mb-1">MODE</div>
            <div className="text-sm font-bold text-cyan-400 mt-1">{drone.flightMode || 'UNK'}</div>
          </div>
        </div>

        {/* Artificial Horizon (Mockup) */}
        <div className="relative h-40 bg-slate-800/50 rounded-xl border border-slate-700/50 overflow-hidden flex flex-col items-center">
           
           {/* Sky/Ground background */}
           <div className="relative w-full h-24 flex flex-col justify-center items-center overflow-hidden">
              <div className="absolute inset-0 flex flex-col">
                <div className="flex-1 bg-sky-500/20"></div>
                <div className="flex-1 bg-amber-700/20 border-t border-white/20"></div>
              </div>
              <div className="absolute w-16 h-[2px] bg-yellow-400/80 rounded-full"></div>
              <div className="absolute w-[2px] h-4 bg-yellow-400/80 rounded-full -mt-2"></div>
              
              {/* Heading indicator at top */}
              <div className="absolute top-1 bg-black/50 px-2 py-0.5 rounded text-[10px] text-white font-mono border border-white/10">
                {heading.toFixed(0)}°
              </div>
           </div>

           {/* Compass Bottom */}
           <div className="relative w-full h-16 bg-slate-900 border-t border-slate-700/50 flex items-center justify-center overflow-hidden">
              {/* Static pointing arrow */}
              <div className="absolute top-0 w-0 h-0 border-l-[6px] border-l-transparent border-r-[6px] border-r-transparent border-t-[8px] border-t-cyan-400 z-10"></div>
              
              {/* Rotating Compass Tape/Ring */}
              <div 
                className="absolute w-24 h-24 border-[3px] border-slate-600 rounded-full flex items-center justify-center transition-transform duration-300"
                style={{ transform: `translateY(20px) rotate(${-heading}deg)` }}
              >
                <div className="absolute top-0.5 text-[8px] font-bold text-red-500">N</div>
                <div className="absolute bottom-0.5 text-[8px] font-bold text-white">S</div>
                <div className="absolute right-0.5 text-[8px] font-bold text-white">E</div>
                <div className="absolute left-0.5 text-[8px] font-bold text-white">W</div>
                {/* Crosshairs inside compass */}
                <div className="w-full h-[1px] bg-slate-600/50 absolute"></div>
                <div className="h-full w-[1px] bg-slate-600/50 absolute"></div>
              </div>
           </div>
        </div>

        {/* Status Text */}
        <div className="flex justify-between items-center text-[10px] text-slate-400 font-bold">
          <div className="flex items-center gap-1">
            <div className="w-2 h-2 rounded-full bg-emerald-500"></div>
            GPS 3D FIX
          </div>
          <div className="text-amber-400">STATE: {drone.state || 'Standby'}</div>
        </div>

      </div>
    </div>
  )
}
