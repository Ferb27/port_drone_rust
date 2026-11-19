/*
 * Purpose: TypeScript type declarations for Vite environment variables.
 * Dependencies: Vite.
 * Author: Antigravity (thay Codex)
 * Date: 2026-06-07
 */

/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Base URL of the DroneEngage Authentication Server (e.g. https://cloud.ardupilot.org:19408) */
  readonly VITE_AUTH_SERVER_URL: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
