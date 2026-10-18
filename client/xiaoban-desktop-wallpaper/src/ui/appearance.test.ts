// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest';
import { appearanceMode, setAppearance } from './appearance.ts';

describe('appearance shared between windows', () => {
  it('follows a choice made in another window, which arrives as a storage event', () => {
    setAppearance('light');
    localStorage.setItem('xiaoban.control.appearance', 'dark');
    window.dispatchEvent(new StorageEvent('storage', { key: 'xiaoban.control.appearance', newValue: 'dark' }));
    expect(appearanceMode.value).toBe('dark');
    localStorage.setItem('xiaoban.control.appearance', 'nonsense');
    window.dispatchEvent(new StorageEvent('storage', { key: 'xiaoban.control.appearance', newValue: 'nonsense' }));
    expect(appearanceMode.value).toBe('system');
  });

  it('ignores unrelated storage keys', () => {
    setAppearance('light');
    window.dispatchEvent(new StorageEvent('storage', { key: 'something.else', newValue: 'dark' }));
    expect(appearanceMode.value).toBe('light');
  });
});
