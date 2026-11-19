import { useEffect, useRef, useState } from 'react'
import { useLogStore } from '../../../store/logStore'

export function DebugConsole() {
  const logs = useLogStore((state) => state.logs)
  const clearLogs = useLogStore((state) => state.clearLogs)
  const [isMinimized, setIsMinimized] = useState(false)
  const scrollRef = useRef<HTMLDivElement>(null)

  // Tự động cuộn xuống dưới cùng khi có log mới
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [logs])

  if (isMinimized) {
    return (
      <div 
        onClick={() => setIsMinimized(false)}
        className="fixed bottom-4 left-1/2 -translate-x-1/2 bg-slate-900/90 border border-emerald-500/30 text-emerald-400 font-mono text-xs px-4 py-2 rounded shadow-2xl cursor-pointer hover:bg-slate-800 transition-colors z-[1000] pointer-events-auto"
      >
        <span>&gt;_ OPEN TERMINAL ({logs.length})</span>
      </div>
    )
  }

  return (
    <div className="fixed bottom-4 left-1/2 -translate-x-1/2 w-96 max-w-[90vw] h-64 bg-slate-900/95 border border-slate-700 rounded-xl shadow-2xl flex flex-col z-[1000] overflow-hidden backdrop-blur-md pointer-events-auto transition-all">
      {/* Header */}
      <div className="flex justify-between items-center bg-slate-800/80 border-b border-slate-700 px-3 py-2 cursor-default">
        <div className="flex items-center gap-2">
          <div className="w-2.5 h-2.5 rounded-full bg-red-500 cursor-pointer" onClick={() => setIsMinimized(true)}></div>
          <div className="w-2.5 h-2.5 rounded-full bg-amber-500"></div>
          <div className="w-2.5 h-2.5 rounded-full bg-emerald-500"></div>
          <span className="text-[10px] text-slate-400 font-bold ml-2">SYSTEM.LOG</span>
        </div>
        <button onClick={clearLogs} className="text-[10px] text-red-400 hover:text-red-300 font-bold transition-colors">
          CLEAR
        </button>
      </div>

      {/* Log list */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto p-3 font-mono text-[11px] leading-tight flex flex-col gap-1.5 scrollbar-thin scrollbar-thumb-slate-600">
        {logs.length === 0 ? (
          <div className="text-slate-500 italic">Waiting for events...</div>
        ) : (
          logs.map((log) => {
            const timeStr = new Date(log.timestamp).toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute:'2-digit', second:'2-digit' })
            let colorClass = 'text-emerald-400'
            if (log.level === 'warn') colorClass = 'text-amber-400'
            if (log.level === 'error') colorClass = 'text-red-400'

            return (
              <div key={log.id} className="flex gap-2 break-words">
                <span className="text-slate-500 shrink-0">[{timeStr}]</span>
                <span className={`${colorClass} break-all`}>{log.message}</span>
              </div>
            )
          })
        )}
      </div>
    </div>
  )
}
