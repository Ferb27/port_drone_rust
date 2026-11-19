import * as fs from 'fs'

const ORIGIN_LAT = 10.8507
const ORIGIN_LNG = 106.7717
const DEG_TO_M = 111320

interface Drone {
    id: string
    lat: number
    lng: number
    alt: number
    heading: number
    speed: number
}

// Giả lập 5 Drone sát nhau để test khả năng né
let drones: Drone[] = [
    { id: 'D1', lat: ORIGIN_LAT, lng: ORIGIN_LNG, alt: 100, heading: 0, speed: 0 },
    { id: 'D2', lat: ORIGIN_LAT - 0.00005, lng: ORIGIN_LNG - 0.00005, alt: 100, heading: 0, speed: 0 },
    { id: 'D3', lat: ORIGIN_LAT - 0.00005, lng: ORIGIN_LNG + 0.00005, alt: 100, heading: 0, speed: 0 },
    { id: 'D4', lat: ORIGIN_LAT + 0.00005, lng: ORIGIN_LNG - 0.00005, alt: 100, heading: 0, speed: 0 },
    { id: 'D5', lat: ORIGIN_LAT + 0.00005, lng: ORIGIN_LNG + 0.00005, alt: 100, heading: 0, speed: 0 },
]

function getDistanceMeters(lat1: number, lng1: number, lat2: number, lng2: number) {
    const dLat = lat1 - lat2
    const dLng = lng1 - lng2
    return Math.sqrt(dLat*dLat + dLng*dLng) * DEG_TO_M
}

const TOTAL_TICKS = 5000 // 5000 ticks ~ 500 seconds of flight time (10Hz)
const logs: string[] = []
let minDistanceObserved = 999999
let crashCount = 0

logs.push(`Starting Flocking Simulation for ${TOTAL_TICKS} ticks with ${drones.length} drones...`)

for (let tick = 0; tick < TOTAL_TICKS; tick++) {
    const nextStates: any[] = []

    for (let i = 0; i < drones.length; i++) {
        const sd = drones[i]
        
        // BOIDS ORGANIC FLOCKING (REYNOLDS)
        let cohLat = 0, cohLng = 0
        let alignHeading = 0
        let sepLat = 0, sepLng = 0
        let neighborCount = 0

        for (const other of drones) {
            if (other.id !== sd.id) {
                const dLat = sd.lat - other.lat
                const dLng = sd.lng - other.lng
                const otherDistDeg = Math.sqrt(dLat*dLat + dLng*dLng)
                const otherDistM = otherDistDeg * DEG_TO_M
                
                // Track min distance
                if (otherDistM < minDistanceObserved) {
                    minDistanceObserved = otherDistM
                }
                // Va chạm cứng (dưới 5m)
                if (otherDistM < 5) {
                    crashCount++
                    if (tick % 50 === 0) { // Log sparingly
                        logs.push(`[Tick ${tick}] CRASH: ${sd.id} and ${other.id} are ${otherDistM.toFixed(2)}m apart!`)
                    }
                }

                if (otherDistDeg < 0.0003) { // Bán kính nhận biết đàn: ~33m
                    neighborCount++
                    cohLat += other.lat
                    cohLng += other.lng
                    alignHeading += other.heading
                    
                    if (otherDistDeg < 0.00015) { // Bán kính tránh va chạm: ~16m
                        // Lực đẩy tỷ lệ nghịch
                        const strength = (0.00015 - otherDistDeg) / 0.00015
                        sepLat += (dLat / otherDistDeg) * strength
                        sepLng += (dLng / otherDistDeg) * strength
                    }
                }
            }
        }

        let destLat = sd.lat
        let destLng = sd.lng

        if (neighborCount > 0) {
            cohLat /= neighborCount
            cohLng /= neighborCount
            alignHeading /= neighborCount
            
            const cLat = cohLat - sd.lat
            const cLng = cohLng - sd.lng
            
            const rad = alignHeading * Math.PI / 180
            const aLat = Math.cos(rad) * 0.0001 
            const aLng = Math.sin(rad) * 0.0001

            const homeLat = (ORIGIN_LAT - sd.lat) * 0.02
            const homeLng = (ORIGIN_LNG - sd.lng) * 0.02

            // THE FLOCKING EQUATION (Current Implementation)
            destLat = sd.lat + cLat * 0.1 + sepLat * 0.00015 + aLat + homeLat
            destLng = sd.lng + cLng * 0.1 + sepLng * 0.00015 + aLng + homeLng
        } else {
            sd.heading = (sd.heading + 2) % 360
            const rad = sd.heading * Math.PI / 180
            destLat = sd.lat + Math.cos(rad) * 0.0001
            destLng = sd.lng + Math.sin(rad) * 0.0001
        }

        // MOVEMENT MATH
        const distLat = destLat - sd.lat
        const distLng = destLng - sd.lng
        const targetHeading = Math.atan2(distLng, distLat) * (180 / Math.PI)
        const normalizedTargetHeading = (targetHeading + 360) % 360

        let headingDiff = normalizedTargetHeading - sd.heading
        if (headingDiff > 180) headingDiff -= 360
        if (headingDiff < -180) headingDiff += 360

        let newHeading = sd.heading
        if (Math.abs(headingDiff) > 30) {
            newHeading += (headingDiff > 0 ? 30 : -30)
            newHeading = (newHeading + 360) % 360
        } else {
            newHeading = normalizedTargetHeading
        }

        const distToDest = Math.sqrt(distLat*distLat + distLng*distLng)
        let maxMove = 0.000015 // 15m/s (10 ticks/s)
        let forwardMoveDist = Math.min(distToDest, maxMove)

        const moveRad = newHeading * (Math.PI / 180)
        let newLat = sd.lat + Math.cos(moveRad) * forwardMoveDist
        let newLng = sd.lng + Math.sin(moveRad) * forwardMoveDist

        nextStates.push({
            id: sd.id,
            lat: newLat,
            lng: newLng,
            alt: sd.alt,
            heading: newHeading,
            speed: 15
        })
    }

    for (let i = 0; i < drones.length; i++) {
        drones[i] = nextStates[i]
    }
}

logs.push(`\n[SIMULATION COMPLETED]`)
logs.push(`Minimum Distance Observed: ${minDistanceObserved.toFixed(2)} meters`)
logs.push(`Total Crash Occurrences (< 5m): ${crashCount}`)

fs.writeFileSync('simulation_log.txt', logs.join('\n'))
console.log(`Simulation finished. Min Dist: ${minDistanceObserved.toFixed(2)}m, Crashes: ${crashCount}`)
