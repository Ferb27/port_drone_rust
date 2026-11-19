/*
 * Purpose: Shared TypeScript types for verified Andruav envelope shapes.
 * Dependencies: None.
 * Author: Codex
 * Date: 2026-06-07
 */

export type AndruavCommandType = 's' | 'c'

export type AndruavEnvelope<TPayload = unknown> = {
  readonly _ty: AndruavCommandType
  readonly msg?: TPayload
  readonly messageType?: number
  readonly sender?: string
  readonly g?: string
  readonly i?: string
}
