import type { AiProtocol } from '../bridge.ts';

export interface ProtocolOption { value: AiProtocol; label: string; placeholder: string; hint: string }

export const aiProtocols: ProtocolOption[] = [
  { value: 'chat', label: 'OpenAI 兼容（Chat Completions）', placeholder: 'https://api.openai.com/v1', hint: '多数服务商和本地模型服务都支持这种，不确定就选它。' },
  { value: 'responses', label: 'OpenAI Responses', placeholder: 'https://api.openai.com/v1', hint: 'OpenAI 较新的接口，只在服务商明确支持时选。' },
  { value: 'anthropic', label: 'Anthropic Claude（Messages）', placeholder: 'https://api.anthropic.com', hint: '地址不用带 /v1，带了也可以。' },
];

export const protocolOption = (protocol: AiProtocol) => aiProtocols.find(option => option.value === protocol) ?? aiProtocols[0];

/** Trimmed, without trailing slashes: the form the host stores. */
export const normalizeBaseUrl = (value: string) => value.trim().replace(/\/+$/, '');

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
