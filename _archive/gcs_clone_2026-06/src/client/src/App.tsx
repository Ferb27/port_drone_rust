/*
 * Purpose: Root component and route shell for the Clone Drone Engage client.
 *          Xử lý routing giữa Login và Dashboard, bao gồm protected route.
 * Dependencies: React Router, authStore.ts, AuthPage, DashboardPage.
 * Author: Codex (original), Antigravity (updated)
 * Date: 2026-06-07
 */

import { Navigate, Route, Routes } from 'react-router-dom'
import { AuthPage } from './features/auth/AuthPage'
import { DashboardPage } from './features/dashboard/DashboardPage'
import { useAuthStore } from './store/authStore'

/**
 * Protected Route wrapper — chỉ cho phép truy cập nếu đã xác thực.
 * Nếu chưa xác thực, redirect về /login.
 */
function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)
  if (!isAuthenticated) {
    return <Navigate to="/login" replace />
  }
  return <>{children}</>
}

function App() {
  return (
    <Routes>
      <Route path="/login" element={<AuthPage />} />
      <Route
        path="/dashboard"
        element={
          <ProtectedRoute>
            <DashboardPage />
          </ProtectedRoute>
        }
      />
      <Route path="*" element={<Navigate to="/login" replace />} />
    </Routes>
  )
}

export default App
