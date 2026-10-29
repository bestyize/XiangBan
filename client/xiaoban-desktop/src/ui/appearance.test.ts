// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest';
import { appearanceMode, followHost, isDark, setAppearance } from './appearance.ts';

describe('appearance shared between windows', () => {
  it('follows the choice the host reports, which is how a window with its own storage learns it', () => {
    setAppearance('light');
    followHost('dark');
    expect(appearanceMode.value).toBe('dark');
    expect(isDark.value).toBe(true);
    followHost('light');
    expect(isDark.value).toBe(false);
    followHost('system');
    expect(appearanceMode.value).toBe('system');
  });

  it('ignores anything that is not one of the three choices', () => {
    setAppearance('light');
    for (const bad of ['sepia', '', null, undefined, 3, {}]) followHost(bad);
    expect(appearanceMode.value).toBe('light');
  });
});
