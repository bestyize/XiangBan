import { useEffect, useState } from 'react'
import { adminRequest as request } from './api.ts'
import type { LlmProviderConfig, LlmSettings } from './types.ts'
import { Alerts, Button as PrimaryButton, Field, input, panel, TextButton, useAdminAction } from './ui.tsx'

interface Preset {
  id: string
  name: string
  baseUrl: string
  model: string
  hint: string
}

const PRESETS: Preset[] = [
  { id: 'deepseek', name: 'DeepSeek (深度求索)', baseUrl: 'https://api.deepseek.com/v1', model: 'deepseek-chat', hint: '国内性价比之选，支持自然对话与思考' },
  { id: 'qwen', name: '阿里云通义千问 (Qwen)', baseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1', model: 'qwen-plus', hint: '阿里云兼容接口，生态稳定' },
  { id: 'siliconflow', name: '硅基流动 (SiliconFlow)', baseUrl: 'https://api.siliconflow.cn/v1', model: 'deepseek-ai/DeepSeek-V3', hint: '支持多款开源主流模型，开箱即用' },
  { id: 'moonshot', name: '月之暗面 (Kimi)', baseUrl: 'https://api.moonshot.cn/v1', model: 'moonshot-v1-8k', hint: '长文本与陪伴对谈表现优秀' },
  { id: 'zhipu', name: '智谱 AI (GLM)', baseUrl: 'https://open.bigmodel.cn/api/paas/v4', model: 'glm-4-flash', hint: '免费额度丰富，轻量快速' },
  { id: 'openai', name: 'OpenAI 官方', baseUrl: 'https://api.openai.com/v1', model: 'gpt-4o-mini', hint: '官方标准接口或海外代理' },
  { id: 'custom', name: '自定义供应商', baseUrl: '', model: '', hint: '任何兼容 OpenAI /chat/completions 协议的接口服务' },
]

export default function AdminLlm() {
  const [settings, setSettings] = useState<LlmSettings | null>(null)
  const [editingProvider, setEditingProvider] = useState<LlmProviderConfig | null>(null)
  const [testResult, setTestResult] = useState<{ id: string; reply?: string; error?: string } | null>(null)
  const [testingId, setTestingId] = useState<string | null>(null)
  const [searchUrlInput, setSearchUrlInput] = useState('')
  const [testingSearch, setTestingSearch] = useState(false)
  const [searchTestFeedback, setSearchTestFeedback] = useState<{ count?: number; title?: string; error?: string } | null>(null)
  const { busy, error, message, setMessage, setError, action } = useAdminAction()

  useEffect(() => {
    const controller = new AbortController()
    request<LlmSettings>('/llm', 'GET', undefined, controller.signal)
      .then((s) => {
        setSettings(s)
        setSearchUrlInput(s.webSearchBaseUrl || '')
      })
      .catch((e: Error) => { if (e.name !== 'AbortError') setError(e.message) })
    return () => controller.abort()
  }, [])

  function addFromPreset(preset: Preset) {
    const newId = preset.id === 'custom' ? `custom-${Date.now().toString(36)}` : preset.id
    const existing = settings?.providers.find((p) => p.id === newId)
    if (existing) {
      setEditingProvider(existing)
      return
    }
    setEditingProvider({
      id: newId,
      name: preset.name.split(' ')[0],
      baseUrl: preset.baseUrl,
      apiKey: '',
      model: preset.model,
      apiKeyConfigured: false,
    })
  }

  function handleSaveProvider(e: React.FormEvent) {
    e.preventDefault()
    if (!settings || !editingProvider) return

    const exists = settings.providers.some((p) => p.id === editingProvider.id)
    const updatedProviders = exists
      ? settings.providers.map((p) => (p.id === editingProvider.id ? editingProvider : p))
      : [...settings.providers, editingProvider]

    const nextActiveId = settings.activeProviderId || editingProvider.id
    const nextSettings: LlmSettings = {
      ...settings,
      activeProviderId: nextActiveId,
      providers: updatedProviders,
    }

    void action('save', async () => {
      const saved = await request<LlmSettings>('/llm', 'PUT', nextSettings)
      setSettings(saved)
      setEditingProvider(null)
      setMessage(`供应商「${editingProvider.name}」已保存并更新配置`)
    })
  }

  function handleDeleteProvider(id: string) {
    if (!settings || !window.confirm('确定要删除这个大模型供应商吗？')) return
    const filtered = settings.providers.filter((p) => p.id !== id)
    const nextActive = settings.activeProviderId === id ? (filtered[0]?.id || '') : settings.activeProviderId
    const nextSettings: LlmSettings = {
      ...settings,
      activeProviderId: nextActive,
      providers: filtered,
    }

    void action('delete', async () => {
      const saved = await request<LlmSettings>('/llm', 'PUT', nextSettings)
      setSettings(saved)
      setMessage('已删除供应商配置')
    })
  }

  function handleSetActive(id: string) {
    if (!settings) return
    const nextSettings: LlmSettings = { ...settings, activeProviderId: id }
    void action('switch', async () => {
      const saved = await request<LlmSettings>('/llm', 'PUT', nextSettings)
      setSettings(saved)
      setMessage(`已切换主供应商为「${saved.providers.find((p) => p.id === id)?.name || id}」`)
    })
  }

  function handleToggleEnabled(enabled: boolean) {
    if (!settings) return
    const nextSettings: LlmSettings = { ...settings, enabled }
    void action('toggle', async () => {
      const saved = await request<LlmSettings>('/llm', 'PUT', nextSettings)
      setSettings(saved)
      setMessage(enabled ? '大模型服务已开启' : '大模型服务已停用')
    })
  }

  async function handleTest(provider: LlmProviderConfig) {
    setTestingId(provider.id)
    setTestResult(null)
    try {
      const res = await request<{ reply: string }>('/llm/test', 'POST', {
        id: provider.id,
        baseUrl: provider.baseUrl,
        apiKey: provider.apiKey,
        model: provider.model,
      })
      setTestResult({ id: provider.id, reply: res.reply })
    } catch (e) {
      setTestResult({ id: provider.id, error: (e as Error).message })
    } finally {
      setTestingId(null)
    }
  }

  function handleSaveSearchUrl(e: React.FormEvent) {
    e.preventDefault()
    if (!settings) return
    const nextSettings: LlmSettings = {
      ...settings,
      webSearchBaseUrl: searchUrlInput.trim(),
    }
    void action('saveSearch', async () => {
      const saved = await request<LlmSettings>('/llm', 'PUT', nextSettings)
      setSettings(saved)
      setMessage('已保存 SearXNG 网络搜索服务配置')
    })
  }

  async function handleTestSearch() {
    if (!searchUrlInput.trim()) return
    setTestingSearch(true)
    setSearchTestFeedback(null)
    try {
      const res = await request<{ success: boolean; count: number; sampleTitle: string }>('/llm/test-search', 'POST', {
        baseUrl: searchUrlInput.trim(),
      })
      setSearchTestFeedback({ count: res.count, title: res.sampleTitle })
    } catch (e) {
      setSearchTestFeedback({ error: (e as Error).message })
    } finally {
      setTestingSearch(false)
    }
  }

  return (
    <>
      <Alerts error={error} message={message} />
      {settings ? (
        <div className="space-y-6">
          {/* Main Switch Panel */}
          <section className={panel}>
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div>
                <h2 className="text-lg font-semibold text-slate-900">大模型供应商与接口配置</h2>
                <p className="mt-1 text-sm text-slate-500">
                  兼容 OpenAI /chat/completions 协议标准。配置后，下班寄语 AI 润色与小伴流式伴聊将直接调用该接口。
                </p>
              </div>
              <label className="flex cursor-pointer items-center gap-2.5 rounded-full border border-slate-200 bg-slate-50 px-4 py-2">
                <input
                  type="checkbox"
                  className="size-4 accent-emerald-500"
                  checked={settings.enabled}
                  disabled={!!busy}
                  onChange={(e) => handleToggleEnabled(e.target.checked)}
                />
                <span className="text-sm font-medium text-slate-700">
                  {settings.enabled ? '服务已启用' : '服务已暂停'}
                </span>
              </label>
            </div>
          </section>

          {/* Preset Buttons for Quick Adding */}
          <section className={panel}>
            <h3 className="text-sm font-semibold text-slate-700">快捷添加主流供应商</h3>
            <div className="mt-3 flex flex-wrap gap-2.5">
              {PRESETS.map((p) => (
                <button
                  key={p.id}
                  type="button"
                  onClick={() => addFromPreset(p)}
                  className="rounded-xl border border-slate-200 bg-slate-50 px-3.5 py-2 text-xs font-medium text-slate-700 transition hover:border-slate-300 hover:bg-slate-100 active:scale-95"
                  title={p.hint}
                >
                  + {p.name}
                </button>
              ))}
            </div>
          </section>

          {/* Provider Card List */}
          <section className={panel}>
            <div className="flex items-center justify-between border-b border-slate-100 pb-4">
              <h3 className="text-base font-semibold text-slate-900">已配置供应商列表</h3>
              <span className="text-xs text-slate-400">单选勾选表示当前生效的主供应商</span>
            </div>

            {settings.providers.length === 0 ? (
              <div className="py-12 text-center text-sm text-slate-400">
                暂未添加任何大模型供应商，点击上方快捷按钮即可快速添加。
              </div>
            ) : (
              <div className="mt-4 space-y-3.5">
                {settings.providers.map((p) => {
                  const isActive = settings.activeProviderId === p.id
                  const isTesting = testingId === p.id
                  const test = testResult?.id === p.id ? testResult : null

                  return (
                    <div
                      key={p.id}
                      className={`relative rounded-xl border p-4 transition ${
                        isActive
                          ? 'border-emerald-400 bg-emerald-50/20 shadow-xs'
                          : 'border-slate-200 bg-white hover:border-slate-300'
                      }`}
                    >
                      <div className="flex flex-wrap items-start justify-between gap-3">
                        <label className="flex cursor-pointer items-center gap-3">
                          <input
                            type="radio"
                            name="activeProvider"
                            checked={isActive}
                            onChange={() => handleSetActive(p.id)}
                            className="size-4 accent-emerald-500"
                          />
                          <div>
                            <span className="font-bold text-slate-900">{p.name}</span>
                            {isActive && (
                              <span className="ml-2 rounded-full bg-emerald-100 px-2 py-0.5 text-[11px] font-medium text-emerald-800">
                                当前使用
                              </span>
                            )}
                            <span className="ml-2 rounded-md bg-slate-100 px-2 py-0.5 text-xs text-slate-600">
                              {p.model}
                            </span>
                          </div>
                        </label>

                        <div className="flex items-center gap-1">
                          <TextButton
                            disabled={isTesting || !!busy}
                            onClick={() => void handleTest(p)}
                            className="text-xs text-slate-600 hover:text-slate-900"
                          >
                            {isTesting ? '测试中…' : '测试连通性'}
                          </TextButton>
                          <TextButton
                            onClick={() => setEditingProvider(p)}
                            className="text-xs text-slate-600 hover:text-slate-900"
                          >
                            编辑
                          </TextButton>
                          <TextButton
                            onClick={() => handleDeleteProvider(p.id)}
                            className="text-xs text-rose-600 hover:text-rose-700"
                          >
                            删除
                          </TextButton>
                        </div>
                      </div>

                      <div className="mt-2 text-xs text-slate-500">
                        <span>接口地址：{p.baseUrl}</span>
                        <span className="ml-4">
                          Key 状态：
                          {p.apiKeyConfigured ? (
                            <span className="text-emerald-600">已配置</span>
                          ) : (
                            <span className="text-amber-600">未设置</span>
                          )}
                        </span>
                      </div>

                      {/* Test feedback */}
                      {test && (
                        <div
                          className={`mt-3 rounded-lg p-2.5 text-xs leading-relaxed ${
                            test.error ? 'bg-rose-50 text-rose-700' : 'bg-emerald-50 text-emerald-800'
                          }`}
                        >
                          {test.error ? `连通测试失败：${test.error}` : `测试成功！模型回复：「${test.reply}」`}
                        </div>
                      )}
                    </div>
                  )
                })}
              </div>
            )}
          </section>

          {/* Real-time Web Search (SearXNG) Panel */}
          <section className={panel}>
            <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-100 pb-4">
              <div>
                <h3 className="text-base font-semibold text-slate-900">实时网络搜索服务 (Web Search / SearXNG)</h3>
                <p className="mt-1 text-xs text-slate-500">
                  基于 SearXNG 开源元搜索引擎。开启后，小伴在被问到最新时事、新闻热点、电影或当下生活资讯时将自动触发搜索。
                </p>
              </div>
              <label className="flex cursor-pointer items-center gap-2 rounded-full border border-slate-200 bg-slate-50 px-3.5 py-1.5">
                <input
                  type="checkbox"
                  className="size-4 accent-emerald-500"
                  checked={settings.webSearchEnabled}
                  disabled={!!busy}
                  onChange={(e) => {
                    const next = { ...settings, webSearchEnabled: e.target.checked }
                    void action('toggleSearch', async () => {
                      const saved = await request<LlmSettings>('/llm', 'PUT', next)
                      setSettings(saved)
                      setMessage(e.target.checked ? '实时联网搜索已开启' : '实时联网搜索已停用')
                    })
                  }}
                />
                <span className="text-xs font-medium text-slate-700">
                  {settings.webSearchEnabled ? '已开启搜索' : '已停用搜索'}
                </span>
              </label>
            </div>

            <form className="mt-5 space-y-4" onSubmit={handleSaveSearchUrl}>
              <Field
                title="SearXNG 服务地址 (Base URL)"
                hint="SearXNG 容器访问根地址。如你部署在香港服务器上，填入 http://43.155.99.113:50001"
              >
                <div className="flex gap-2">
                  <input
                    className={input}
                    value={searchUrlInput}
                    onChange={(e) => setSearchUrlInput(e.target.value)}
                    placeholder="http://43.155.99.113:50001"
                  />
                  <button
                    type="button"
                    onClick={() => setSearchUrlInput('http://43.155.99.113:50001')}
                    className="shrink-0 rounded-xl border border-slate-200 bg-slate-50 px-3 text-xs text-slate-600 transition hover:bg-slate-100 active:scale-95"
                  >
                    填入香港服务器
                  </button>
                </div>
              </Field>

              <div className="flex flex-wrap items-center justify-between gap-3 pt-1">
                <TextButton
                  type="button"
                  disabled={testingSearch || !searchUrlInput.trim()}
                  onClick={() => void handleTestSearch()}
                  className="border border-slate-200 text-xs text-slate-700 hover:bg-slate-50"
                >
                  {testingSearch ? '测试中…' : '测试搜索连通性'}
                </TextButton>

                <PrimaryButton type="submit" disabled={busy === 'saveSearch'}>
                  {busy === 'saveSearch' ? '正在保存…' : '保存搜索地址'}
                </PrimaryButton>
              </div>

              {searchTestFeedback && (
                <div
                  className={`rounded-lg p-3 text-xs leading-relaxed ${
                    searchTestFeedback.error ? 'bg-rose-50 text-rose-700' : 'bg-emerald-50 text-emerald-800'
                  }`}
                >
                  {searchTestFeedback.error
                    ? `测试失败: ${searchTestFeedback.error}`
                    : `测试成功！SearXNG 响应正常，返回 ${searchTestFeedback.count} 条结果，第一条：【${searchTestFeedback.title}】`}
                </div>
              )}
            </form>
          </section>

          {/* Edit / Add Modal or Inline Form */}
          {editingProvider && (
            <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/40 p-4">
              <div className="w-full max-w-lg rounded-2xl bg-white p-6 shadow-xl">
                <div className="flex items-center justify-between border-b border-slate-100 pb-3">
                  <h3 className="font-bold text-slate-900">
                    {settings.providers.some((p) => p.id === editingProvider.id) ? '编辑供应商' : '添加供应商'}
                  </h3>
                  <button
                    type="button"
                    onClick={() => setEditingProvider(null)}
                    className="text-slate-400 hover:text-slate-600"
                  >
                    ✕
                  </button>
                </div>

                <form className="mt-4 space-y-4" onSubmit={handleSaveProvider}>
                  <Field title="供应商名称" hint="如 DeepSeek V3、Qwen Plus 或自建本地模型">
                    <input
                      className={input}
                      required
                      value={editingProvider.name}
                      onChange={(e) => setEditingProvider({ ...editingProvider, name: e.target.value })}
                      placeholder="DeepSeek"
                    />
                  </Field>

                  <Field title="API 接口地址 (Base URL)" hint="必须是以 https:// 或 http:// 开头的 OpenAI 协议地址">
                    <input
                      className={input}
                      required
                      value={editingProvider.baseUrl}
                      onChange={(e) => setEditingProvider({ ...editingProvider, baseUrl: e.target.value })}
                      placeholder="https://api.deepseek.com/v1"
                    />
                  </Field>

                  <Field title="模型名称 (Model)" hint="接口指定的模型参数，如 deepseek-chat 或 gpt-4o-mini">
                    <input
                      className={input}
                      required
                      value={editingProvider.model}
                      onChange={(e) => setEditingProvider({ ...editingProvider, model: e.target.value })}
                      placeholder="deepseek-chat"
                    />
                  </Field>

                  <Field
                    title="API Key 访问密钥"
                    hint={
                      editingProvider.apiKeyConfigured
                        ? '已有保存的 Key，留空保持原值。如需更换，输入新 Key 即可。'
                        : '填写服务商提供的 API Key。'
                    }
                  >
                    <input
                      className={input}
                      type="password"
                      autoComplete="new-password"
                      required={!editingProvider.apiKeyConfigured}
                      value={editingProvider.apiKey}
                      onChange={(e) => setEditingProvider({ ...editingProvider, apiKey: e.target.value })}
                      placeholder={editingProvider.apiKeyConfigured ? '••••••••••••••••' : 'sk-...'}
                    />
                  </Field>

                  <div className="mt-6 flex justify-end gap-3 pt-3">
                    <TextButton type="button" onClick={() => setEditingProvider(null)}>
                      取消
                    </TextButton>
                    <PrimaryButton type="submit" disabled={!!busy}>
                      {busy === 'save' ? '正在保存…' : '保存供应商'}
                    </PrimaryButton>
                  </div>
                </form>
              </div>
            </div>
          )}
        </div>
      ) : (
        <p className="py-8 text-slate-500">正在加载大模型配置…</p>
      )}
    </>
  )
}
