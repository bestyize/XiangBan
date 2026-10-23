import { describe, expect, it, vi } from 'vitest';
import { pcm16 } from './models.ts';

describe('native turn models', () => {
  it('sends audio as clamped 16-bit samples', () => {
    expect(pcm16(Float32Array.from([0, 0.5, -1, 2]))).toEqual([0, 16384, -32767, 32767]);
  });
  it('is unavailable when the host cannot load the models, and asks again next time', async () => {
    vi.resetModules();
    const { loadTurnModels } = await import('./models.ts');
    const invoke = vi.fn(async () => false) as never;
    expect(await loadTurnModels(invoke)).toBeNull();
    expect(await loadTurnModels(invoke)).toBeNull();
    expect((invoke as unknown as ReturnType<typeof vi.fn>).mock.calls.filter(([c]) => c === 'voice_turn_ready')).toHaveLength(2);
  });
  it('gives each listener its own VAD stream and judges turns on the host', async () => {
    vi.resetModules();
    const { loadTurnModels } = await import('./models.ts');
    const invoke = vi.fn(async (command: string, _args?: Record<string, unknown>) => (command === 'voice_turn_ready' ? true : command === 'voice_vad' ? [0.9] : command === 'voice_turn_complete' ? 0.8 : undefined));
    const a = (await loadTurnModels(invoke as never))!, b = (await loadTurnModels(invoke as never))!;
    expect(await a.vad.process(new Float32Array(512))).toEqual([0.9]);
    await b.vad.process(new Float32Array(512));
    const streams = invoke.mock.calls.filter(([c]) => c === 'voice_vad').map(([, args]) => (args as unknown as { stream: string }).stream);
    expect(new Set(streams).size).toBe(2);
    expect(await a.turn.predict(new Float32Array(1600))).toBe(0.8);
    expect(invoke.mock.calls.filter(([c]) => c === 'voice_turn_ready')).toHaveLength(1);
  });
});
