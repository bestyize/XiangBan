// 翻译服务设置：密钥只写不读，后端保存前先用示例文本验证。
import { request as adminRequest } from './admin'

export interface TranslationOptions { translate_summary: boolean; translate_readme: boolean; readme_max_chars: number; max_new: number }
export interface TranslationLog { at: string; repo: string; kind: 'summary' | 'readme' | 'test'; ok: boolean; note: string; millis: number }
export interface TranslationSettings {
  configured: boolean; base_url: string; model: string; hint: string | null
  options: TranslationOptions; updated_at?: string; recent: TranslationLog[]
}
export interface TranslationInput { base_url: string; model: string; key?: string; options?: Partial<TranslationOptions> }

export const translationApi = {
  get: () => adminRequest<TranslationSettings>('GET', '/translation'),
  save: (body: TranslationInput) => adminRequest<{ ok: true; hint: string | null }>('PUT', '/translation', body),
  test: () => adminRequest<{ source: string; translated: string; millis: number }>('POST', '/translation/test'),
}
