/*
 * Purpose: Trang xác thực (Login Page) của Clone Drone Engage.
 *          Giao diện dark-mode hiện đại, gửi POST lên Auth Server,
 *          lưu kết quả vào Zustand store, và chuyển hướng về Dashboard.
 * Dependencies: React, React Router, authService.ts, authStore.ts.
 * Author: Antigravity (thay Codex)
 * Date: 2026-06-07
 */

import { useState, type FormEvent } from 'react'
import { useNavigate } from 'react-router-dom'
import { authenticateWithServer } from './authService'
import { useAuthStore } from '../../store/authStore'
import { logClientEvent } from '../../utils/logger'

export function AuthPage() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')

  const { isLoading, error, setAuthInfo, setError, setLoading } = useAuthStore()
  const navigate = useNavigate()

  /**
   * Xử lý sự kiện submit form đăng nhập.
   * Gọi authService → lưu state vào Zustand → chuyển hướng.
   */
  async function handleSubmit(e: FormEvent) {
    e.preventDefault()

    // Validation cơ bản
    if (!email.trim() || !password.trim()) {
      setError('Vui lòng điền đầy đủ Email và Access Code.')
      return
    }

    setLoading(true)
    setError('')

    try {
      const authData = await authenticateWithServer(email.trim(), password.trim())

      // Lưu thông tin Communication Server vào Zustand
      setAuthInfo(authData.cs, authData.sid)

      logClientEvent('info', 'AUTH_REDIRECT', { target: '/dashboard' })

      // Chuyển hướng sang Dashboard
      navigate('/dashboard', { replace: true })
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Đã xảy ra lỗi không xác định.'
      setError(message)
      logClientEvent('error', 'AUTH_SUBMIT_ERROR', { message })
    }
  }

  return (
    <main className="relative flex min-h-screen items-center justify-center overflow-hidden bg-slate-950 px-4 py-10">
      {/* Animated background gradient orbs */}
      <div className="pointer-events-none absolute -top-40 -left-40 h-96 w-96 rounded-full bg-cyan-500/10 blur-3xl animate-pulse" />
      <div className="pointer-events-none absolute -bottom-40 -right-40 h-96 w-96 rounded-full bg-indigo-500/10 blur-3xl animate-pulse" style={{ animationDelay: '1s' }} />

      <section className="relative z-10 w-full max-w-md">
        {/* Card container with glassmorphism */}
        <div className="rounded-2xl border border-slate-700/50 bg-slate-900/80 p-8 shadow-2xl backdrop-blur-xl">

          {/* Logo & Title */}
          <div className="mb-8 text-center">
            <div className="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-xl bg-gradient-to-br from-cyan-400 to-indigo-500 shadow-lg shadow-cyan-500/25">
              <svg xmlns="http://www.w3.org/2000/svg" className="h-7 w-7 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
              </svg>
            </div>
            <p className="text-xs font-bold uppercase tracking-[0.2em] text-cyan-400">
              Clone Drone Engage
            </p>
            <h1 className="mt-2 text-2xl font-bold text-white">
              Đăng Nhập
            </h1>
            <p className="mt-1 text-sm text-slate-400">
              Kết nối vào hệ thống điều khiển drone bầy đàn
            </p>
          </div>

          {/* Error message */}
          {error && (
            <div className="mb-6 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
              <span className="mr-2">⚠</span>
              {error}
            </div>
          )}

          {/* Login Form */}
          <form onSubmit={handleSubmit} className="space-y-5">
            {/* Email field */}
            <div>
              <label htmlFor="email-input" className="mb-2 block text-sm font-medium text-slate-300">
                Email
              </label>
              <input
                id="email-input"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="you@example.com"
                autoComplete="email"
                disabled={isLoading}
                className="w-full rounded-lg border border-slate-600/50 bg-slate-800/50 px-4 py-3 text-sm text-white
                           placeholder-slate-500 outline-none transition-all duration-200
                           focus:border-cyan-500/50 focus:ring-2 focus:ring-cyan-500/20
                           disabled:cursor-not-allowed disabled:opacity-50"
              />
            </div>

            {/* Access Code field */}
            <div>
              <label htmlFor="password-input" className="mb-2 block text-sm font-medium text-slate-300">
                Access Code
              </label>
              <input
                id="password-input"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="••••••••"
                autoComplete="current-password"
                disabled={isLoading}
                className="w-full rounded-lg border border-slate-600/50 bg-slate-800/50 px-4 py-3 text-sm text-white
                           placeholder-slate-500 outline-none transition-all duration-200
                           focus:border-cyan-500/50 focus:ring-2 focus:ring-cyan-500/20
                           disabled:cursor-not-allowed disabled:opacity-50"
              />
            </div>

            {/* Submit button */}
            <button
              id="login-button"
              type="submit"
              disabled={isLoading}
              className="relative w-full overflow-hidden rounded-lg bg-gradient-to-r from-cyan-500 to-indigo-500
                         px-4 py-3 text-sm font-semibold text-white shadow-lg shadow-cyan-500/25
                         transition-all duration-300 hover:shadow-xl hover:shadow-cyan-500/30
                         hover:brightness-110 active:scale-[0.98]
                         disabled:cursor-not-allowed disabled:opacity-60"
            >
              {isLoading ? (
                <span className="flex items-center justify-center gap-2">
                  <svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
                    <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" className="opacity-25" />
                    <path fill="currentColor" className="opacity-75" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
                  </svg>
                  Đang kết nối...
                </span>
              ) : (
                'Đăng Nhập'
              )}
            </button>
          </form>

          {/* Footer info */}
          <div className="mt-6 border-t border-slate-700/50 pt-4 flex flex-col items-center gap-3">
            {/* Nút Bypass để test cục bộ */}
            <button
              type="button"
              onClick={() => {
                setAuthInfo({ f: 'mock', g: 'localhost', h: 8080 }, 'dev_session')
                navigate('/dashboard', { replace: true })
              }}
              className="text-sm font-semibold text-cyan-400 bg-cyan-900/30 hover:bg-cyan-800/40 px-4 py-2 rounded-lg border border-cyan-500/30 transition-all w-full"
            >
              🚀 DEV: Bypass Login
            </button>

            <p className="text-center text-xs text-slate-500">
              Tài khoản được tạo tại{' '}
              <a
                href="https://cloud.ardupilot.org:8001/accounts"
                target="_blank"
                rel="noopener noreferrer"
                className="text-cyan-400/70 underline transition-colors hover:text-cyan-300"
              >
                cloud.ardupilot.org
              </a>
            </p>
          </div>
        </div>

        {/* Server status indicator */}
        <div className="mt-4 flex items-center justify-center gap-2 text-xs text-slate-500">
          <span className="inline-block h-2 w-2 rounded-full bg-emerald-500/80 shadow-sm shadow-emerald-500/50" />
          cloud.ardupilot.org:19408
        </div>
      </section>
    </main>
  )
}
