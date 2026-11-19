/*
 * Purpose: Hiển thị bản đồ Leaflet và các drone đang hoạt động. Thêm tính năng Fly Here (Guided Mode).
 * Dependencies: react-leaflet, useDroneStore
 * Author: Antigravity
 * Date: 2026-06-07
 */

import React, { useEffect, useState } from 'react'
import { MapContainer, TileLayer, Marker, Popup, Polyline, useMapEvents } from 'react-leaflet'
import 'leaflet/dist/leaflet.css'
import L from 'leaflet'
import { useDroneStore } from '../../store/droneStore'
import { protocolManager } from '../../core/ProtocolManager'

// Fix icon lỗi của Leaflet khi dùng với Vite
import iconRetinaUrl from 'leaflet/dist/images/marker-icon-2x.png'
import iconUrl from 'leaflet/dist/images/marker-icon.png'
import shadowUrl from 'leaflet/dist/images/marker-shadow.png'

delete (L.Icon.Default.prototype as any)._getIconUrl
L.Icon.Default.mergeOptions({
  iconRetinaUrl,
  iconUrl,
  shadowUrl,
})

const createRotatedDroneIcon = (heading: number, isLeader: boolean, isSimulated: boolean) => {
  if (!isSimulated) {
    return new L.Icon({
      iconUrl: 'https://raw.githubusercontent.com/pointhi/leaflet-color-markers/master/img/marker-icon-2x-blue.png',
      shadowUrl: 'https://cdnjs.cloudflare.com/ajax/libs/leaflet/0.7.7/images/marker-shadow.png',
      iconSize: [25, 41],
      iconAnchor: [12, 41],
      popupAnchor: [1, -34],
      shadowSize: [41, 41]
    })
  }

  const color = isLeader ? '#a855f7' : '#ef4444' // Purple for leader, Red for followers
  
  // HTML SVG cho drone, với transform: rotate() để xoay icon theo Heading
  // Có mũi nhọn màu trắng (Front Arrow Indicator) để chỉ hướng mũi drone
  const svg = `
    <div style="transform: rotate(${heading}deg); transition: transform 0.3s linear; width: 40px; height: 40px; filter: drop-shadow(0 0 4px ${color});">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" style="width: 100%; height: 100%;">
        <!-- Arms -->
        <path d="M 20 20 L 80 80 M 80 20 L 20 80" stroke="${color}" stroke-width="6"/>
        <!-- Back rotors -->
        <circle cx="20" cy="80" r="15" fill="none" stroke="${color}" stroke-width="4"/>
        <circle cx="80" cy="80" r="15" fill="none" stroke="${color}" stroke-width="4"/>
        <!-- Front rotors (opacity slightly lower to distinguish) -->
        <circle cx="20" cy="20" r="15" fill="${color}" stroke="${color}" stroke-width="4" fill-opacity="0.2"/>
        <circle cx="80" cy="20" r="15" fill="${color}" stroke="${color}" stroke-width="4" fill-opacity="0.2"/>
        <circle cx="20" cy="20" r="15" fill="none" stroke="${color}" stroke-width="4"/>
        <circle cx="80" cy="20" r="15" fill="none" stroke="${color}" stroke-width="4"/>
        <!-- Body -->
        <circle cx="50" cy="50" r="12" fill="${color}"/>
        <!-- Front Arrow Indicator -->
        <path d="M 50 15 L 60 40 L 40 40 Z" fill="#ffffff"/>
      </svg>
    </div>
  `
  return L.divIcon({
    className: 'custom-drone-icon bg-transparent border-none',
    html: svg,
    iconSize: [40, 40],
    iconAnchor: [20, 20],
    popupAnchor: [0, -20],
  })
}

const getDroneColorHex = (id: string) => {
  if (id.includes('1')) return '#f59e0b'; // Amber cho sim_drone_1
  if (id.includes('2')) return '#3b82f6'; // Blue cho sim_drone_2
  if (id.includes('3')) return '#10b981'; // Emerald cho sim_drone_3
  return '#0ea5e9'; // Mặc định Cyan
}

const getDroneMarkerUrl = (id: string) => {
  if (id.includes('1')) return 'https://raw.githubusercontent.com/pointhi/leaflet-color-markers/master/img/marker-icon-2x-gold.png';
  if (id.includes('2')) return 'https://raw.githubusercontent.com/pointhi/leaflet-color-markers/master/img/marker-icon-2x-blue.png';
  if (id.includes('3')) return 'https://raw.githubusercontent.com/pointhi/leaflet-color-markers/master/img/marker-icon-2x-green.png';
  return 'https://raw.githubusercontent.com/pointhi/leaflet-color-markers/master/img/marker-icon-2x-violet.png';
}

const getTargetIcon = (id: string) => new L.Icon({
  iconUrl: getDroneMarkerUrl(id),
  shadowUrl: 'https://cdnjs.cloudflare.com/ajax/libs/leaflet/0.7.7/images/marker-shadow.png',
  iconSize: [25, 41],
  iconAnchor: [12, 41],
  popupAnchor: [1, -34],
  shadowSize: [41, 41]
})

// Component để auto-pan bản đồ đến drone đầu tiên
function MapFocus() {
  const drones = useDroneStore((state) => state.drones)

  useEffect(() => {
    const droneList = Object.values(drones)
    if (droneList.length > 0) {
      // const firstDrone = droneList[0]
      // Chỉ auto-pan lần đầu, không liên tục pan làm người dùng khó thao tác click
      // Ở đây dùng cờ tạm thời, nhưng để an toàn cứ để người dùng tự do pan
    }
  }, [drones]) // Dependency to only pan on mount/first load

  return null
}

// Component xử lý click trên bản đồ
function MapClickHandler() {
  const [clickedPos, setClickedPos] = useState<L.LatLng | null>(null)
  const drones = useDroneStore((state) => state.drones)
  const selectedDroneId = useDroneStore((state) => state.selectedDroneId)
  
  useMapEvents({
    click(e) {
      setClickedPos(e.latlng)
    },
  })

  if (!clickedPos) return null

  // Dùng selectedDrone thay vì hardcode phần tử đầu tiên
  const activeDrone = selectedDroneId ? drones[selectedDroneId] : undefined
  const isGuided = activeDrone?.flightMode === 'Guided'

  const handleFlyHere = () => {
    if (activeDrone && isGuided) {
      protocolManager.setTargetWaypoint(activeDrone.partyID, clickedPos.lat, clickedPos.lng)
    }
    setClickedPos(null)
  }

  const handleAddWaypoint = () => {
    if (activeDrone && isGuided) {
      useDroneStore.getState().addWaypoint(activeDrone.partyID, clickedPos.lat, clickedPos.lng)
    }
    setClickedPos(null)
  }

  const isLeaderDrone = useDroneStore.getState().leaderId === activeDrone?.partyID || useDroneStore.getState().leaderId === null
  const isFollower = !isLeaderDrone

  return (
    <Popup position={clickedPos} eventHandlers={{ remove: () => setClickedPos(null) }}>
      <div className="text-center w-36">
        <h4 className="font-bold text-sm mb-2 text-gray-800 border-b pb-1">Waypoint Options</h4>
        <p className="text-[10px] text-gray-600 mb-3 font-mono">
          Lat: {clickedPos.lat.toFixed(6)}<br/>
          Lng: {clickedPos.lng.toFixed(6)}
        </p>
        {isFollower ? (
          <p className="text-[10px] text-red-500 font-bold bg-red-100 p-1 rounded border border-red-200">
            Follower is controlled by Leader.
          </p>
        ) : isGuided ? (
          <div className="flex flex-col gap-2">
            <button 
              onClick={handleFlyHere}
              className="bg-orange-500 hover:bg-orange-600 text-white font-bold py-1 px-3 rounded text-xs w-full transition-colors"
            >
              Fly to here
            </button>
            <button 
              onClick={handleAddWaypoint}
              className="bg-cyan-600 hover:bg-cyan-700 text-white font-bold py-1 px-3 rounded text-xs w-full transition-colors"
            >
              Add to Mission
            </button>
          </div>
        ) : (
          <p className="text-xs text-red-500 italic font-bold">Switch to GUIDED mode first.</p>
        )}
      </div>
    </Popup>
  )
}


// Component theo dõi va chạm dựa trên pixel
function CollisionTracker() {
  const map = useMapEvents({})
  const drones = useDroneStore((state) => state.drones)
  const [collisions, setCollisions] = useState<string[]>([])

  useEffect(() => {
    const droneList = Object.values(drones)
    if (droneList.length < 2) return

    const newCollisions: string[] = []

    for (let i = 0; i < droneList.length; i++) {
      for (let j = i + 1; j < droneList.length; j++) {
        const d1 = droneList[i]
        const d2 = droneList[j]
        
        // Chuyển toạ độ lat/lng thành toạ độ pixel trên màn hình
        const p1 = map.latLngToContainerPoint([d1.lat, d1.lng])
        const p2 = map.latLngToContainerPoint([d2.lat, d2.lng])

        // Tính khoảng cách pixel giữa 2 drone
        const distancePx = p1.distanceTo(p2)
        
        // Icon size là 40x40 -> Bán kính là 20. Nếu khoảng cách pixel < 40px thì xem như pixel chạm hoặc đè lên nhau.
        if (distancePx < 40) {
          newCollisions.push(`${d1.partyID} và ${d2.partyID} đang va chạm! (Distance: ${Math.round(distancePx)}px)`)
          console.error(`[COLLISION DETECTED] ${d1.partyID} vs ${d2.partyID} - Distance: ${distancePx.toFixed(2)}px`)
        }
      }
    }

    setCollisions(newCollisions)
  }, [drones, map])

  if (collisions.length === 0) return null

  return (
    <div className="absolute top-1/4 left-1/2 transform -translate-x-1/2 z-[1000] pointer-events-none">
      {collisions.map((msg, idx) => (
        <div key={idx} className="bg-red-600 text-white font-bold text-lg px-6 py-3 rounded-lg shadow-2xl border-4 border-red-300 animate-bounce mb-2 collision-alert">
          ⚠️ CẢNH BÁO VA CHẠM: {msg} ⚠️
        </div>
      ))}
    </div>
  )
}

export const DroneMap: React.FC = () => {
  const drones = useDroneStore((state) => state.drones)
  const droneList = Object.values(drones)

  // Toạ độ mặc định (HCMUTE)
  const centerPosition: [number, number] = [10.850145, 106.771661]

  return (
    <div className="absolute inset-0 w-full h-full z-0">
      <MapContainer
        center={centerPosition}
        zoom={16}
        maxZoom={22}
        scrollWheelZoom={true}
        zoomControl={false} // Tắt zoom control mặc định để nhìn clean hơn
        style={{ height: '100%', width: '100%', background: '#0f172a' }}
      >
        <TileLayer
          attribution='&copy; <a href="https://www.esri.com/">Esri</a>'
          url="https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}"
          maxNativeZoom={18}
          maxZoom={22}
        />
        
        {/* Click handler */}
        <MapClickHandler />
        
        {/* Bộ theo dõi va chạm (Collision Tracker) */}
        <CollisionTracker />

        {droneList.map((drone) => (
          <React.Fragment key={drone.partyID}>
            {/* Vẽ đường bay từ vị trí hiện tại -> targetWaypoint -> các điểm trong missionWaypoints */}
            {(() => {
              const positions: [number, number][] = [[drone.lat, drone.lng]]
              if (drone.targetWaypoint) positions.push([drone.targetWaypoint.lat, drone.targetWaypoint.lng])
              if (drone.missionWaypoints) {
                 drone.missionWaypoints.forEach(wp => positions.push([wp.lat, wp.lng]))
              }
              
              if (positions.length > 1) {
                return (
                  <Polyline 
                    positions={positions} 
                    color={getDroneColorHex(drone.partyID)} 
                    dashArray="5, 10" 
                    weight={2} 
                  />
                )
              }
              return null
            })()}

            {/* Marker cho Target Waypoint hiện tại */}
            {drone.targetWaypoint && (
              <Marker position={[drone.targetWaypoint.lat, drone.targetWaypoint.lng]} icon={getTargetIcon(drone.partyID)}>
                 <Popup className="bg-slate-800 text-white"><span className="p-1">Target Waypoint</span></Popup>
              </Marker>
            )}

            {/* Markers cho Mission Waypoints */}
            {drone.missionWaypoints && drone.missionWaypoints.map((wp, index) => {
               // Tạo icon số thứ tự
               const colorHex = getDroneColorHex(drone.partyID);
               const numberIcon = L.divIcon({
                 className: 'bg-transparent border-none',
                 html: `<div style="background-color: ${colorHex}" class="w-6 h-6 rounded-full border-2 border-white flex items-center justify-center text-white font-bold text-xs shadow-lg">${index + 1}</div>`,
                 iconSize: [24, 24],
                 iconAnchor: [12, 12]
               })
               return (
                 <Marker key={index} position={[wp.lat, wp.lng]} icon={numberIcon} />
               )
            })}

            <Marker
              position={[drone.lat, drone.lng]}
              icon={createRotatedDroneIcon(drone.heading || 0, useDroneStore.getState().leaderId === drone.partyID, !!drone.isSimulated)}
            >
              <Popup className="bg-slate-800 text-white rounded-lg">
                <div className="p-1">
                  <h3 className="font-bold text-sky-400 border-b border-slate-600 pb-1 mb-1">
                    {drone.partyID} {drone.isSimulated && '(Mock)'}
                  </h3>
                  <p className="text-sm m-0">Alt: {drone.alt?.toFixed(1) || 0}m</p>
                  <p className="text-sm m-0">Spd: {drone.speed?.toFixed(1) || 0}m/s</p>
                  <p className="text-sm m-0 text-slate-400 text-xs mt-1">
                    Updated: {new Date(drone.lastUpdated).toLocaleTimeString()}
                  </p>
                </div>
              </Popup>
            </Marker>
          </React.Fragment>
        ))}

        <MapFocus />
      </MapContainer>
    </div>
  )
}
