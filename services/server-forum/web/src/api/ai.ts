import { consumeAiStream, type AiChatProtocol } from '../lib/aiStream'
import type { AiMessage } from '../lib/aiHistory'
import { boundedImageResponse, generatedImageItems, imagePayload, UNKNOWN_IMAGE_RESULT, type ImageCapability, type ImageOptions } from '../lib/aiImages'

export interface AiTier { tier: string; display_name: string; description: string; is_default: boolean; available: boolean; requires_paid: boolean; chat_protocol?: AiChatProtocol }
export interface AiBalance { product: {product_id:string; name:string}; total: number; paid:number; quota_per_unit: number; buckets: {id:number; kind:string; total:number; remaining:number; expires_at:string|null; created_at:string}[]; trial: { available: boolean; quota:number; days:number }; credit_purchase_url: string }
export interface AiRedeemed { kind: string; quota: number; expires_at: string | null; balance: number }

export function aiBaseUrl(configured: string): string {
  const url = new URL(configured || 'https://ai.xiangban.xyz')
  const local = ['localhost', '127.0.0.1', '[::1]'].includes(url.hostname)
  if ((url.protocol !== 'https:' && !(url.protocol === 'http:' && local)) || url.username || url.password || url.search || url.hash) throw new Error('AI 网关地址配置无效')
  return url.toString().replace(/\/$/, '')
}
const base = () => aiBaseUrl(import.meta.env.VITE_AI_BASE_URL || '')
async function request(path: string, token: string, options: RequestInit = {}) {
  let response: Response
  try { response = await fetch(`${base()}${path}`, { ...options, headers: { Authorization: `Bearer ${token}`, ...(options.body ? { 'Content-Type': 'application/json' } : {}) }, redirect: 'error' }) }
  catch (error) { if (options.signal?.aborted) throw error; throw new Error('无法连接 AI 服务，请检查网络后重试。') }
  if (!response.ok) {
    const payload = await response.json().catch(() => null)
    const code = payload?.error?.code || payload?.code
    const messages: Record<string, string> = { insufficient_quota: 'AI 额度不足，请兑换额度后重试。', paid_required: '此模式需要付费额度。', code_invalid: '兑换码无效。', code_used: '兑换码已使用。', code_void: '兑换码已作废。', code_wrong_product: '兑换码不适用于当前产品。', too_many_attempts: '尝试次数过多，请稍后再试。' }
    Object.assign(messages, { model_not_found:'此模式已不可用，请刷新模式列表后重试。', context_length_exceeded:'对话超过模型上下文上限，请新建对话后重试。', request_too_large:'本次消息过长，请缩短内容或新建对话。', rate_limited:'请求过于频繁，请稍后重试。', forbidden:'当前账号尚未开通此产品的 AI，请联系站方。' })
    throw new Error(messages[code] || (response.status === 401 ? '登录已过期，请重新登录。' : `AI 服务暂不可用（${response.status}），请稍后重试。`))
  }
  return response
}
export const aiApi = {
  async capabilities(token: string, signal?:AbortSignal): Promise<{tiers:AiTier[]; media:Record<string, ImageCapability>}> {
    const value = await (await request('/api/ai/tiers', token, {signal})).json()
    if (!Array.isArray(value?.tiers) || !value.tiers.every((item:AiTier) => typeof item?.tier === 'string' && typeof item.display_name === 'string' && typeof item.available === 'boolean')) throw new Error('AI 模式数据异常，请重试。')
    return {tiers:value.tiers, media:value.media ?? {}}
  },
  async tiers(token: string, signal?:AbortSignal): Promise<AiTier[]> {
    return (await aiApi.capabilities(token, signal)).tiers
  },
  async images(token: string, options: ImageOptions, capability: ImageCapability, signal: AbortSignal): Promise<Blob[]> {
    const payload = imagePayload(options, capability)
    let response: Response
    try {
      response = await fetch(`${base()}/v1/images/generations`, {method:'POST', body:JSON.stringify(payload), headers:{Authorization:`Bearer ${token}`, 'Content-Type':'application/json'}, signal, redirect:'error'})
    } catch { throw Error(UNKNOWN_IMAGE_RESULT) }
    if (!response.ok) {
      const errors: Record<string,string> = {paid_required:'图片生成需要先兑换付费额度。', insufficient_quota:'小伴 AI 额度不足。', image_unavailable:'图片生成暂未开放。'}
      const body = JSON.parse(new TextDecoder().decode(await boundedImageResponse(response, 64 * 1024)))
      throw Error(response.status === 401 ? '登录已过期，请重新登录。' : errors[body?.code ?? body?.error?.code] || (response.status >= 500 ? UNKNOWN_IMAGE_RESULT : `生图请求被拒绝（${response.status}），请检查描述和参数。`))
    }
    try {
      const body = JSON.parse(new TextDecoder().decode(await boundedImageResponse(response)))
      const images = await generatedImageItems(body, options.n, signal)
      if (options.output_format === 'gif' && images.some(image => image.type !== 'image/gif')) throw Error('服务未返回 GIF')
      return images
    } catch { throw Error(UNKNOWN_IMAGE_RESULT) }
  },
  async balance(token: string, signal?:AbortSignal): Promise<AiBalance> {
    const value = await (await request('/api/ai/balance', token, {signal})).json()
    if (!Number.isFinite(value?.total) || typeof value?.trial?.available !== 'boolean' || !Array.isArray(value?.buckets)) throw new Error('AI 额度数据异常，请重试。')
    return value
  },
  async redeem(token: string, code: string): Promise<AiRedeemed> {
    return (await request('/api/ai/redeem', token, { method: 'POST', body: JSON.stringify({ code }) })).json()
  },
  async chat(token: string, tier: AiTier, messages: AiMessage[], signal: AbortSignal, onText: (text: string) => void): Promise<void> {
    const protocol = tier.chat_protocol ?? 'chat_completions'
    const responses = protocol === 'responses'
    const payload = responses
      ? { model: tier.tier, input: messages, stream: true, store: false }
      : { model: tier.tier, messages, stream: true }
    const response = await request(responses ? '/v1/responses' : '/v1/chat/completions', token, {
      method: 'POST', body: JSON.stringify(payload), signal,
    })
    if (!response.body) throw new Error('浏览器无法读取流式回复。')
    await consumeAiStream(response.body, onText, signal, protocol)
  },
}
