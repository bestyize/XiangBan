export interface AiModel { reasoningEfforts?: string[] | null; id: string; name: string; description: string; enabled: boolean }
export interface AiAccess { uid: string; apikey?: string; hasKey?: boolean }
export type AiProviderType = 'third' | 'free' | 'paid'
export interface AiProvider {
  id: string; provider: string; description: string; endPoint: string; protocol: string;
  type: AiProviderType; enabled: boolean; defaultModel: string; models: AiModel[];
  apikey?: string; hasKey?: boolean; clearKey?: boolean;
  endpoints: Record<string, string>; consoleUrl: string; consoleText: string; userAccess: AiAccess[];
}
/** The built-in 小伴 AI entry of the desktop app. It is not in `providers`; this switch is all the console controls. */
export interface AiEntry { enabled: boolean }
export interface AiCatalogue { version: number; revision: number; refreshAfterSeconds: number; xiaobanAi?: AiEntry; providers: AiProvider[] }

export const AI_PROTOCOLS = ['OpenAI Responses', 'OpenAI Chat Completions', 'Anthropic Messages']

export const AI_TYPES: { value: AiProviderType; label: string; hint: string }[] = [
  { value: 'third', label: '第三方', hint: '用户自己填写 Key，小伴只提供地址和推荐模型。' },
  { value: 'free', label: '小伴免费', hint: '登录用户免费使用，凭证由这里填写（公共免密网关可留空）。' },
  { value: 'paid', label: '小伴付费', hint: '只有被授权的用户能看到，每人一个受限的网关令牌。' },
]

export const aiTypeLabel = (type: AiProviderType) => AI_TYPES.find(t => t.value === type)?.label ?? type

export const newAiProvider = (): AiProvider => ({
  id: '', provider: '', description: '', endPoint: '', protocol: AI_PROTOCOLS[0], type: 'third',
  enabled: false, defaultModel: '', models: [], endpoints: {}, consoleUrl: '', consoleText: '', userAccess: [],
})
