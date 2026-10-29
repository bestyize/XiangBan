import { describe, expect, it, vi } from 'vitest';
import { createVoiceOwner } from './owner.ts';

/** Two windows on one bus: an event reaches every listener, the sender included, as in Tauri. */
function windows() {
  const listeners: ((payload: unknown) => void)[] = [];
  const bus = {
    emit: vi.fn(async (_event: string, payload: unknown) => { listeners.slice().forEach(listener => listener(payload)); }),
    listen: async <T,>(_event: string, callback: (payload: T) => void) => { const l = callback as (payload: unknown) => void; listeners.push(l); return () => { listeners.splice(listeners.indexOf(l), 1); }; },
  };
  const lost = { control: vi.fn(), assistant: vi.fn() };
  const control = createVoiceOwner({ me: 'control', ...bus, onLost: lost.control });
  const assistant = createVoiceOwner({ me: 'assistant', ...bus, onLost: lost.assistant });
  return { control, assistant, lost, bus };
}

describe('voice owner', () => {
  it('stops the window that had the voice when another takes it, and not the one that took it', async () => {
    const { control, assistant, lost } = windows();
    await Promise.resolve();
    control.claim(); await Promise.resolve();
    expect(lost.control).not.toHaveBeenCalled(); expect(lost.assistant).not.toHaveBeenCalled(); // nobody had it before
    assistant.claim(); await Promise.resolve();
    expect(lost.control).toHaveBeenCalledOnce(); expect(lost.assistant).not.toHaveBeenCalled();
    control.claim(); await Promise.resolve();
    expect(lost.assistant).toHaveBeenCalledOnce(); expect(lost.control).toHaveBeenCalledOnce();
  });
  it('tells the others once, not at every sentence', async () => {
    const { control, bus } = windows();
    await Promise.resolve();
    control.claim(); control.claim(); control.claim();
    expect(bus.emit).toHaveBeenCalledOnce();
  });
  it('claims again after losing the voice', async () => {
    const { control, assistant, bus } = windows();
    await Promise.resolve();
    control.claim(); assistant.claim(); await Promise.resolve();
    control.claim(); await Promise.resolve();
    expect(bus.emit).toHaveBeenCalledTimes(3);
  });
  it('works alone when the other windows cannot be told, and ignores a window that was disposed', async () => {
    const lost = vi.fn();
    const alone = createVoiceOwner({ me: 'control', emit: () => { throw new Error('no bridge'); }, listen: async () => () => {}, onLost: lost });
    expect(() => alone.claim()).not.toThrow();
    const { control, assistant, lost: calls } = windows();
    await Promise.resolve();
    control.claim(); control.dispose(); assistant.claim(); await Promise.resolve();
    expect(calls.control).not.toHaveBeenCalled();
  });
});
