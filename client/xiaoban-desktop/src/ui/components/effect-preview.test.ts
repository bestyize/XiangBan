// @vitest-environment happy-dom
import { mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { defineComponent, h, KeepAlive, nextTick, ref } from 'vue';
import { drawEffects } from '../../effects/draw.ts';
import { EffectsLayer } from '../../effects/layer.ts';
import { parsePack } from '../../effects/parse.ts';
import EffectPreview from './EffectPreview.vue';

vi.mock('../../effects/draw.ts', () => ({ drawEffects: vi.fn() }));
const parsed = parsePack({ schemaVersion: 1, id: 'snow', name: '雪', layers: [{ count: 8, shape: { kind: 'circle' }, colors: ['#fff'] }] });
if (!parsed.ok) throw new Error(parsed.error);
const pack = parsed.pack;
let sample = 0;
const freshPack = () => ({ ...pack, id: `snow-${++sample}` });

describe('effect card animation lifecycle', () => {
  let wrapper: VueWrapper | undefined;
  let frames: Map<number, FrameRequestCallback>;
  let frameId: number;
  beforeEach(() => {
    vi.useFakeTimers();
    frames = new Map(); frameId = 0;
    vi.mocked(drawEffects).mockClear();
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue({ setTransform: vi.fn() } as unknown as CanvasRenderingContext2D);
    vi.spyOn(HTMLCanvasElement.prototype, 'toBlob').mockImplementation(callback => callback(new Blob(['still'])));
    vi.spyOn(URL, 'createObjectURL').mockImplementation(() => `blob:still-${++sample}`);
    vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => { frames.set(++frameId, callback); return frameId; });
    vi.stubGlobal('cancelAnimationFrame', (id: number) => { frames.delete(id); });
  });
  afterEach(async () => {
    wrapper?.unmount(); wrapper = undefined;
    await vi.runAllTimersAsync();
    vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals();
  });

  it('mounts no canvas and generates a cached image in a later task', async () => {
    const current = freshPack();
    const update = vi.spyOn(EffectsLayer.prototype, 'update');
    wrapper = mount(EffectPreview, { props: { pack: current } });
    expect(wrapper.find('canvas').exists()).toBe(false);
    expect(drawEffects).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(16);
    expect(wrapper.find('img').exists()).toBe(true);
    expect(drawEffects).toHaveBeenCalledTimes(1);
    expect(frames.size).toBe(0);
    const url = wrapper.get('img').attributes('src');
    wrapper.unmount();
    wrapper = mount(EffectPreview, { props: { pack: JSON.parse(JSON.stringify(current)) } });
    await nextTick();
    expect(wrapper.get('img').attributes('src')).toBe(url);
    await wrapper.setProps({ fps: 30 });
    expect(drawEffects).toHaveBeenCalledTimes(1);
    expect(update).not.toHaveBeenCalled();
  });

  it('starts on demand, removes the canvas on deselection or global pause, and cleans up on unmount', async () => {
    wrapper = mount(EffectPreview, { props: { pack: freshPack() } });
    await wrapper.setProps({ playing: true });
    expect(frames.size).toBe(1);
    expect(wrapper.find('canvas').exists()).toBe(true);
    const [id, tick] = [...frames][0]!;
    frames.delete(id); tick(100);
    expect(frames.size).toBe(1);
    await wrapper.setProps({ playing: false });
    expect(frames.size).toBe(0);
    expect(wrapper.find('canvas').exists()).toBe(false);
    await wrapper.setProps({ playing: true, paused: true });
    expect(frames.size).toBe(0);
    expect(wrapper.find('canvas').exists()).toBe(false);
    await wrapper.setProps({ paused: false });
    expect(frames.size).toBe(1);
    wrapper.unmount(); wrapper = undefined;
    expect(frames.size).toBe(0);
  });

  it('stops while hidden and resumes only a requested preview', async () => {
    let hidden = false;
    vi.spyOn(document, 'hidden', 'get').mockImplementation(() => hidden);
    wrapper = mount(EffectPreview, { props: { pack: freshPack(), playing: true } });
    expect(frames.size).toBe(1);
    hidden = true; document.dispatchEvent(new Event('visibilitychange'));
    expect(frames.size).toBe(0);
    hidden = false; document.dispatchEvent(new Event('visibilitychange'));
    expect(frames.size).toBe(1);
    await wrapper.setProps({ playing: false });
    document.dispatchEvent(new Event('visibilitychange'));
    expect(frames.size).toBe(0);
  });

  it('destroys on KeepAlive deactivation without redrawing, and respects playback changes while inactive', async () => {
    const visible = ref(true), playing = ref(true), current = freshPack();
    const update = vi.spyOn(EffectsLayer.prototype, 'update');
    const destroy = vi.spyOn(EffectsLayer.prototype, 'destroy');
    wrapper = mount(defineComponent({ setup: () => () => h(KeepAlive, null, {
      default: () => visible.value ? h(EffectPreview, { pack: current, playing: playing.value }) : h('div'),
    }) }));
    expect(frames.size).toBe(1);
    expect(update).not.toHaveBeenCalled();
    const draws = vi.mocked(drawEffects).mock.calls.length;
    visible.value = false; await nextTick();
    expect(frames.size).toBe(0);
    expect(destroy).toHaveBeenCalledTimes(1);
    expect(update).not.toHaveBeenCalled();
    await vi.runAllTimersAsync();
    expect(drawEffects).toHaveBeenCalledTimes(draws);
    playing.value = false; visible.value = true; await nextTick();
    expect(wrapper.find('canvas').exists()).toBe(false);
    expect(frames.size).toBe(0);
    await vi.runAllTimersAsync();
    expect(wrapper.find('img').exists()).toBe(true);
  });

  it('discards an old in-flight thumbnail when the pack changes', async () => {
    const callbacks: BlobCallback[] = [];
    vi.mocked(HTMLCanvasElement.prototype.toBlob).mockImplementation(callback => { callbacks.push(callback); });
    wrapper = mount(EffectPreview, { props: { pack: freshPack() } });
    await vi.advanceTimersByTimeAsync(16);
    await wrapper.setProps({ pack: freshPack() });
    callbacks[0](new Blob()); await nextTick();
    expect(wrapper.find('img').exists()).toBe(false);
    await vi.advanceTimersByTimeAsync(16);
    callbacks[1](new Blob()); await nextTick();
    expect(wrapper.find('img').exists()).toBe(true);
  });
});
