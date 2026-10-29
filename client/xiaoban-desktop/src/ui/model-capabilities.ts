import type { ProviderConfigView } from '../bridge.ts';
type Model = NonNullable<ProviderConfigView['models']>[number];
export type ModelFacts = Pick<Model, 'maxOutputTokens' | 'contextWindowTokens' | 'supportsTools' | 'supportsVision' | 'reasoningSummary' | 'thinkingMode'>;

/** Keep independent capability fields when editing, without mutating saved config. */
export function modelFacts(model: Partial<Model>): ModelFacts {
  return {
    maxOutputTokens: model.maxOutputTokens, contextWindowTokens: model.contextWindowTokens,
    supportsTools: model.supportsTools, supportsVision: model.supportsVision,
    reasoningSummary: model.reasoningSummary,
    thinkingMode: model.thinkingMode ? { ...model.thinkingMode } : model.thinkingMode,
  };
}
export function modelFactsPayload(facts: ModelFacts, endpointChanged: boolean) {
  return endpointChanged ? {} : Object.fromEntries(Object.entries(facts).filter(([, value]) => value != null));
}
