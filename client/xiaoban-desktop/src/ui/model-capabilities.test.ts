import { describe, it, expect } from 'vitest';
import { modelFacts, modelFactsPayload } from './model-capabilities.ts';
describe('independent model capabilities', () => {
  it('round trips summary and budget mode without alias effort levels', () => {
    const saved = { reasoningSummary: true, thinkingMode: { type: 'enabled' as const, budget_tokens: 2048 }, maxOutputTokens: 8192 };
    const draft = modelFacts(saved);
    expect(modelFactsPayload(draft, false)).toEqual(saved);
    if (draft.thinkingMode?.type === 'enabled') draft.thinkingMode.budget_tokens = 4096;
    expect(saved.thinkingMode.budget_tokens).toBe(2048);
    expect(modelFactsPayload(draft, true)).toEqual({});
  });
  it('preserves explicit false and leaves legacy fields undeclared', () => {
    expect(modelFactsPayload(modelFacts({ reasoningSummary: false }), false)).toEqual({ reasoningSummary: false });
    expect(modelFactsPayload(modelFacts({}), false)).toEqual({});
  });
});
