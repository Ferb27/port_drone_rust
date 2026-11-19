/*
 * Purpose: Service layer cho việc gọi HTTP POST lên Auth Server,
 *          lấy temp_key và thông tin Communication Server.
 *          Logic tách biệt khỏi React component theo thiết kế data-flow.md.
 *
 *          ĐÃ XÁC MINH: Endpoint chính xác là POST /w/wl/
 *          Body fields: acc (email), pwd (password), gr ("1"), app ("andruav"),
 *                       ver (version), ex (app_name), at ("g")
 *          Source: Reverse-engineered từ JS bundle gốc tại
 *          cloud.ardupilot.org:8001/static/js/main.7909d061.js
 *
 * Dependencies: types/auth.ts, utils/logger.ts.
 * Author: Antigravity (thay Codex)
 * Date: 2026-06-07
 */

import type { AuthResponse, CommServerInfo } from '../../types/auth'
import { logClientEvent } from '../../utils/logger'

/** Đọc base URL từ biến môi trường Vite */
const AUTH_BASE_URL = import.meta.env.VITE_AUTH_SERVER_URL

/**
 * Raw response shape từ Auth Server (đã xác minh từ source code gốc).
 * Field names ngắn vì protocol Andruav dùng format compact.
 */
interface RawAuthResponse {
  /** Error code: 0 = success */
  e: number
  /** Session ID */
  sid?: string
  /** Permission string */
  per?: string
  /** Permission bitmask */
  prm?: number
  /** Communication Server info */
  cs?: {
    /** Temp login key */
    f: string
    /** Host */
    g: string
    /** Port */
    h: number
  }
  /** Command indicator */
  cm?: string
  /** Error message (khi lỗi) */
  em?: string
}

/**
 * Gửi yêu cầu xác thực lên DroneEngage Authentication Server.
 *
 * Endpoint: POST {AUTH_BASE_URL}/w/wl/
 * Body: { acc, pwd, gr, app, ver, ex, at }
 *
 * @param email - Email đã đăng ký trên DroneEngage
 * @param password - Access code (password)
 * @returns AuthResponse nếu thành công (đã normalize field names)
 * @throws Error nếu server trả về lỗi hoặc mạng bị lỗi
 */
export async function authenticateWithServer(
  email: string,
  password: string,
): Promise<AuthResponse> {
  const url = `${AUTH_BASE_URL}/w/wl/`

  logClientEvent('info', 'AUTH_REQUEST', { url, email })

  // Body fields theo đúng protocol Andruav (đã xác minh từ source code gốc)
  const body = {
    acc: email,         // Account (email)
    pwd: password,      // Password (access code)
    gr: '1',            // Group
    app: 'andruav',     // App identifier
    ver: '5.0.0',       // App version (phải để 5.0.0 server mới cho phép)
    ex: 'Clone Drone Engage Web Client', // App display name
    at: 'g',            // Auth type: 'g' = GCS
  }

  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    })

    const data: RawAuthResponse = await response.json()

    // Kiểm tra error code
    if (data.e !== 0 || !data.cs) {
      const errorMsg = data.em || 'Xác thực thất bại. Kiểm tra lại email và access code.'
      logClientEvent('error', 'AUTH_FAILED', { errorMsg, rawResponse: data })
      throw new Error(errorMsg)
    }

    // Normalize response sang interface sạch
    const commServer: CommServerInfo = {
      g: data.cs.g,
      h: String(data.cs.h),
      f: data.cs.f,
    }

    const result: AuthResponse = {
      cs: commServer,
      sid: data.sid ?? '',
    }

    logClientEvent('info', 'AUTH_SUCCESS', {
      host: commServer.g,
      port: commServer.h,
      sid: result.sid,
    })

    return result
  } catch (error) {
    // Nếu là network error (fetch thất bại hoàn toàn)
    if (error instanceof TypeError) {
      logClientEvent('error', 'AUTH_NETWORK_ERROR', { message: error.message })
      throw new Error(
        'Không thể kết nối đến Auth Server. Kiểm tra lại kết nối mạng hoặc URL server.',
      )
    }
    throw error
  }
}
