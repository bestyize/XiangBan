import { describe, expect, it } from 'vitest';
import { modelFailure } from './model-errors.ts';
import { applyEvent, newView } from './view.ts';
describe('model failure recovery', () => {
  it('does not treat a missing route or temporary limits as a retired model', () => {
    expect(modelFailure('HTTP 404: route not found')).toMatchObject({ refresh: false });
    expect(modelFailure('model_not_found')).toMatchObject({ refresh: true });
    expect(modelFailure('HTTP 429 rate_limit')).toMatchObject({ refresh: false });
    expect(modelFailure('insufficient_quota').message).toContain('额度');
    expect(modelFailure('invalid_api_key').message).toContain('认证');
  });
  it('offers original-message retry but stops offering it after output or a tool action', () => {
    for (const partial of [false, true]) {
      const view = newView('s');
      view.items.push({ kind: 'user', id: 'u', text: 'hello', images: ['data:image/png;base64,x'] });
      if (partial) applyEvent(view, { type: 'tool_start', id: 't', name: 'shell', arguments: '{}' });
      applyEvent(view, { type: 'error', message: 'model_not_found' });
      expect(view.items.at(-1)).toMatchObject({ canSwitchModel: true, canRetry: !partial });
      expect(view.items.filter(i => i.kind === 'user')).toHaveLength(1);
    }
  });
});
