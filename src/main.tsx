/**
 * Stone - React Renderer Entry Point
 */

// Must be first import for why-did-you-render to work
import './wdyr';

import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from '@/App';
import './index.css';
import 'tippy.js/dist/tippy.css';

// Defer font loading - not critical for initial render
// Fonts will load in background after app mounts
// Use requestIdleCallback if available, otherwise setTimeout as fallback
const loadFonts = () => {
  import('@fontsource/inter/400.css');
  import('@fontsource/inter/500.css');
  import('@fontsource/inter/600.css');
  import('@fontsource/inter/700.css');
  import('@fontsource/barlow/400.css');
  import('@fontsource/barlow/500.css');
  import('@fontsource/barlow/600.css');
  import('@fontsource/barlow-condensed/400.css');
  import('@fontsource/barlow-condensed/500.css');
  import('@fontsource/barlow-condensed/600.css');
  import('@fontsource/barlow-condensed/700.css');
  import('@fontsource/barlow-semi-condensed/400.css');
  import('@fontsource/barlow-semi-condensed/500.css');
  import('@fontsource/barlow-semi-condensed/600.css');
  import('@fontsource/barlow-semi-condensed/700.css');
  import('@fontsource/patrick-hand/latin.css');
  import('@fontsource/fira-code/400.css');
  import('@fontsource/fira-code/500.css');
  import('@fontsource/fira-code/600.css');
};

if (typeof requestIdleCallback !== 'undefined') {
  requestIdleCallback(loadFonts);
} else {
  setTimeout(loadFonts, 1);
}

const root = document.getElementById('root');

if (!root) {
  throw new Error('Root element not found');
}

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
