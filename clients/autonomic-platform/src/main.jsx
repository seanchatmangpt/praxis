import React from 'react';
import { createRoot } from 'react-dom/client';
import AutonomicPlatform from './AutonomicPlatform.js';

// dataMode 'praxis' (default) reads real repo artifacts through
// src/praxis-adapter.js; pass ?mode=mock for the simulation-only demo
// (every screen then carries the NON-STANDING banner).
const mode = new URLSearchParams(window.location.search).get('mode') === 'mock' ? 'mock' : 'praxis';

createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <AutonomicPlatform dataMode={mode} />
  </React.StrictMode>,
);
