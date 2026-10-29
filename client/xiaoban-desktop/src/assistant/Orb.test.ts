// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import Orb from './Orb.vue';

describe('Orb', () => {
  it('shows the brand mark when idle or thinking and sound bars while listening or reading aloud', () => {
    for (const phase of ['idle', 'thinking', 'answering', 'attention'] as const) {
      const orb = mount(Orb, { props: { phase } });
      expect(orb.find('.mark').exists()).toBe(true); expect(orb.find('.bars').exists()).toBe(false);
    }
    for (const phase of ['listening', 'speaking'] as const) {
      const orb = mount(Orb, { props: { phase } });
      expect(orb.find('.bars').exists()).toBe(true); expect(orb.find('.mark').exists()).toBe(false);
    }
  });
  it('follows the microphone level while listening and greys out on error', () => {
    const orb = mount(Orb, { props: { phase: 'listening', level: 3, size: 44, muted: true } });
    const frame = orb.get('.orb-frame');
    expect(frame.attributes('style')).toContain('--loud: 1'); expect(frame.attributes('style')).toContain('width: 44px');
    expect(frame.classes()).toContain('muted');
  });
});
