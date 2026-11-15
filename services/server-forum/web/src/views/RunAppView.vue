<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { contentApi, runtimeBase } from '@/api/content'
import AppRuntimeWindow from '@/components/creation/AppRuntimeWindow.vue'
import BuiltinGamePlayer from '@/components/creation/BuiltinGamePlayer.vue'
import CommunityGameTrial from '@/components/creation/CommunityGameTrial.vue'
import { webLink } from '@/components/community/contract'
import { useCommunityResource } from '@/components/community/resource'
import { canTrialPack } from '@/lib/appRuntime'
import { builtinGame, canOpenBundledWancisou, WANCISOU_ICON, WANCISOU_WEB_ENTRY } from '@/lib/builtinCreation'
import { useHead } from '@/lib/head'

const route = useRoute(), router = useRouter()
const isMbti = computed(() => route.name === 'mbti-run')
const mbti = builtinGame('mbti')!
const packId = computed(() => Number(route.params.id))
const version = computed(() => Number(route.params.version))
const launch = useCommunityResource(async () => {
  if (isMbti.value) return null
  const id = packId.value, requestedVersion = version.value
  if (!Number.isSafeInteger(id) || id < 1 || !Number.isSafeInteger(requestedVersion) || requestedVersion < 1) {
    throw new Error('作品地址无效')
  }
  const pack = await contentApi.pack(id)
  const manifest = pack.versions.find(item => item.version === requestedVersion)?.manifest
  if (pack.id !== id || pack.kind !== 'app' || manifest?.runtimeProtocol !== 2 ||
      !canTrialPack(pack.kind, pack.status, pack.trust_level, manifest.runtimeProtocol)) {
    throw new Error('这个版本的应用已不可用')
  }
  if (pack.slug === 'wancisou' && pack.network_disabled === false && pack.version === requestedVersion) {
    try {
      const response = await fetch('/builtin-games/wancisou/version.json', { signal: AbortSignal.timeout(5000) })
      if (response.ok && canOpenBundledWancisou(pack, (await response.json())?.version)) {
        return { pack, mode: 'bundled' as const }
      }
    } catch { /* The published version may still run on the configured app host. */ }
  }
  return { pack, mode: runtimeBase ? 'runtime' as const : 'client' as const }
})
const title = computed(() => isMbti.value ? mbti.name : launch.data.value?.pack.title || '小伴应用')
const icon = computed(() => isMbti.value ? mbti.icon :
  launch.data.value?.pack.slug === 'wancisou' ? WANCISOU_ICON : webLink(launch.data.value?.pack.cover_url))
useHead(() => ({ title: title.value }))

const player = ref<InstanceType<typeof CommunityGameTrial> | null>(null)
const revision = ref(0)
const reloading = ref(false), closing = ref(false)
const bundledReady = ref(false), bundledTimedOut = ref(false)
let bundledTimer: ReturnType<typeof setTimeout> | undefined
let disposed = false
watch([isMbti, packId, version], () => void launch.load(), { immediate: true })
watch(() => launch.data.value?.mode, mode => {
  clearTimeout(bundledTimer)
  bundledReady.value = false
  bundledTimedOut.value = false
  if (mode === 'bundled') bundledTimer = setTimeout(() => { bundledTimedOut.value = true }, 10000)
})
function bundledLoaded() {
  clearTimeout(bundledTimer)
  bundledTimedOut.value = false
  bundledReady.value = true
}
onBeforeUnmount(() => {
  disposed = true
  clearTimeout(bundledTimer)
})

async function reload() {
  if (closing.value || reloading.value) return
  const currentPath = route.fullPath
  reloading.value = true
  try {
    await player.value?.prepareExit()
    if (disposed || closing.value || route.fullPath !== currentPath) return
    revision.value++
    if (!isMbti.value) await launch.load()
  } finally {
    reloading.value = false
  }
}

function returnPage() {
  const previous = router.options.history.state.back
  if (typeof previous === 'string' && /^\/(?!\/)/.test(previous)) {
    const entry = router.resolve(previous)
    if ((entry.name === 'create' && !entry.query.tool && !entry.query.game) ||
        (entry.name === 'pack' && Number(entry.params.id) === packId.value)) return entry.fullPath
  }
  return launch.data.value ? `/packs/${packId.value}` : '/create'
}

async function close() {
  if (closing.value) return
  const currentPath = route.fullPath
  closing.value = true
  try {
    await player.value?.prepareExit()
    // Replace the host entry directly. A work may have added its own frame history.
    if (!disposed && route.fullPath === currentPath) await router.replace(returnPage())
  } finally {
    closing.value = false
  }
}
</script>

<template>
  <AppRuntimeWindow :title="title" :icon="icon" :compact="isMbti"
    :busy="reloading || launch.loading.value" :closing="closing" @reload="reload" @close="close">
    <BuiltinGamePlayer v-if="isMbti" :key="revision" :game-id="mbti.id" standalone @close="close" />
    <div v-else-if="launch.loading.value" class="state" role="status">正在加载应用…</div>
    <div v-else-if="launch.error.value" class="state" role="alert">
      <p>{{ launch.error.value }}</p>
      <button class="btn" type="button" @click="reload">重试</button>
    </div>
    <div v-else-if="launch.data.value?.mode === 'bundled'" class="bundled">
      <iframe :key="revision" :src="WANCISOU_WEB_ENTRY" :title="title" sandbox="allow-scripts allow-forms"
        allow="clipboard-write" referrerpolicy="no-referrer" @load="bundledLoaded" />
      <div v-if="!bundledReady" class="state bundled-status">
        <p v-if="bundledTimedOut" role="alert">应用未响应 <button class="btn" type="button" @click="reload">重试</button></p>
        <span v-else role="status">正在加载应用…</span>
      </div>
    </div>
    <CommunityGameTrial v-else-if="launch.data.value?.mode === 'runtime'" ref="player"
      :key="`${packId}:${version}:${revision}`" :pack="launch.data.value.pack" :version="version" standalone autostart
      @retry="reload" @exit="close" />
    <div v-else-if="launch.data.value" class="state">
      <p>请在小伴客户端打开这个应用。</p>
      <RouterLink class="btn" :to="`/packs/${packId}`">查看作品详情</RouterLink>
    </div>
  </AppRuntimeWindow>
</template>

<style scoped>
.bundled { position: relative; height: 100%; }
.bundled iframe { display: block; width: 100%; height: 100%; border: 0; background: var(--surface-2); }
.state { display: grid; align-content: center; justify-items: center; gap: 12px; height: 100%; padding: 20px; text-align: center; color: var(--ink-2); }
.state p { margin: 0; overflow-wrap: anywhere; }
.bundled-status { position: absolute; inset: 0; background: var(--surface); }
.bundled-status .btn { margin-inline-start: 8px; }
</style>
