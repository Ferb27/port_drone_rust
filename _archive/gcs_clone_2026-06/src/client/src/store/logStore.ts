import { create } from 'zustand'

export interface LogEntry {
  id: string
  timestamp: number
  message: string
  level: 'info' | 'warn' | 'error'
}

interface LogStore {
  logs: LogEntry[]
  addLog: (message: string, level?: 'info' | 'warn' | 'error') => void
  clearLogs: () => void
}

export const useLogStore = create<LogStore>((set) => ({
  logs: [],
  addLog: (message, level = 'info') => set((state) => {
    const newLog: LogEntry = {
      id: Math.random().toString(36).substr(2, 9),
      timestamp: Date.now(),
      message,
      level
    }
    // Giữ lại tối đa 100 log gần nhất để tránh lag
    const newLogs = [...state.logs, newLog]
    if (newLogs.length > 100) newLogs.shift()
    return { logs: newLogs }
  }),
  clearLogs: () => set({ logs: [] })
}))
