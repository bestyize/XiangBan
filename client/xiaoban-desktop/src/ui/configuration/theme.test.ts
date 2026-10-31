import { expect, it } from 'vitest';
import { tokensFor } from '../design-tokens.ts';
import { accents } from '../appearance.ts';
import { configurationThemeFor } from '../theme.ts';

function luminance(hex: string) {
  const channels = [1, 3, 5].map(start => parseInt(hex.slice(start, start + 2), 16) / 255)
    .map(value => value <= .04045 ? value / 12.92 : ((value + .055) / 1.055) ** 2.4);
  return channels[0] * .2126 + channels[1] * .7152 + channels[2] * .0722;
}
function contrast(a: string, b: string) {
  const light = luminance(a), dark = luminance(b);
  return (Math.max(light, dark) + .05) / (Math.min(light, dark) + .05);
}
it.each([false, true])('keeps configuration text and placeholders at least 4.5:1 across all accents (dark=%s)', dark => {
  for (const accent of accents) {
    const t = tokensFor(dark, accent.id), theme = configurationThemeFor(dark, accent.id);
    const textColors = [t.textPrimary, t.configurationTextSecondary, t.configurationTextTertiary, theme.common!.placeholderColor!];
    for (const background of [t.configurationCanvas, t.configurationSurface, t.configurationSidebar, t.configurationGroup, t.configurationControl]) {
      for (const text of textColors) expect(contrast(text, background), `${accent.id}: ${text} on ${background}`).toBeGreaterThanOrEqual(4.5);
    }
  }
});
