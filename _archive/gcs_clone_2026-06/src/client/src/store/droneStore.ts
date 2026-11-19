/*
 * Purpose: Quản lý trạng thái của các Drone (Telemetry, GPS)
 * Dependencies: zustand
 * Author: Antigravity
 * Date: 2026-06-07
 */

import { create } from 'zustand'

export type FormationType = 'V-Shape' | 'Line' | 'Column' | 'Circle'

export interface DroneData {
  partyID: string
  lat: number
  lng: number
  alt?: number
  heading?: number
  speed?: number
  isSimulated?: boolean
  isArmed?: boolean
  flightMode?: string
  state?: 'Standby' | 'TakingOff' | 'Flying' | 'Landing' | 'RTL'
  targetWaypoint?: { lat: number, lng: number }
  missionWaypoints?: { lat: number, lng: number }[]
  lastUpdated: number
}

interface DroneStore {
  drones: Record<string, DroneData>
  leaderId: string | null
  selectedDroneId: string | null
  swarmFormation: FormationType
  isFlockMode: boolean
  settings: {
    altStep: number
    radius: number
    hOffset: number
    vOffset: number
    simSpeed: number
  }
  updateSettings: (settings: Partial<DroneStore['settings']>) => void
  setLeader: (partyID: string | null) => void
  setSelectedDrone: (partyID: string | null) => void
  setSwarmFormation: (formation: FormationType) => void
  setFlockMode: (flock: boolean) => void
  /** Thêm hoặc cập nhật vị trí drone */
  updateDrone: (data: Partial<DroneData> & { partyID: string }) => void
  addWaypoint: (partyID: string, lat: number, lng: number) => void
  clearWaypoints: (partyID: string) => void
  popWaypoint: (partyID: string) => void
  /** Xoá drone khỏi bản đồ (khi offline quá lâu) */
  removeDrone: (partyID: string) => void
  /** Xoá toàn bộ drone */
  clearDrones: () => void
}

export const useDroneStore = create<DroneStore>((set) => ({
  drones: {},
  leaderId: null,
  selectedDroneId: null,
  swarmFormation: 'V-Shape',
  isFlockMode: false,
  settings: {
    altStep: 100,
    radius: 50,
    hOffset: 100,
    vOffset: 50,
    simSpeed: 1
  },
  updateSettings: (newSettings) => set((state) => ({ settings: { ...state.settings, ...newSettings } })),
  setLeader: (partyID) => set({ leaderId: partyID }),
  setSelectedDrone: (partyID) => set({ selectedDroneId: partyID }),
  setSwarmFormation: (formation) => set({ swarmFormation: formation }),
  setFlockMode: (flock) => set({ isFlockMode: flock }),
  updateDrone: (data) =>
    set((state) => {
      const existing = state.drones[data.partyID]
      return {
        drones: {
          ...state.drones,
          [data.partyID]: {
            ...existing,
            ...data,
            lastUpdated: Date.now(),
          },
        },
      }
    }),
  addWaypoint: (partyID, lat, lng) => set((state) => {
    const drone = state.drones[partyID]
    if (!drone) return state
    
    const missionWaypoints = [...(drone.missionWaypoints || []), { lat, lng }]
    return {
      drones: {
        ...state.drones,
        [partyID]: { ...drone, missionWaypoints }
      }
    }
  }),
  clearWaypoints: (partyID) => set((state) => {
    const drone = state.drones[partyID]
    if (!drone) return state
    return {
      drones: {
        ...state.drones,
        [partyID]: { ...drone, missionWaypoints: [] }
      }
    }
  }),
  popWaypoint: (partyID) => set((state) => {
    const drone = state.drones[partyID]
    if (!drone || !drone.missionWaypoints || drone.missionWaypoints.length === 0) return state
    
    const missionWaypoints = [...drone.missionWaypoints]
    missionWaypoints.shift()
    return {
      drones: {
        ...state.drones,
        [partyID]: { ...drone, missionWaypoints }
      }
    }
  }),
  removeDrone: (partyID) =>
    set((state) => {
      const newDrones = { ...state.drones }
      delete newDrones[partyID]
      return { drones: newDrones }
    }),
  clearDrones: () => set({ drones: {} }),
}))
