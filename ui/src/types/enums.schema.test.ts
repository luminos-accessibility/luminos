import { describe, expect, test } from 'vitest';

import {
  colorFilterTypeSchema,
  interpolationModeSchema,
  magnificationModeSchema,
  trackingModeSchema,
} from './enums';

describe('enums.schema', () => {
  const VALID_MODES = ['FullScreen', 'Docked', 'Lens'] as const;

  test.each(VALID_MODES)('magnification_mode_schema_accepts_%s', (mode) => {
    expect(magnificationModeSchema.safeParse(mode).success).toBe(true);
  });

  test('magnification_mode_schema_rejects_snake_case', () => {
    // Wire format is PascalCase; snake_case must be rejected so drift surfaces.
    expect(magnificationModeSchema.safeParse('full_screen').success).toBe(false);
  });

  test('magnification_mode_schema_rejects_unknown_variant', () => {
    expect(magnificationModeSchema.safeParse('Picture').success).toBe(false);
  });

  test('tracking_mode_schema_accepts_text_caret_pascal_case', () => {
    expect(trackingModeSchema.safeParse('TextCaret').success).toBe(true);
    expect(trackingModeSchema.safeParse('text_caret').success).toBe(false);
  });

  test('color_filter_type_schema_accepts_all_variants', () => {
    for (const variant of ['None', 'Invert', 'SmartInvert', 'Grayscale', 'HighContrast', 'Custom']) {
      expect(colorFilterTypeSchema.safeParse(variant).success).toBe(true);
    }
  });

  test('interpolation_mode_schema_rejects_lowercase', () => {
    expect(interpolationModeSchema.safeParse('bilinear').success).toBe(false);
    expect(interpolationModeSchema.safeParse('Bilinear').success).toBe(true);
  });
});
