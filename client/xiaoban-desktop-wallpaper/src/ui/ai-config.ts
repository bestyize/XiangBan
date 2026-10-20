import type { AiProtocol, ProviderConfigView } from '../bridge.ts';

export interface ProviderMeta {
  type?: "third" | "free" | "paid";
  models?: { id: string; name: string; description?: string }[];
  id: string;
  name: string;
  desc: string;
  /** The protocol a fresh setup starts on. */
  protocol: AiProtocol;
  /** The base address for the protocol the form is currently on, if this provider serves it. */
  endpoints: Partial<Record<AiProtocol, string>>;
  defaultModel: string;
  suggestedModels: string[];
  consoleUrl?: string;
  consoleText?: string;
}

/**
 * The built-in providers, with one base address per protocol. The four vendors serve the same
 * model behind all three wire formats but each on its own path, so switching protocol has to
 * change the address too:
 *
 *   DeepSeek   chat https://api.deepseek.com            responses https://api.deepseek.com
 *              anthropic https://api.deepseek.com/anthropic
 *   智谱        chat https://open.bigmodel.cn/api/paas/v4
 *              responses https://open.bigmodel.cn/api/v1
 *              anthropic https://open.bigmodel.cn/api/anthropic
 *   Kimi       chat https://api.moonshot.cn/v1          responses https://api.moonshot.cn/v1
 *              anthropic https://api.moonshot.cn/anthropic
 *   MiniMax    chat https://api.minimax.cn/v1           responses https://api.minimax.cn/v1
 *              anthropic https://api.minimax.cn/anthropic
 */
export const BUILTIN_PROVIDERS: ProviderMeta[] = [
  {
    id: 'deepseek',
    name: 'DeepSeek',
    desc: '深度求索最新一代大模型，默认配置 deepseek-v4-pro 旗舰大模型',
    protocol: 'responses',
    endpoints: {
      chat: 'https://api.deepseek.com',
      responses: 'https://api.deepseek.com',
      anthropic: 'https://api.deepseek.com/anthropic',
    },
    defaultModel: 'deepseek-v4-pro',
    suggestedModels: ['deepseek-v4-pro', 'deepseek-flash', 'deepseek-chat', 'deepseek-reasoner'],
    consoleUrl: 'https://platform.deepseek.com/api_keys',
    consoleText: '前往 DeepSeek 开放平台获取 API Key',
  },
  {
    id: 'zhipu',
    name: '智谱 BigModel',
    desc: '清华系自研基座大模型，默认升级为官方最新 GLM-5.3 旗舰大模型',
    protocol: 'responses',
    endpoints: {
      chat: 'https://open.bigmodel.cn/api/paas/v4',
      responses: 'https://open.bigmodel.cn/api/v1',
      anthropic: 'https://open.bigmodel.cn/api/anthropic',
    },
    defaultModel: 'glm-5.3',
    suggestedModels: ['glm-5.3', 'glm-5.3-flash', 'glm-5.2', 'glm-4-plus', 'glm-4-flash'],
    consoleUrl: 'https://open.bigmodel.cn/usercenter/apikeys',
    consoleText: '前往智谱开放平台获取 API Key',
  },
  {
    id: 'kimi',
    name: 'Kimi',
    desc: '月之暗面长文本大模型，默认配置官方最新 1M 上下文 Kimi K3 旗舰大模型',
    protocol: 'responses',
    endpoints: {
      chat: 'https://api.moonshot.cn/v1',
      responses: 'https://api.moonshot.cn/v1',
      anthropic: 'https://api.moonshot.cn/anthropic',
    },
    defaultModel: 'kimi-k3',
    suggestedModels: ['kimi-k3', 'kimi-k2.7-code', 'kimi-k2.7-code-highspeed', 'kimi-k2.6', 'moonshot-v1-auto'],
    consoleUrl: 'https://platform.kimi.com/console/api-keys',
    consoleText: '前往 Kimi 开放平台获取 API Key',
  },
  {
    id: 'minimax',
    name: 'MiniMax',
    desc: 'MiniMax 开放平台大模型，默认采用官方最新万亿参数 MiniMax-M3 旗舰基座',
    protocol: 'responses',
    endpoints: {
      chat: 'https://api.minimax.cn/v1',
      responses: 'https://api.minimax.cn/v1',
      anthropic: 'https://api.minimax.cn/anthropic',
    },
    defaultModel: 'MiniMax-M3',
    suggestedModels: ['MiniMax-M3', 'MiniMax-M2.7', 'MiniMax-M2.7-highspeed', 'MiniMax-M2'],
    consoleUrl: 'https://platform.minimaxi.com/user-center/basic-information/interface-key',
    consoleText: '前往 MiniMax 开放平台获取 API Key',
  },
  {
    id: 'custom',
    name: '自定义供应商',
    desc: '自定义第三方端点、海外服务或本地部署的 Ollama / vLLM 模型服务',
    protocol: 'responses',
    endpoints: {},
    defaultModel: '',
    suggestedModels: [],
  },
];

export function getProviderMeta(id: string): ProviderMeta {
  return BUILTIN_PROVIDERS.find(p => p.id === id) ?? BUILTIN_PROVIDERS[BUILTIN_PROVIDERS.length - 1];
}

/** The address this provider uses for a protocol; empty when it does not serve that one. */
export function providerEndpoint(id: string, protocol: AiProtocol): string {
  return getProviderMeta(id).endpoints[protocol] ?? '';
}

export interface ProtocolOption { value: AiProtocol; label: string; placeholder: string; hint: string }

/** In the order the picker shows them: newest first, with the broadly compatible one last. */
export const aiProtocols: ProtocolOption[] = [
  { value: 'responses', label: 'OpenAI Responses', placeholder: 'https://api.openai.com/v1', hint: 'OpenAI 较新的接口，只在服务商明确支持时选。' },
  { value: 'anthropic', label: 'Anthropic Claude（Messages）', placeholder: 'https://api.anthropic.com', hint: '地址不用带 /v1，带了也可以。' },
  { value: 'chat', label: 'OpenAI 兼容（Chat Completions）', placeholder: 'https://api.openai.com/v1', hint: '多数服务商和本地模型服务都支持这种，不确定就选它。' },
];

export const protocolOption = (protocol: AiProtocol) =>
  aiProtocols.find(option => option.value === protocol) ?? aiProtocols.find(option => option.value === 'responses')!;

/** Trimmed, without trailing slashes: the form the host stores. */
export const normalizeBaseUrl = (value: string) => value.trim().replace(/\/+$/, '');

/** Explicitly verified endpoint/model pairs; keep aligned with ai_reasoning::supported. */
export function supportsReplyPreference(protocol: AiProtocol, baseUrl: string, model: string): boolean {
  if (protocol !== 'responses') return false;
  const base = normalizeBaseUrl(baseUrl);
  const name = model.trim();
  if (base === 'https://api.openai.com/v1') return ['gpt-6-astra', 'gpt-6.1-sol', 'gpt-6-sol', 'gpt-6-luna'].includes(name);
  if (['https://api.deepseek.com', 'https://api.deepseek.com/v1'].includes(base)) return ['deepseek-flash', 'deepseek-v4-pro'].includes(name);
  return false;
}

/** The address a request will go to, spelled the same way the host builds it. */
export function endpointPreview(protocol: AiProtocol, baseUrl: string): string {
  const base = normalizeBaseUrl(baseUrl);
  if (!base) return '';
  if (protocol === 'responses') return `${base}/responses`;
  if (protocol === 'anthropic') return base.endsWith('/v1') ? `${base}/messages` : `${base}/v1/messages`;
  return `${base}/chat/completions`;
}

/** A first look for the form; the host checks again and has the last word. */
export function validateBaseUrl(value: string): string {
  const text = value.trim();
  if (!text) return '请填写 API 地址';
  let url: URL;
  try { url = new URL(text); } catch { return 'API 地址格式不对，应类似 https://api.example.com/v1'; }
  if (url.protocol !== 'http:' && url.protocol !== 'https:') return 'API 地址必须以 http:// 或 https:// 开头';
  if (url.username || url.password) return 'API 地址里不要带用户名或密码，密钥请填在密钥栏';
  if (url.search || url.hash) return 'API 地址里不要带 ? 或 # 参数，只填到接口根路径';
  return '';
}

export function validateAiForm(form: { baseUrl: string; model: string }): string {
  return validateBaseUrl(form.baseUrl) || (form.model.trim() ? '' : '请填写模型名');
}

/** Empty means "keep the saved key", so only a typed key is checked. */
export function validateKey(value: string): string {
  const key = value.trim();
  return key && /\s/.test(key) ? '密钥里不能有空格或换行，请重新复制粘贴' : '';
}

const loopback = (host: string) => host === 'localhost' || host.endsWith('.localhost') || host === '[::1]' || /^127(\.\d{1,3}){3}$/.test(host);

/** Plain http to anything but this computer sends the key and the chat in the clear. */
export function insecureRemote(baseUrl: string): boolean {
  try {
    const url = new URL(baseUrl.trim());
    return url.protocol === 'http:' && !loopback(url.hostname);
  } catch { return false; }
}


export function remoteProviderMeta(p: ProviderConfigView): ProviderMeta {
  const fallback = getProviderMeta(p.id);
  return {
    id: p.id, name: p.name, desc: p.description || fallback.desc, protocol: p.protocol,
    endpoints: Object.keys(p.endpoints ?? {}).length ? p.endpoints! : { [p.protocol]: p.baseUrl },
    defaultModel: p.defaultModel || p.model, suggestedModels: p.models?.map(m => m.id) ?? fallback.suggestedModels,
    consoleUrl: p.consoleUrl, consoleText: p.consoleText, type: p.type === 'custom' ? undefined : p.type,
    models: p.models,
  };
}
