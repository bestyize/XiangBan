// @vitest-environment happy-dom
import { describe, expect, it, vi } from 'vitest';
import { appearanceMode, publishAppearanceTo, reconcile, setAppearance } from './appearance.ts';

describe('console colour scheme and the host', () => {
  it('sends the choice made in the console to the host, so the other windows can follow', () => {
    const publish = vi.fn();
    publishAppearanceTo(publish);
    setAppearance('dark');
    expect(publish).toHaveBeenCalledWith('dark');
    expect(appearanceMode.value).toBe('dark');
  });

  it('carries a choice the console already had up to a host that has none yet, instead of overwriting it', () => {
    expect(reconcile({ appearance: 'system', appearanceKept: false }, 'dark')).toEqual({ send: 'dark' });
    expect(reconcile({ appearance: 'system', appearanceKept: false }, 'light')).toEqual({ send: 'light' });
  });

  it('has nothing to send when the console was on the system setting, and then the host rules', () => {
    expect(reconcile({ appearance: 'system', appearanceKept: false }, 'system')).toEqual({});
    expect(reconcile({ appearance: 'dark', appearanceKept: true }, 'light')).toEqual({ adopt: 'dark' });
    expect(reconcile({ appearance: 'dark', appearanceKept: true }, 'dark')).toEqual({});
  });
});
