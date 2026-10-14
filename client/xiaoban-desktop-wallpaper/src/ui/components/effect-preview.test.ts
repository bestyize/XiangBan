// @vitest-environment happy-dom
import { mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { drawEffects } from '../../effects/draw.ts';
import { parsePack } from '../../effects/parse.ts';
import EffectPreview from './EffectPreview.vue';

vi.mock('../../effects/draw.ts', () => ({ drawEffects: vi.fn() }));

const parsed = parsePack({ schemaVersion: 1, id: 'snow', name: '雪', layers: [{ count: 8, shape: { kind: 'circle' }, colors: ['#fff'] }] });
if (!parsed.ok) throw new Error(parsed.error);
const pack = parsed.pack;

describe('effect card animation lifecycle', () => {
  let wrapper: VueWrapper | undefined;
  let frames: Map<number, FrameRequestCallback>;
  let frameId: number;
  beforeEach(() => {
    frames = new Map(); frameId = 0;
    vi.mocked(drawEffects).mockClear();
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue({ setTransform: vi.fn() } as unknown as CanvasRenderingContext2D);
    vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => { frames.set(++frameId, callback); return frameId; });
    vi.stubGlobal('cancelAnimationFrame', (id: number) => { frames.delete(id); });
  });
  afterEach(() => { wrapper?.unmount(); wrapper = undefined; vi.restoreAllMocks(); vi.unstubAllGlobals(); });

  it('draws a static frame by default without requesting animation frames', async () => {
    wrapper = mount(EffectPreview, { props: { pack } });
    expect(wrapper.find('canvas').exists()).toBe(true);
    expect(drawEffects).toHaveBeenCalled();
    expect(frames.size).toBe(0);
    await wrapper.setProps({ fps: 30 });
    expect(frames.size).toBe(0);
  });

  it('starts on demand, stops on deselection or global pause, and cleans up on unmount', async () => {
    wrapper = mount(EffectPreview, { props: { pack } });
    await wrapper.setProps({ playing: true });
    expect(frames.size).toBe(1);
    const [id, tick] = [...frames][0]!;
    frames.delete(id); tick(100);
    expect(frames.size).toBe(1);
    await wrapper.setProps({ playing: false });
    expect(frames.size).toBe(0);
    expect(wrapper.find('canvas').exists()).toBe(true);
    await wrapper.setProps({ playing: true, paused: true });
    expect(frames.size).toBe(0);
    await wrapper.setProps({ paused: false });
    expect(frames.size).toBe(1);
    wrapper.unmount(); wrapper = undefined;
    expect(frames.size).toBe(0);
  });

  it('stops while hidden and resumes only a requested preview', async () => {
    let hidden = false;
    vi.spyOn(document, 'hidden', 'get').mockImplementation(() => hidden);
    wrapper = mount(EffectPreview, { props: { pack, playing: true } });
    expect(frames.size).toBe(1);
    hidden = true; document.dispatchEvent(new Event('visibilitychange'));
    expect(frames.size).toBe(0);
    hidden = false; document.dispatchEvent(new Event('visibilitychange'));
    expect(frames.size).toBe(1);
    await wrapper.setProps({ playing: false });
    document.dispatchEvent(new Event('visibilitychange'));
    expect(frames.size).toBe(0);
  });
});
