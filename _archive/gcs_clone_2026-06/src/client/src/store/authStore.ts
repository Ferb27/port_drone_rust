/*
 * Purpose: Zustand store for authentication session state.
 *          Lưu trữ thông tin Communication Server (host, port, temp_key)
 *          sau khi xác thực thành công với Auth Server.
 *          Theo thiết kế trong docs/architecture/data-flow.md.
 * Dependencies: Zustand, types/auth.ts.
 * Author: Codex (original), Antigravity (updated)
 * Date: 2026-06-07
 */

import { create } from 'zustand'
import type { CommServerInfo } from '../types/auth'

interface AuthState {
  /** Đã xác thực thành công hay chưa */
  isAuthenticated: boolean
  /** Session ID từ Auth Server */
  sessionId: string | null
  /** Thông tin Communication Server (host, port, temp_key) */
  commServer: CommServerInfo | null
  /** Lỗi xác thực gần nhất (nếu có) */
  error: string | null
  /** Đang trong quá trình gửi request */
  isLoading: boolean
}

interface AuthActions {
  /** Lưu kết quả xác thực thành công */
  setAuthInfo: (commServer: CommServerInfo, sessionId: string) => void
  /** Ghi nhận lỗi */
  setError: (error: string) => void
  /** Bật trạng thái loading */
  setLoading: (loading: boolean) => void
  /** Xóa toàn bộ session (logout) */
  clearSession: () => void
}

const initialState: AuthState = {
  isAuthenticated: false,
  sessionId: null,
  commServer: null,
  error: null,
  isLoading: false,
}

export const useAuthStore = create<AuthState & AuthActions>((set) => ({
  ...initialState,

  setAuthInfo: (commServer, sessionId) =>
    set({
      isAuthenticated: true,
      commServer,
      sessionId,
      error: null,
      isLoading: false,
    }),

  setError: (error) =>
    set({
      error,
      isLoading: false,
    }),

  setLoading: (isLoading) =>
    set({ isLoading }),

  clearSession: () =>
    set(initialState),
}))
