/**
 * Why Did You Render - React Performance Debugging
 *
 * This file is imported before React to enable re-render tracking in development.
 * In production, this is a no-op.
 */

// Only enable in development mode
if (import.meta.env.DEV) {
  // Uncomment and install @welldone-software/why-did-you-render if needed for debugging
  // import React from 'react';
  // import whyDidYouRender from '@welldone-software/why-did-you-render';
  // whyDidYouRender(React, {
  //   trackAllPureComponents: true,
  // });
}

export {};
