import { GATEWAY_PROVIDER, type AiConfigView } from '../../bridge.ts';
import type { SessionModel } from './types.ts';

/** Overlay a conversation's choice without editing the shared provider registry. */
export function sessionModelConfig(config: AiConfigView | null, model: SessionModel | null): AiConfigView | null {
  if (!config || !model) return config;
  const provider = config.providers?.find(p => p.id === model.providerId);
  return {
    ...config, activeProvider: model.providerId, model: model.model,
    ...(model.providerId === GATEWAY_PROVIDER
      ? { gatewayTier: model.model, gatewayReplyPreference: model.replyPreference, gatewayReasoningEfforts: model.reasoningEfforts ?? [] }
      : { providers: config.providers?.map(p => p.id === model.providerId ? { ...p, model: model.model, replyPreference: model.replyPreference } : p),
          baseUrl: provider?.baseUrl ?? '', hasKey: provider?.hasKey ?? false }),
  };
}
