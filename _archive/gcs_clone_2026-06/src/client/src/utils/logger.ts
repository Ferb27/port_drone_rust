/*
 * Purpose: Client-side logger facade for consistent diagnostics.
 * Dependencies: Browser console.
 * Author: Codex
 * Date: 2026-06-07
 */

type LogLevel = 'info' | 'warn' | 'error'

/**
 * Writes a structured diagnostic message through the browser console.
 */
export function logClientEvent(level: LogLevel, action: string, detail: unknown) {
  const entry = {
    timestamp: new Date().toISOString(),
    action,
    detail,
  }

  console[level](`[CloneDroneEngage] ${action}`, entry)
}
