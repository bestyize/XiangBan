import { describe, expect, it } from 'vitest';
import { ACK_PHRASES, chooseAck } from './ack.ts';

describe('chooseAck', () => {
  it('never repeats the phrase used last', () => {
    for (const last of ACK_PHRASES) for (const pick of [0, 0.5, 0.99]) expect(chooseAck(ACK_PHRASES, last, () => pick)).not.toBe(last);
  });
  it('uses the only phrase there is, and says nothing when there is none', () => {
    expect(chooseAck(['好的'], '好的')).toBe('好的');
    expect(chooseAck([], undefined)).toBeUndefined();
  });
});
