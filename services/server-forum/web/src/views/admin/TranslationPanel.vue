<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { translationApi, type TranslationLog, type TranslationSettings } from '@/api/translation'
import { errorText, fmtTime, notify } from './shared'

const settings = ref<TranslationSettings | null>(null)
const loading = ref(false)
const loadError = ref('')
const busy = ref<'' | 'save' | 'test'>('')
const actionError = ref('')
const result = ref<{ source: string; translated: string; millis: number } | null>(null)
const form = reactive({ base_url: '', model: '', key: '', translate_summary: true, translate_readme: true, readme_max_chars: 8000, max_new: 30 })
const configured = computed(() => !!settings.value?.configured)
const KIND: Record<TranslationLog['kind'], string> = { summary: '简介', readme: 'README', test: '测试' }

async function load() {
  loading.value = true; loadError.value = ''
  try {
    const next = await translationApi.get()
    settings.value = next
    form.base_url = next.base_url || form.base_url
    form.model = next.model || form.model
    Object.assign(form, next.options)
  } catch (e) { loadError.value = errorText(e) }
  finally { loading.value = false }
}
async function save() {
  if (busy.value) return
  actionError.value = ''; result.value = null; busy.value = 'save'
  try {
    const key = form.key.trim()
    await translationApi.save({
      base_url: form.base_url.trim(), model: form.model.trim(), ...(key ? { key } : {}),
      options: { translate_summary: form.translate_summary, translate_readme: form.translate_readme, readme_max_chars: Number(form.readme_max_chars), max_new: Number(form.max_new) },
    })
    form.key = ''
    notify('翻译服务已验证并保存')
    await load()
  } catch (e) { actionError.value = errorText(e) }
  finally { busy.value = '' }
}
async function test() {
  if (busy.value || !configured.value) return
  actionError.value = ''; result.value = null; busy.value = 'test'
  try { result.value = await translationApi.test(); await load() }
  catch (e) { actionError.value = errorText(e); await load() }
  finally { busy.value = '' }
}
const sec = (ms: number) => (ms >= 1000 ? `${(ms / 1000).toFixed(1)} 秒` : `${ms} 毫秒`)
onMounted(load)
</script>
<template>
  <section class="panel adm-card translation">
    <div class="collection-heading"><h2>翻译服务</h2><span class="adm-tag" :class="configured ? 'valid' : 'suspect'">{{ configured ? '密钥已配置' : '未配置密钥' }}</span></div>
    <p class="muted">把榜单项目的简介和 README 翻译成中文。采集时调用，译文随榜单保存，用户打开详情页不用等待。调用失败时详情页回退显示英文，榜单照常发布。</p>
    <div v-if="loadError" class="collection-error" role="alert">读取失败：{{ loadError }} <button class="btn btn-sm" :disabled="loading" @click="load">重试</button></div>
    <div class="translation-layout">
      <form class="adm-form" @submit.prevent="save">
        <label class="field"><span>接口地址</span><input v-model="form.base_url" class="input" spellcheck="false" autocomplete="off" required :disabled="!!busy" placeholder="https://ai.xiangban.xyz/v1" /><small class="muted">兼容 OpenAI 的 /chat/completions 接口，仅支持 https 公网域名。</small></label>
        <label class="field"><span>模型</span><input v-model="form.model" class="input" spellcheck="false" autocomplete="off" required :disabled="!!busy" placeholder="standard" /><small class="muted">填模型名，或网关里配置的档位名。</small></label>
        <label class="field"><span>{{ configured ? '更新 API key' : 'API key' }}</span><input v-model="form.key" class="input" type="password" autocomplete="off" spellcheck="false" :required="!configured" :disabled="!!busy" placeholder="只写入，不回显已保存密钥" /><small class="muted">{{ configured ? `已保存：${settings?.hint ?? ''} · 留空则沿用，输入新值才会更换。` : '验证成功后加密保存。' }}密钥不会保存到浏览器本地存储。</small></label>
        <div class="translation-actions"><button class="btn btn-primary" type="submit" :disabled="!!busy">{{ busy === 'save' ? '验证并保存中…' : configured ? '验证并更新' : '验证并保存' }}</button><button class="btn" type="button" :disabled="!!busy || !configured" @click="test">{{ busy === 'test' ? '测试中…' : '测试翻译' }}</button></div>
        <p v-if="actionError" class="collection-error" role="alert">{{ actionError }}</p>
        <div v-if="result" class="translation-result" role="status"><div><b>原文</b>{{ result.source }}</div><div><b>译文</b>{{ result.translated }}</div><small class="muted">耗时 {{ sec(result.millis) }}</small></div>
      </form>
      <aside class="translation-scope">
        <p class="adm-eyebrow">翻译范围</p>
        <label class="collection-check"><input v-model="form.translate_summary" type="checkbox" :disabled="!!busy" /> 项目简介</label>
        <label class="collection-check"><input v-model="form.translate_readme" type="checkbox" :disabled="!!busy" /> README 全文</label>
        <label class="field"><span>README 最多翻译字数</span><select v-model.number="form.readme_max_chars" class="input" :disabled="!!busy"><option :value="8000">8,000 字</option><option :value="16000">16,000 字</option><option :value="30000">30,000 字</option></select></label>
        <label class="field"><span>每次最多新翻译（个）</span><input v-model.number="form.max_new" class="input" type="number" min="1" max="30" step="1" :disabled="!!busy" /></label>
        <p class="muted">只翻译新出现或原文变化的内容，没变的直接沿用上次译文，不重复计费。超出字数上限的 README 会在段落处截断并在页面上注明。修改后点「验证并更新」生效。</p>
      </aside>
    </div>
    <h3>最近翻译</h3>
    <p v-if="!settings?.recent.length" class="muted">还没有翻译记录。</p>
    <table v-else class="translation-table"><thead><tr><th>时间</th><th>项目</th><th>内容</th><th>结果</th><th>耗时</th></tr></thead><tbody><tr v-for="(log, i) in settings.recent" :key="i"><td>{{ fmtTime(log.at) }}</td><td>{{ log.repo || '—' }}</td><td>{{ KIND[log.kind] ?? log.kind }}</td><td><span class="adm-tag" :class="log.ok ? 'valid' : 'suspect'">{{ log.ok ? '成功' : '失败' }}</span><small v-if="log.note" class="muted"> {{ log.note }}</small></td><td>{{ sec(log.millis) }}</td></tr></tbody></table>
  </section>
</template>
<style scoped>
.collection-heading { display:flex; align-items:center; flex-wrap:wrap; gap:12px; margin-bottom:12px; }
.collection-heading h2 { margin:0 auto 0 0; font-size:17px; }
.collection-check { display:flex; align-items:center; gap:8px; min-height:44px; }
.collection-error { color:var(--danger); overflow-wrap:anywhere; white-space:pre-wrap; }
.translation h3 { margin:24px 0 8px; font-size:15px; }
.translation-layout { display:grid; grid-template-columns:minmax(0,1fr) 300px; gap:24px; margin-top:12px; }
.translation-actions { display:flex; gap:10px; margin-top:16px; }
.translation-scope { display:grid; align-content:start; gap:12px; }
.translation-result { margin-top:14px; padding:12px 14px; border:1px solid var(--line); border-radius:12px; background:var(--surface-2); font-size:13px; line-height:1.7; }
.translation-result b { display:inline-block; width:40px; color:var(--ink-3); font-weight:600; }
.translation-table { width:100%; border-collapse:collapse; font-size:13px; }
.translation-table th { text-align:left; font-weight:600; color:var(--ink-3); padding:8px 10px; border-bottom:1px solid var(--line); }
.translation-table td { padding:10px; border-bottom:1px solid var(--line); vertical-align:top; overflow-wrap:anywhere; }
@media (max-width:900px) { .translation-layout { grid-template-columns:1fr; } }
</style>
