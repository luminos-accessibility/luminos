import type { JSX } from 'react';

import { FrameTimingDisplay } from '../components/magnification/FrameTimingDisplay';
import { MagnificationModeSelector } from '../components/magnification/MagnificationModeSelector';
import { ZoomLevelSlider } from '../components/magnification/ZoomLevelSlider';

/**
 * The Magnification settings page — the single Phase-0 page. Hosts the zoom
 * slider, the mode selector, and (in debug builds) the frame-timing readout.
 * The page is labeled by its heading via `aria-labelledby` for landmark
 * navigation.
 */
export const MagnificationPage = (): JSX.Element => {
  return (
    <section aria-labelledby="magnification-heading">
      <h1 id="magnification-heading">Magnification</h1>
      <ZoomLevelSlider />
      <MagnificationModeSelector />
      <FrameTimingDisplay />
    </section>
  );
};
