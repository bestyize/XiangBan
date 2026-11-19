<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { onBeforeRouteLeave, onBeforeRouteUpdate, useRoute, useRouter } from 'vue-router'
import { contentApi, gameSandboxUrl } from '@/api/content'
import AppRuntimeStatus from '@/components/creation/AppRuntimeStatus.vue'
import AppRuntimeWindow from '@/components/creation/AppRuntimeWindow.vue'
import BuiltinGamePlayer from '@/components/creation/BuiltinGamePlayer.vue'
import CommunityGameTrial from '@/components/creation/CommunityGameTrial.vue'
import { webLink } from '@/components/community/contract'
import { useCommunityResource } from '@/components/community/resource'
import { canTrialPack } from '@/lib/appRuntime'
import { builtinAsset, builtinGame, canOpenBundledWancisou, WANCISOU_ICON, WANCISOU_WEB_ENTRY } from '@/lib/builtinCreation'
import { useHead } from '@/lib/head'
import { homeTabOf } from '@/lib/homeNavigation'
import { ideaPlayerId, ideaPlayers, ideaReturnLocation } from '@/lib/ideaPlayers'

const route = useRoute(), router = useRouter()
const isBuiltin = computed(() => route.name === 'builtin-run')
const isIdea = computed(() => route.name === 'idea-run')
const ideaId = computed(() => isIdea.value ? ideaPlayerId(route.params.slug) : undefined)
const builtin = computed(() => typeof route.params.gameId === 'string' ? builtinGame(route.params.gameId) : undefined)
const fullscreenOrientation = computed(() => {
  const game = builtin.value
  if (!game || game.id === 'mbti') return
  return game.orientation === 'landscape' ? 'landscape' : 'portrait'
})
const packId = computed(() => Number(route.params.id))
const version = computed(() => Number(route.params.version))
let launchController: AbortController | undefined
const launch = useCommunityResource(async () => {
  launchController?.abort()
  if (isBuiltin.value || isIdea.value) return null
  const id = packId.value, requestedVersion = version.value
  if (!Number.isSafeInteger(id) || id < 1 || !Number.isSafeInteger(requestedVersion) || requestedVersion < 1) {
    throw new Error('作品地址无效')
  }
  const controller = new AbortController()
  launchController = controller
  const pack = await contentApi.pack(id, false, controller.signal)
  const manifest = pack.versions.find(item => item.version === requestedVersion)?.manifest
  if (pack.id !== id || !manifest ||
      !canTrialPack(pack.kind, pack.status, pack.trust_level, manifest.runtimeProtocol)) {
    throw new Error('这个版本的作品已不可用')
  }
  if (pack.kind === 'app' && pack.slug === 'wancisou' && pack.network_disabled === false && pack.version === requestedVersion) {
    try {
      const response = await fetch(builtinAsset('wancisou/version.json'), { signal: controller.signal })
      if (response.ok && canOpenBundledWancisou(pack, (await response.json())?.version)) {
        return { pack, mode: 'bundled' as const }
      }
    } catch (error) {
      if (controller.signal.aborted) throw error
    }
  }
  const hasHost = manifest.runtimeProtocol === 2 || gameSandboxUrl
  return { pack, mode: hasHost ? 'runtime' as const : 'client' as const }
})
const title = computed(() => ideaId.value ? ideaPlayers[ideaId.value].title :
  builtin.value?.name || launch.data.value?.pack.title || '小伴应用')
const icon = computed(() => builtin.value?.icon ||
  (launch.data.value?.pack.slug === 'wancisou' ? WANCISOU_ICON : webLink(launch.data.value?.pack.cover_url)))
useHead(() => ({ title: title.value }))

const player = ref<InstanceType<typeof CommunityGameTrial> | null>(null)
const revision = ref(0)
const reloading = ref(false), closing = ref(false)
const bundledReady = ref(false)
let disposed = false
watch([isBuiltin, isIdea, packId, version], () => void launch.load(), { immediate: true })
watch(() => launch.data.value?.mode, () => { bundledReady.value = false })
onBeforeRouteLeave(() => player.value?.prepareExit())
onBeforeRouteUpdate((to, from) => {
  if (to.path !== from.path) return player.value?.prepareExit()
})
const removeNavigationHook = router.afterEach((_to, from, failure) => {
  if (failure && route.fullPath === from.fullPath) player.value?.resume()
})
onBeforeUnmount(() => {
  disposed = true
  launchController?.abort()
  removeNavigationHook()
})

async function reload() {
  if (closing.value || reloading.value) return
  const currentPath = route.fullPath
  reloading.value = true
  try {
    await player.value?.prepareExit()
    if (disposed || closing.value || route.fullPath !== currentPath) return
    revision.value++
    if (!isBuiltin.value && !isIdea.value) void launch.load()
  } finally {
    reloading.value = false
  }
}

function returnPage() {
  const previous = router.options.history.state.back
  if (typeof previous === 'string' && /^\/(?!\/)/.test(previous)) {
    const entry = router.resolve(previous)
    if (isIdea.value) {
      const ideas = entry.name === 'ideas' || (entry.name === 'home' && homeTabOf(entry.query.tab) === 'ideas')
      if (entry.name === 'idea-work' || (ideas && !entry.query.exhibit)) return entry.fullPath
    } else if ((entry.name === 'create' && !entry.query.tool && !entry.query.game) ||
        (entry.name === 'pack' && Number(entry.params.id) === packId.value)) return entry.fullPath
  }
  if (isIdea.value) return router.resolve(ideaReturnLocation(ideaId.value, router.hasRoute('idea-work'))).fullPath
  if (launch.data.value) return `/packs/${packId.value}`
  if (isBuiltin.value) {
    return router.resolve({ name: 'create', query: { ...route.query, tool: undefined, game: undefined } }).fullPath
  }
  return '/create'
}

async function close() {
  if (closing.value) return
  closing.value = true
  try {
    // Replace the host entry directly. A work may have added its own frame history.
    await router.replace(returnPage())
  } finally {
    closing.value = false
  }
}
</script>

<template>
  <AppRuntimeWindow :title="title" :icon="icon" :compact="builtin?.id === 'mbti'"
    :orientation="fullscreenOrientation"
    :busy="reloading" :closing="closing" @reload="reload" @close="close">
    <BuiltinGamePlayer v-if="builtin" :key="revision" :game-id="builtin.id" @close="close" />
    <div v-else-if="isBuiltin" class="state" role="alert">找不到这个内置作品</div>
    <CommunityGameTrial v-else-if="ideaId" ref="player" :key="`${ideaId}:${revision}`" :idea-id="ideaId"
      @retry="reload" @exit="close" />
    <div v-else-if="isIdea" class="state" role="alert">找不到这个创意作品</div>
    <AppRuntimeStatus v-else-if="launch.loading.value || launch.error.value" :key="revision"
      message="正在读取作品信息…" :error="launch.error.value" @retry="reload" />
    <div v-else-if="launch.data.value?.mode === 'bundled'" class="bundled">
      <iframe :key="revision" :src="WANCISOU_WEB_ENTRY" :title="title" sandbox="allow-scripts allow-forms"
        allow="clipboard-write" referrerpolicy="no-referrer" @load="bundledReady = true" />
      <AppRuntimeStatus v-if="!bundledReady" class="bundled-status" message="正在加载作品资源…" @retry="reload" />
    </div>
    <CommunityGameTrial v-else-if="launch.data.value?.mode === 'runtime'" ref="player"
      :key="`${packId}:${version}:${revision}`" :pack="launch.data.value.pack" :version="version"
      @retry="reload" @exit="close" />
    <div v-else-if="launch.data.value" class="state">
      <p>请在小伴客户端打开这个作品。</p>
      <RouterLink class="btn" :to="`/packs/${packId}`">查看作品详情</RouterLink>
    </div>
  </AppRuntimeWindow>
</template>

<style scoped>
.bundled { position: relative; height: 100%; }
.bundled iframe { display: block; width: 100%; height: 100%; border: 0; background: var(--surface-2); }
.state { display: grid; align-content: center; justify-items: center; gap: 12px; height: 100%; padding: 20px; text-align: center; color: var(--ink-2); }
.state p { margin: 0; overflow-wrap: anywhere; }
</style>
