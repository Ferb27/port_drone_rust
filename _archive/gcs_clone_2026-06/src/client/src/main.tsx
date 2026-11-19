/*
 * Purpose: React application entry point and global providers.
 * Dependencies: React, React DOM, React Router, Leaflet CSS, App.
 * Author: Codex
 * Date: 2026-06-07
 */

import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import 'leaflet/dist/leaflet.css'
import './index.css'
import App from './App.tsx'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </StrictMode>,
)
