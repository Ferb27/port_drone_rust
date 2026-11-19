/*
 * Purpose: TypeScript interfaces cho Auth Server response và request payload.
 *          Dựa trên tài liệu Andruav protocol đã xác minh trong README.md
 *          và docs/architecture/data-flow.md.
 * Dependencies: None.
 * Author: Antigravity (thay Codex)
 * Date: 2026-06-07
 */

/** Thông tin Communication Server trả về sau khi xác thực thành công */
export interface CommServerInfo {
  /** Host/IP của Communication Server (ví dụ: "airgap.droneengage.com") */
  readonly g: string
  /** Port của Communication Server (ví dụ: "9966") */
  readonly h: string
  /** Temporary login key — mã dùng một lần để WebSocket xác thực */
  readonly f: string
}

/** Response JSON từ Auth Server khi POST /wl/<groupName> thành công */
export interface AuthResponse {
  /** Communication Server info */
  readonly cs: CommServerInfo
  /** Session ID */
  readonly sid: string
}

/** Response JSON khi Auth Server trả về lỗi */
export interface AuthErrorResponse {
  readonly em: string
  readonly cm: 'error'
}

/** Request body gửi lên Auth Server */
export interface AuthRequestBody {
  readonly email: string
  readonly password: string
  readonly GCS_ID: string
  readonly app_name: string
  readonly app_ver: string
}
