// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises } from '@vue/test-utils';
import { OFFICIAL_THEMES } from './model.ts';
import { skinOf } from '../companion/skins.ts';

const bridge = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn() }));
vi.mock('../bridge.ts', async importOriginal => ({
  ...await importOriginal<typeof import('../bridge.ts')>(),
  desktop: true,
  invoke: bridge.invoke,
  listen: bridge.listen,
}));

function payload(petId: string) {
  const theme = OFFICIAL_THEMES.find(t => t.pets.some(p => p.id === petId))!;
  const definition = theme.pets.find(p => p.id === petId)!;
  return {
    instance: { id: 'test', themeId: theme.id, petId, name: definition.name, size: 210, x: .5, y: .5, layer: 'topmost', enabled: true, expression: 'idle', links: true },
    definition, paused: false, fps: 30, links: true, effect: 'none',
  };
}

async function open(petId: string) {
  bridge.invoke.mockImplementation(async (command: string) => {
    if (command === 'get_pet_instance') return payload(petId);
  });
  await import('../main.ts');
  await vi.waitFor(() => expect(bridge.invoke).toHaveBeenCalledWith('scene_ready'));
  await flushPromises();
}

describe('native pet scene entry', () => {
  beforeEach(() => {
    bridge.invoke.mockReset();
    bridge.listen.mockReset();
    vi.stubGlobal('matchMedia', () => ({ matches: false }));
    vi.stubGlobal('requestAnimationFrame', () => 1);
    vi.stubGlobal('cancelAnimationFrame', vi.fn());
    document.body.innerHTML = '<div id="app"></div>';
    window.history.replaceState({}, '', '/?scene=pet-instance&id=test');
  });

  afterEach(() => {
    window.dispatchEvent(new Event('pagehide'));
    document.body.replaceChildren();
    document.body.className = '';
    window.history.replaceState({}, '', '/');
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    vi.resetModules();
  });

  it.each(['daisy', 'sunflower', 'jade'])('renders the host-selected %s instead of the legacy scene', async petId => {
    await open(petId);
    expect(document.querySelector('.pet-instance-scene .pet-window .pet-actor svg')).not.toBeNull();
    expect(document.querySelector('.floating-stage')).toBeNull();
    const image = document.querySelector('image[data-pet-image]');
    if (petId === 'daisy') {
      expect(image).toBeNull();
      const expected = document.createElement('div');
      const front = skinOf('pot-white-flower').art!.front!;
      expected.innerHTML = typeof front === 'function' ? front('expected') : front;
      expect(document.querySelector('[data-part="plant"]')?.getAttribute('href') === expected.querySelector('[data-part="plant"]')?.getAttribute('href')).toBe(true);
    } else {
      expect(image?.getAttribute('href')).toBe(payload(petId).definition.parts[0].shapes[0].src);
    }
    expect(bridge.listen).not.toHaveBeenCalled();
    expect(bridge.invoke).not.toHaveBeenCalledWith('get_settings');
  });

  it('routes hit tests, pokes, dragging and management to the pet instance', async () => {
    const { PetView } = await import('../pet.ts');
    const hit = vi.spyOn(PetView.prototype, 'hit').mockReturnValue(true);
    const poke = vi.spyOn(PetView.prototype, 'poke');
    await open('sunflower');
    const surface = document.querySelector<HTMLElement>('.pet-window')!;
    surface.setPointerCapture = vi.fn();
    const pointer = () => window.dispatchEvent(new CustomEvent('pet-pointer', { detail: { x: 100, y: 120 } }));
    pointer();
    expect(hit).toHaveBeenCalledWith(100, 120);
    expect(bridge.invoke).toHaveBeenCalledWith('set_pet_hit_test', { hit: true });
    surface.click();
    expect(poke).toHaveBeenCalledOnce();
    surface.dispatchEvent(new PointerEvent('pointerdown', { button: 0, clientX: 100, clientY: 120, screenX: 100, screenY: 120, pointerId: 1 }));
    surface.dispatchEvent(new PointerEvent('pointermove', { screenX: 140, screenY: 150 }));
    await flushPromises();
    surface.dispatchEvent(new PointerEvent('pointerup', { screenX: 140, screenY: 150 }));
    await flushPromises();
    expect(bridge.invoke).toHaveBeenCalledWith('move_pet_instance', expect.objectContaining({ commit: true }));
    surface.click();
    expect(poke).toHaveBeenCalledOnce();
    hit.mockReturnValue(false);
    pointer();
    expect(bridge.invoke).toHaveBeenCalledWith('set_pet_hit_test', { hit: false });
    surface.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true }));
    expect(bridge.invoke).toHaveBeenCalledWith('manage_pet_instance');
  });

  it('refreshes the selected plant when the host changes its definition', async () => {
    await open('daisy');
    window.dispatchEvent(new CustomEvent('pet-config', { detail: payload('jade') }));
    await flushPromises();
    expect(document.querySelector('image[data-pet-image]')?.getAttribute('href')).toBe(payload('jade').definition.parts[0].shapes[0].src);
  });
});
