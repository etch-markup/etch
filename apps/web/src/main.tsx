import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { App } from './App.js';

const root = document.getElementById('root');

if (!root) {
  throw new Error('Failed to find the root element for the Etch web app.');
}

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>
);
