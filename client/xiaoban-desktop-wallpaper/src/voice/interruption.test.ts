import { describe, expect, it } from 'vitest';
import { interruptionNote, withoutInterruptionNote } from './interruption.ts';

describe('interruption note', () => {
  it('tells the model what was heard and is removed again for display', () => {
    const note = interruptionNote('从前有座山，山上有座庙。');
    expect(note).toContain('「从前有座山，山上有座庙。」');
    expect(note.endsWith('\n')).toBe(true);
    expect(withoutInterruptionNote(`${note}换一个故事`)).toBe('换一个故事');
  });
  it('shortens a long reading to its start and its last words', () => {
    const note = interruptionNote('甲'.repeat(40) + '乙'.repeat(300) + '丙'.repeat(100));
    expect(note).toContain('甲'.repeat(40) + '……' + '丙'.repeat(100));
    expect(note).not.toContain('乙');
  });
  it('says so when nothing was heard or the position is unknown', () => {
    expect(interruptionNote('')).toContain('还没听到任何内容');
    expect(interruptionNote(null)).toContain('不详');
  });
  it('leaves ordinary messages alone', () => {
    expect(withoutInterruptionNote('帮我看看【语音打断】是什么')).toBe('帮我看看【语音打断】是什么');
  });
});
