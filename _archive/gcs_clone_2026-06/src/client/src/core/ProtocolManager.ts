/*
 * Purpose: Quản lý kết nối WebSocket đến Communication Server và Simulator (Bầy đàn).
 * Dependencies: useDroneStore, useAuthStore
 * Author: Antigravity
 * Date: 2026-06-07
 */

import { useDroneStore } from '../store/droneStore'
import { useAuthStore } from '../store/authStore'
import { useLogStore } from '../store/logStore'
import { logClientEvent } from '../utils/logger'
import { EventEmitter } from '../utils/EventEmitter'

interface SimDroneState {
  partyID: string
  lat: number
  lng: number
  alt: number
  heading: number
  speed: number
  formationOffset: { back: number, right: number } // offset (m) relative to leader
}

export class ProtocolManager extends EventEmitter {
  private ws: WebSocket | null = null
  private simInterval: ReturnType<typeof setInterval> | null = null
  private isSimulating: boolean = false
  
  // Toạ độ HCMUTE
  private readonly ORIGIN_LAT = 10.850145
  private readonly ORIGIN_LNG = 106.771661
  
  private simDrones: SimDroneState[] = []

  constructor() {
    super()
  }

  public connect() {
    const authState = useAuthStore.getState()
    if (!authState.isAuthenticated || !authState.commServer || !authState.sessionId) {
      console.warn('Cannot connect: Not authenticated')
      return
    }

    const { g: host, h: port, f: tempKey } = authState.commServer
    const partyID = authState.sessionId 
    const url = `wss://${host}:${port}/?f=${tempKey}&s=${partyID}&at=g`
    
    logClientEvent('info', 'WS_CONNECTING', { url })

    try {
      this.ws = new WebSocket(url)
      this.ws.binaryType = 'arraybuffer'
      this.ws.onopen = () => {
        logClientEvent('info', 'WS_CONNECTED', {})
        this.emit('connected', null)
      }
      this.ws.onmessage = (event) => {
        try {
          if (typeof event.data === 'string') {
            const payload = JSON.parse(event.data)
            // Multiplex by sender (partyID of the drone)
            if (payload.sender) {
              this.emit(`message:${payload.sender}`, payload)
            } else if (payload.sysid) {
              this.emit(`telemetry:${payload.sysid}`, payload)
            } else {
              this.emit('system_message', payload)
            }
          } else {
            // Handle binary message (ArrayBuffer)
            // The protocol says: JSON header as byte array + binary payload
            this.emit('binary_message', event.data)
          }
        } catch (error) {
          logClientEvent('error', 'WS_PARSE_ERROR', { error })
        }
      }
      this.ws.onclose = () => {
        logClientEvent('info', 'WS_CLOSED', {})
        this.ws = null
      }
      this.ws.onerror = (error) => logClientEvent('error', 'WS_ERROR', { error })
    } catch (e) {
      logClientEvent('error', 'WS_INIT_FAILED', { error: e })
    }
  }

  public disconnect() {
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
    this.stopSimulator()
  }

  // ==========================================
  // GIAO THỨC TRUYỀN THÔNG (ANDRUAV PROTOCOL)
  // ==========================================
  public sendCommand(targetPartyID: string, messageType: number, msgPayload: any) {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      console.warn('Cannot send command: WebSocket is not open');
      return;
    }
    const authState = useAuthStore.getState()
    const sender = authState.sessionId
    
    const envelope = {
      _ty: "c", // Communication command
      i: targetPartyID, // Target individual (or '_AGN_' for all drones)
      messageType,
      sender,
      msg: msgPayload
    }
    
    this.ws.send(JSON.stringify(envelope))
    logClientEvent('info', 'WS_SEND', { targetPartyID, messageType, msgPayload })
  }

  // ==========================================
  // LỆNH ĐIỀU KHIỂN TELEMETRY (LVL / UDP)
  // ==========================================
  public setTelemetryLevel(partyID: string, level: number) {
    // 1005 = Remote Command, C: 108 = Telemetry Control, Act: 4 = Set Level
    this.sendCommand(partyID, 1005, { C: 108, Act: 4, LVL: level })
  }

  public toggleUdpTelemetry(partyID: string, isPaused: boolean) {
    // Act: 3 = Resume, Act: 5 = Pause
    const act = isPaused ? 5 : 3
    this.sendCommand(partyID, 1005, { C: 108, Act: act })
  }

  // ==========================================
  // LỆNH ĐIỀU KHIỂN INDIVIDUAL
  // ==========================================
  public armDrone(partyID: string) {
    // 1005 = Remote Command, C: 100 = Arm
    this.sendCommand(partyID, 1005, { C: 100 })
    useDroneStore.getState().updateDrone({ partyID, isArmed: true })
    useLogStore.getState().addLog(`[${partyID}] Armed`, 'info')
  }

  public disarmDrone(partyID: string) {
    this.sendCommand(partyID, 1005, { C: 101 }) // Disarm
    useDroneStore.getState().updateDrone({ partyID, isArmed: false })
    useLogStore.getState().addLog(`[${partyID}] Disarmed`, 'warn')
  }

  public setFlightMode(partyID: string, mode: string) {
    const d = useDroneStore.getState().drones[partyID]
    if (d) useDroneStore.getState().updateDrone({ partyID, flightMode: mode })
  }

  public takeoffDrone(partyID: string) {
    // Act: 1 = Takeoff
    this.sendCommand(partyID, 1005, { C: 103, Act: 1 })
    useDroneStore.getState().updateDrone({ partyID, state: 'TakingOff' })
    useLogStore.getState().addLog(`[${partyID}] Taking Off`, 'info')
  }

  public landDrone(partyID: string) {
    const d = useDroneStore.getState().drones[partyID]
    if (d && d.isArmed && (d.alt || 0) > 0) {
      useDroneStore.getState().updateDrone({ partyID, state: 'Landing' })
    }
  }

  public rtlDrone(partyID: string) {
    const d = useDroneStore.getState().drones[partyID]
    if (d && d.isArmed && (d.alt || 0) > 0) {
      useDroneStore.getState().updateDrone({ partyID, state: 'RTL', targetWaypoint: undefined })
    }
  }

  public setTargetWaypoint(partyID: string, lat: number, lng: number) {
    // 1005 = Remote Command, C: 104 = Waypoint Control, Act: 1 = Fly to here
    this.sendCommand(partyID, 1005, { C: 104, Act: 1, lat, lng })
    useDroneStore.getState().updateDrone({ partyID, targetWaypoint: { lat, lng } })
    useLogStore.getState().addLog(`[${partyID}] Flying to Lat:${lat.toFixed(4)} Lng:${lng.toFixed(4)}`, 'info')
  }

  public startMission(partyID: string) {
    const drone = useDroneStore.getState().drones[partyID]
    if (drone && drone.missionWaypoints && drone.missionWaypoints.length > 0) {
      const firstWp = drone.missionWaypoints[0]
      useDroneStore.getState().popWaypoint(partyID)
      this.setTargetWaypoint(partyID, firstWp.lat, firstWp.lng)
      useLogStore.getState().addLog(`[${partyID}] Started Mission`, 'info')
    }
  }

  // ==========================================
  // LỆNH ĐIỀU KHIỂN SWARM
  // ==========================================
  public armAll() {
    const state = useDroneStore.getState()
    Object.keys(state.drones).forEach(partyID => this.armDrone(partyID))
  }

  public takeoffAll() {
    const state = useDroneStore.getState()
    Object.keys(state.drones).forEach(partyID => this.takeoffDrone(partyID))
  }
  
  public landAll() {
    const state = useDroneStore.getState()
    Object.keys(state.drones).forEach(partyID => this.landDrone(partyID))
  }

  public rtlAll() {
    const state = useDroneStore.getState()
    Object.keys(state.drones).forEach(partyID => this.rtlDrone(partyID))
  }

  // ==========================================
  // DRONE SIMULATOR LOGIC (SWARM)
  // ==========================================

  public toggleSimulator() {
    if (this.isSimulating) {
      this.stopSimulator()
    } else {
      this.startSimulator()
    }
    return this.isSimulating
  }

  private startSimulator() {
    this.isSimulating = true

    // Khởi tạo 3 drones với vị trí ban đầu hơi lệch nhau
    this.simDrones = [
      { partyID: 'sim_drone_1', lat: this.ORIGIN_LAT, lng: this.ORIGIN_LNG, alt: 0, heading: 0, speed: 0, formationOffset: { back: 0, right: 0 } },
      { partyID: 'sim_drone_2', lat: this.ORIGIN_LAT - 0.001, lng: this.ORIGIN_LNG - 0.001, alt: 0, heading: 0, speed: 0, formationOffset: { back: 20, right: -20 } },
      { partyID: 'sim_drone_3', lat: this.ORIGIN_LAT - 0.001, lng: this.ORIGIN_LNG + 0.001, alt: 0, heading: 0, speed: 0, formationOffset: { back: 20, right: 20 } }
    ]

    logClientEvent('info', 'SIMULATOR_STARTED', { swarmSize: 3 })

    // Đẩy vào store
    this.simDrones.forEach(sd => {
      useDroneStore.getState().updateDrone({
        partyID: sd.partyID,
        lat: sd.lat,
        lng: sd.lng,
        heading: sd.heading,
        alt: sd.alt,
        speed: 0,
        isSimulated: true,
        isArmed: false,
        flightMode: 'Guided', // Mặc định Guided cho dễ fly here
        state: 'Standby'
      })
    })
    
    // Mặc định set leader là drone 1
    useDroneStore.getState().setLeader('sim_drone_1')

    this.simInterval = setInterval(() => {
      const state = useDroneStore.getState()
      const settings = state.settings
      const leaderId = state.leaderId

      this.simDrones.forEach(sd => {
        const d = state.drones[sd.partyID]
        if (!d) return

        let currentSpeed = 0
        const isFlockMode = state.isFlockMode
        const isLeader = !isFlockMode && (leaderId === sd.partyID || !leaderId)
        const leaderData = leaderId && !isLeader && !isFlockMode ? state.drones[leaderId] : null
        const leaderSimData = leaderId && !isLeader && !isFlockMode ? this.simDrones.find(s => s.partyID === leaderId) : null

        if (d.isArmed) {
          // XỬ LÝ ĐỘ CAO (ALT)
          let targetAlt = 0
          if (d.state === 'TakingOff') targetAlt = settings.altStep
          else if (d.state === 'Flying' || d.state === 'RTL') {
            // Nếu là follower, copy state của leader nếu có
            if (!isLeader && leaderData && leaderData.state === 'TakingOff') {
               useDroneStore.getState().updateDrone({ partyID: sd.partyID, state: 'TakingOff' })
            }
            if (!isLeader && leaderData && leaderData.state === 'Landing') {
               useDroneStore.getState().updateDrone({ partyID: sd.partyID, state: 'Landing' })
            }
            if (!isLeader && leaderData && leaderData.state === 'RTL') {
               useDroneStore.getState().updateDrone({ partyID: sd.partyID, state: 'RTL' })
            }
            targetAlt = (leaderData && !isLeader) ? (leaderData.alt || settings.altStep) : settings.altStep
          }

          if (d.state === 'TakingOff' || d.state === 'Flying' || d.state === 'RTL') {
            if (sd.alt < targetAlt) sd.alt = Math.min(targetAlt, sd.alt + 2)
            else if (sd.alt > targetAlt) sd.alt = Math.max(targetAlt, sd.alt - 2)
            
            if (d.state === 'TakingOff' && sd.alt >= targetAlt) {
              useDroneStore.getState().updateDrone({ partyID: sd.partyID, state: 'Flying' })
            }
          } else if (d.state === 'Landing') {
            if (sd.alt > 0) sd.alt -= 2
            else {
              sd.alt = 0
              useDroneStore.getState().updateDrone({ partyID: sd.partyID, state: 'Standby' })
            }
          }

          // XỬ LÝ DI CHUYỂN
          if ((d.state === 'Flying' && d.flightMode !== 'Loiter' && d.flightMode !== 'Stabilize') || d.state === 'RTL') {
            
            let destLat = sd.lat
            let destLng = sd.lng

            if (d.state === 'RTL') {
              destLat = this.ORIGIN_LAT
              destLng = this.ORIGIN_LNG
            } else if (isFlockMode) {
              // ==========================================
              // BOIDS ORGANIC FLOCKING (REYNOLDS)
              // ==========================================
              let cohLat = 0, cohLng = 0
              let alignHeading = 0
              let sepLat = 0, sepLng = 0
              let neighborCount = 0

              for (const other of this.simDrones) {
                 if (other.partyID !== sd.partyID && other.alt > 0) {
                    const dLat = sd.lat - other.lat
                    const dLng = sd.lng - other.lng
                    const otherDist = Math.sqrt(dLat*dLat + dLng*dLng)
                    if (otherDist < 0.003) { // Bán kính nhận biết đàn: ~300m
                       neighborCount++
                       cohLat += other.lat
                       cohLng += other.lng
                       alignHeading += other.heading
                       
                       if (otherDist < 0.001) { // Bán kính tránh va chạm: ~110m (để không đè pixel)
                          // Lực đẩy tỷ lệ nghịch (càng gần càng đẩy mạnh ra)
                          const strength = (0.001 - otherDist) / 0.001
                          sepLat += (dLat / otherDist) * strength
                          sepLng += (dLng / otherDist) * strength
                       }
                    }
                 }
              }

              if (neighborCount > 0) {
                 // Tính trung bình
                 cohLat /= neighborCount
                 cohLng /= neighborCount
                 alignHeading /= neighborCount
                 
                 // Lực dính (Cohesion): Kéo về tâm của bầy
                 const cLat = cohLat - sd.lat
                 const cLng = cohLng - sd.lng
                 
                 // Lực đồng hướng (Alignment): Bay theo hướng của bầy
                 const rad = alignHeading * Math.PI / 180
                 const aLat = Math.cos(rad) * 0.0001 // Trọng số bay tới
                 const aLng = Math.sin(rad) * 0.0001

                 // Lực hút về giữa bản đồ (tránh bay mất tích ra ngoài map)
                 const homeLat = (this.ORIGIN_LAT - sd.lat) * 0.02
                 const homeLng = (this.ORIGIN_LNG - sd.lng) * 0.02

                 // Tổng hợp lực: 10% Cohesion + Separation cực mạnh + Alignment bay tới + lực hút nhẹ về nhà
                 destLat = sd.lat + cLat * 0.1 + sepLat * 0.00015 + aLat + homeLat
                 destLng = sd.lng + cLng * 0.1 + sepLng * 0.00015 + aLng + homeLng
              } else {
                 // Đang cô đơn, lượn vòng tròn tìm bầy
                 sd.heading = (sd.heading + 2) % 360
                 const rad = sd.heading * Math.PI / 180
                 destLat = sd.lat + Math.cos(rad) * 0.0001
                 destLng = sd.lng + Math.sin(rad) * 0.0001
              }
            } else if (isLeader) {
              if (d.flightMode === 'Guided' && d.targetWaypoint) {
                destLat = d.targetWaypoint.lat
                destLng = d.targetWaypoint.lng
              } else if (d.flightMode === 'Auto') {
                const radius = settings.radius / 100000 
                const speedDeg = 2 
                sd.heading = (sd.heading + speedDeg) % 360
                const rad = sd.heading * (Math.PI / 180)
                destLat = this.ORIGIN_LAT + radius * Math.cos(rad)
                destLng = this.ORIGIN_LNG + radius * Math.sin(rad)
              }
            } else if (leaderData && leaderSimData && leaderData.state === 'Flying') {
              // ==========================================
              // VIRTUAL STRUCTURE (ADVANCED FORMATIONS)
              // ==========================================
              const followers = this.simDrones.filter(s => s.partyID !== leaderId).sort((a,b) => a.partyID.localeCompare(b.partyID))
              const index = followers.findIndex(s => s.partyID === sd.partyID) + 1 // 1-based index

              let back = 0
              let right = 0

              switch (state.swarmFormation) {
                case 'V-Shape':
                   back = Math.ceil(index / 2) * 20
                   right = (index % 2 === 0 ? 1 : -1) * Math.ceil(index / 2) * 20
                   break
                case 'Line':
                   back = 0
                   right = (index % 2 === 0 ? 1 : -1) * Math.ceil(index / 2) * 20
                   break
                case 'Column':
                   back = index * 20
                   right = 0
                   break
                case 'Circle':
                   const angle = (360 / followers.length) * (index - 1)
                   const circleRad = angle * Math.PI / 180
                   back = Math.cos(circleRad) * 30 // Bán kính 30m
                   right = Math.sin(circleRad) * 30
                   break
              }

              const leadHeadingRad = leaderSimData.heading * (Math.PI / 180)
              const backRad = leadHeadingRad + Math.PI
              const rightRad = leadHeadingRad + (Math.PI / 2)
              const mToDeg = 0.000009 

              const backOffsetLat = Math.cos(backRad) * back * mToDeg
              const backOffsetLng = Math.sin(backRad) * back * mToDeg
              const rightOffsetLat = Math.cos(rightRad) * right * mToDeg
              const rightOffsetLng = Math.sin(rightRad) * right * mToDeg

              destLat = leaderSimData.lat + backOffsetLat + rightOffsetLat
              destLng = leaderSimData.lng + backOffsetLng + rightOffsetLng
            }

            // Di chuyển về phía dest
            const distLat = destLat - sd.lat
            const distLng = destLng - sd.lng
            const dist = Math.sqrt(distLat*distLat + distLng*distLng)

            // Tính góc hướng tới đích
            let targetHeading = Math.atan2(distLng, distLat) * (180 / Math.PI)
            if (targetHeading < 0) targetHeading += 360

            // Xoay đầu drone về đích trước khi bay
            let headingDiff = targetHeading - sd.heading
            // Đưa về khoảng [-180, 180] để xoay theo đường ngắn nhất
            while (headingDiff <= -180) headingDiff += 360
            while (headingDiff > 180) headingDiff -= 360

            if (d.state === 'RTL' && dist < 0.00005) {
               useDroneStore.getState().updateDrone({ partyID: sd.partyID, state: 'Landing' })
               useLogStore.getState().addLog(`[${sd.partyID}] Reached Home. Landing.`, 'info')
            } else if (dist > 0.00002) {
               const simSpeedMult = settings.simSpeed || 1
               currentSpeed = (isLeader ? 5 : 7) * simSpeedMult 
               const maxMove = currentSpeed * 0.00001
               
               // Nếu lệch góc lớn hơn 30 độ (tốc độ xoay), chỉ xoay chứ chưa bay
               if (Math.abs(headingDiff) > 30) {
                  sd.heading += Math.sign(headingDiff) * 30 * simSpeedMult // Tốc độ xoay: 30 độ / tick
                  currentSpeed = 0 // Đang xoay thì dừng lại
               } else {
                  // Góc lệch đã nhỏ hơn 30 độ -> Snap thẳng vào mục tiêu để KHÔNG BỊ OVERSHOOT
                  sd.heading = targetHeading 
                  
                  // COLLISION AVOIDANCE (Chỉ áp dụng khi không phải Flock Mode)
                  let collisionRepelLat = 0
                  let collisionRepelLng = 0
                  let forwardMoveDist = Math.min(dist, maxMove)
                  
                  if (!isFlockMode) {
                    for (const other of this.simDrones) {
                      if (other.partyID !== sd.partyID && other.alt > 0) {
                        const dLat = sd.lat - other.lat
                        const dLng = sd.lng - other.lng
                        const otherDist = Math.sqrt(dLat*dLat + dLng*dLng)
                        
                        // Bán kính tránh va chạm: ~130 mét (để tuyệt đối không đè pixel < 40px)
                        const safeRadius = 0.0012
                        if (otherDist < safeRadius && otherDist > 0) {
                           // Giảm vận tốc tiến tới nếu đang vướng vật cản để khỏi lao thẳng vào nhau
                           forwardMoveDist *= 0.5

                           // Tăng lực đẩy cực đại lên 2.5 lần tốc độ di chuyển
                           const maxRepel = maxMove * 2.5
                           const repelStrength = maxRepel * (1 - otherDist / safeRadius)
                           
                           // Vector đẩy lùi (Repulsion)
                           const pushLat = (dLat / otherDist) * repelStrength
                           const pushLng = (dLng / otherDist) * repelStrength
                           
                           // Vector trượt ngang (Tangential) để lượn qua nhau
                           const slideLat = -(dLng / otherDist) * (repelStrength * 0.8)
                           const slideLng = (dLat / otherDist) * (repelStrength * 0.8)

                           collisionRepelLat += pushLat + slideLat
                           collisionRepelLng += pushLng + slideLng
                           
                           // Bỏ "phanh khẩn cấp" (forwardMoveDist = 0) để không bị kẹt cứng (deadlock) khi xếp đội hình
                           
                           // Log 1 lần xác suất cực nhỏ để khỏi spam log
                           if (Math.random() < 0.02) {
                              useLogStore.getState().addLog(`[${sd.partyID}] Avoiding collision with [${other.partyID}]`, 'warn')
                           }
                        }
                      }
                    }
                  }

                  sd.lat += (distLat / dist) * forwardMoveDist + collisionRepelLat
                  sd.lng += (distLng / dist) * forwardMoveDist + collisionRepelLng
               }
               // Chuẩn hoá góc
               while (sd.heading < 0) sd.heading += 360
               while (sd.heading >= 360) sd.heading -= 360

            } else {
               // Đã tới nơi (Hover)
               if (!isLeader && leaderSimData) {
                 sd.heading = leaderSimData.heading // Quay đầu theo leader
               } else if (isLeader && d.flightMode === 'Guided') {
                 if (d.missionWaypoints && d.missionWaypoints.length > 0) {
                    const nextWp = d.missionWaypoints[0]
                    useDroneStore.getState().popWaypoint(sd.partyID)
                    this.setTargetWaypoint(sd.partyID, nextWp.lat, nextWp.lng)
                 }
               }
               currentSpeed = 0
            }
          }
        }

        // Bơm vào Zustand
        useDroneStore.getState().updateDrone({
          partyID: sd.partyID,
          lat: sd.lat,
          lng: sd.lng,
          heading: sd.heading,
          alt: sd.alt,
          speed: currentSpeed
        })
      })
    }, 1000)
  }

  private stopSimulator() {
    this.isSimulating = false
    if (this.simInterval) {
      clearInterval(this.simInterval)
      this.simInterval = null
    }
    this.simDrones.forEach(sd => useDroneStore.getState().removeDrone(sd.partyID))
    this.simDrones = []
    useDroneStore.getState().setLeader(null)
    logClientEvent('info', 'SIMULATOR_STOPPED', {})
  }
}

export const protocolManager = new ProtocolManager()
