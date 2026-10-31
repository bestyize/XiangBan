<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { AdminApiError } from '@/api/admin'
import { toolMirrorsApi, type MirrorList, type MirrorTool, type MirrorArtifact } from '@/api/toolMirrors'
import { errorText, fmtTime, notify } from './shared'

const result = ref<MirrorList | null>(null)
const loading = ref(false)
const error = ref('')
const busy = ref('')
const actionError = ref('')
let sequence = 0
let disposed = false
const STATUS: Record<string, string> = { draft: '待审核', scheduled: '已审核，待定时发布', published: '已发布副本' }
const key = (t: MirrorTool, a: MirrorArtifact) => `${t.key}/${a.asset_id}`
const mib = (bytes: number) => `${(bytes / 1024 / 1024).toFixed(1)} MiB`
async function load() {
  const mine = ++sequence
  loading.value = true
  error.value = ''
  try {
    const data = await toolMirrorsApi.list()
    if (!disposed && mine === sequence) result.value = data
  } catch (e) {
    if (!disposed && mine === sequence && !(e instanceof AdminApiError && e.status === 401)) error.value = errorText(e)
  } finally { if (!disposed && mine === sequence) loading.value = false }
}
async function mirror(t: MirrorTool, a: MirrorArtifact) {
  if (busy.value || !result.value?.resource_configured) return
  busy.value = key(t, a)
  actionError.value = ''
  try {
    await toolMirrorsApi.create(t.key, a.asset_id, t.revision)
    if (disposed) return
    notify('镜像已写入并通过回读 SHA-256 校验')
    await load()
  } catch (e) {
    if (!disposed && !(e instanceof AdminApiError && e.status === 401)) {
      actionError.value = `${errorText(e)}。请刷新核对登记状态及草稿版本后重试。`
    }
  } finally { if (!disposed) busy.value = '' }
}
onMounted(() => { void load() })
onUnmounted(() => { disposed = true; ++sequence })
</script>

<template>
  <div class="adm-toolbar">
    <button class="btn" :disabled="loading || !!busy" @click="load">刷新镜像状态</button>
    <RouterLink class="btn" to="/admin/collection">工具来源与采集</RouterLink>
    <RouterLink class="btn" to="/admin/discover">发现页编辑与审核</RouterLink>
  </div>
  <p class="muted">手动从官方 GitHub 安装包创建镜像，写入后重新开文件回读并校验 SHA-256；不解压、不执行安装包。镜像操作不会发布草稿，待审内容仍需人工审核。</p>
  <p class="muted">只有已发布的同一安装包才能公开使用镜像，官方 GitHub 下载入口始终保留。镜像暂时不可用时，下载会回退到官方来源。</p>
  <div v-if="error" class="adm-card panel" role="alert"><p>{{ error }}</p><button class="btn" :disabled="loading" @click="load">重试</button></div>
  <p v-if="loading && !result" role="status">正在读取工具镜像…</p>
  <template v-if="result">
    <section class="panel adm-card">
      <h2>资源配额</h2>
      <p>单文件 {{ mib(result.max_asset_bytes) }}；总存储 {{ mib(result.max_storage_bytes) }}，最多 {{ result.max_storage_objects }} 个对象。资源机一次处理一个安装包，每次最多三分钟；相同哈希复用已验证文件。</p>
      <p v-if="result.invalid_items" role="alert">{{ result.invalid_items }} 条工具快照未通过格式或来源校验，已停止展示镜像操作，请到发现页检查。</p>
      <p v-if="!result.resource_configured" role="alert">尚未配置资源服务。请完成资源机部署，并配置论坛的资源服务地址及双方签名密钥后使用。</p>
      <p v-if="busy" role="status">正在下载、写入并回读校验，请等待结果。草稿在此期间被修改时，本次结果不会登记。</p>
      <p v-if="actionError" role="alert">{{ actionError }}</p>
    </section>
    <p v-if="!result.items.length" class="muted">暂无可镜像的官方工具安装包。请先配置工具仓库并采集正式版本。</p>
    <section v-for="tool in result.items" :key="tool.key" class="panel adm-card">
      <h2>{{ tool.name }} <small class="muted">{{ tool.version }}</small></h2>
      <p class="muted">{{ STATUS[tool.status] || tool.status }} · 草稿版本 {{ tool.revision }} · {{ tool.key }}</p>
      <div v-for="artifact in tool.artifacts" :key="artifact.asset_id" class="adm-card">
        <h3>{{ artifact.filename }}</h3>
        <div class="adm-form-grid">
          <p>{{ artifact.os }} / {{ artifact.arch }} · {{ mib(artifact.size) }}</p>
          <p>{{ artifact.verified_at ? `已回读校验 · ${fmtTime(artifact.verified_at)}` : '尚未创建镜像' }}</p>
        </div>
        <details><summary>查看 SHA-256</summary><code>{{ artifact.sha256 }}</code></details>
        <div class="adm-toolbar">
          <button class="btn btn-primary" :disabled="!result.resource_configured || !!busy || loading" @click="mirror(tool, artifact)">{{ busy === key(tool, artifact) ? '镜像校验中…' : artifact.verified_at ? '重新核验镜像' : '校验并创建镜像' }}</button>
          <a class="btn" :href="artifact.github_url" target="_blank" rel="noopener noreferrer">GitHub 官方下载</a>
        </div>
      </div>
    </section>
  </template>
</template>
