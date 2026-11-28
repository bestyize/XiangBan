<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import type { GameItem } from '../data/games';
import { loadProto1Save, writeProto1Save, type StorageAdapter } from '../lib/storage';
import UiIcon from './UiIcon.vue';

const props = defineProps<{
  game: GameItem;
  dark: boolean;
  muted: boolean;
  rotateGuideEnabled: boolean;
  storage: StorageAdapter;
}>();

const emit = defineEmits<{
  exit: [playedSeconds: number];
  score: [gameId: string, score: number];
  saveUpdated: [gameId: string];
  toggleMute: [];
  share: [game: GameItem];
}>();

const stageRef = ref<HTMLElement | null>(null);
const frameRef = ref<HTMLIFrameElement | null>(null);
const generation = ref(1);
const nonce = ref(makeNonce());
const ready = ref(false);
const hostMenuOpen = ref(false);
const dismissedRotate = ref(false);
const viewportW = ref(window.innerWidth);
const viewportH = ref(window.innerHeight);

const startedAt = Date.now();

function makeNonce(): string {
  return typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function'
    ? crypto.randomUUID()
    : `xb-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

const isLandscapeViewport = computed(() => viewportW.value >= viewportH.value);
const isTouchDevice = computed(() =>
  typeof window !== 'undefined' && window.matchMedia('(pointer: coarse)').matches
);

const showRotateOverlay = computed(() => {
  if (!props.rotateGuideEnabled || dismissedRotate.value) return false;
  if (props.game.orient !== 'landscape') return false;
  return isTouchDevice.value && !isLandscapeViewport.value;
});

const frameUrl = computed(() => {
  const params = new URLSearchParams({
    hostNonce: nonce.value,
    xbTheme: props.dark ? 'dark' : 'light',
    xbMuted: props.muted ? '1' : '0'
  });
  return `./game-packs/${props.game.id}/index.html?${params.toString()}`;
});

function sendInit() {
  const win = frameRef.value?.contentWindow;
  if (!win) return;
  const save = loadProto1Save(props.storage, props.game.id);
  const safeArea = { top: 0, right: 0, bottom: 0, left: 0 };
  // Protocol 1 handshake
  win.postMessage(
    {
      channel: 'xiaban.game-host',
      protocol: 1,
      type: 'host.init',
      nonce: nonce.value,
      save,
      safeArea
    },
    '*'
  );
  // Protocol 2 safeArea & capsule event
  win.postMessage(
    {
      channel: 'xiaban.host',
      protocol: 2,
      kind: 'event',
      name: 'safearea',
      data: {
        safeArea,
        capsule: { top: 8, right: 8, width: 44, height: 44 }
      }
    },
    '*'
  );
  // Fallback ready if a game has no explicit handshake
  window.setTimeout(() => {
    ready.value = true;
  }, 600);
}

function handleMessage(ev: MessageEvent) {
  const win = frameRef.value?.contentWindow;
  if (!win || ev.source !== win || !ev.data || typeof ev.data !== 'object') return;
  const d = ev.data as Record<string, unknown>;

  // Protocol 1 messages
  if (d.channel === 'xiaban.game-pack' && d.protocol === 1 && d.nonce === nonce.value) {
    if (d.type === 'game.ready') {
      ready.value = true;
    } else if (d.type === 'game.exit') {
      handleExit();
    } else if ((d.type === 'game.progress' || d.type === 'game.result') && typeof d.score === 'number') {
      emit('score', props.game.id, d.score);
    } else if (d.type === 'game.save' && 'save' in d) {
      writeProto1Save(props.storage, props.game.id, d.save);
      emit('saveUpdated', props.game.id);
    }
    return;
  }

  // Protocol 2 bridge messages
  if (d.channel === 'xiaban.app' && d.protocol === 2) {
    const kind = d.kind;
    const payload = (d.payload || {}) as Record<string, unknown>;
    if (kind === 'ready') {
      ready.value = true;
    } else if (kind === 'exit') {
      handleExit();
    } else if ((kind === 'game.progress' || kind === 'game.result') && typeof payload.score === 'number') {
      emit('score', props.game.id, payload.score);
    } else if (kind === 'storage.changed') {
      emit('saveUpdated', props.game.id);
    }
  }
}

async function tryFullscreenLandscape() {
  try {
    const el = stageRef.value;
    if (el && el.requestFullscreen && !document.fullscreenElement) {
      await el.requestFullscreen();
    }
    const ori = screen.orientation as unknown as { lock?: (mode: string) => Promise<void> };
    if (ori?.lock) {
      await ori.lock('landscape');
    }
    dismissedRotate.value = true;
  } catch {
    // iOS Safari / 微信不支持自动锁横屏时，提示用户物理旋转设备或直接放行
    dismissedRotate.value = true;
  }
}

function restartGame() {
  hostMenuOpen.value = false;
  ready.value = false;
  nonce.value = makeNonce();
  generation.value++;
}

function handleExit() {
  hostMenuOpen.value = false;
  if (document.fullscreenElement) {
    document.exitFullscreen().catch(() => {});
  }
  const seconds = Math.max(1, Math.round((Date.now() - startedAt) / 1000));
  emit('exit', seconds);
}

function onResize() {
  viewportW.value = window.innerWidth;
  viewportH.value = window.innerHeight;
  const win = frameRef.value?.contentWindow;
  if (win) {
    win.postMessage(
      {
        channel: 'xiaban.game-host',
        protocol: 1,
        type: 'host.layout',
        nonce: nonce.value,
        safeArea: { top: 0, right: 0, bottom: 0, left: 0 }
      },
      '*'
    );
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    hostMenuOpen.value = !hostMenuOpen.value;
  }
}

watch(
  () => props.dark,
  (isDark) => {
    frameRef.value?.contentWindow?.postMessage(
      { channel: 'xiaban.host', protocol: 2, kind: 'event', name: 'theme', data: { theme: isDark ? 'dark' : 'light' } },
      '*'
    );
  }
);

onMounted(() => {
  window.addEventListener('message', handleMessage);
  window.addEventListener('resize', onResize);
  window.addEventListener('keydown', onKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener('message', handleMessage);
  window.removeEventListener('resize', onResize);
  window.removeEventListener('keydown', onKeydown);
  if (document.fullscreenElement) {
    document.exitFullscreen().catch(() => {});
  }
});
</script>

<template>
  <section ref="stageRef" class="player-root" :aria-label="game.name">
    <!-- 协议 2 右上角无圆底透明阴影竖三点（规范 4.3） -->
    <button
      type="button"
      class="host-more-btn"
      title="游戏菜单（Esc）"
      aria-label="打开游戏菜单"
      @click="hostMenuOpen = true"
    >
      <UiIcon name="moreV" :size="18" />
    </button>

    <!-- 加载中遮罩 -->
    <div v-if="!ready" class="loading-curtain">
      <img class="loading-icon" :src="game.icon" :alt="game.name" />
      <b>{{ game.name }}</b>
      <small>正在加载本地资源…</small>
    </div>

    <!-- 游戏真机画布 -->
    <div class="frame-stage" :class="{ 'desktop-landscape': !isTouchDevice && game.orient === 'landscape', 'desktop-portrait': !isTouchDevice && game.orient !== 'landscape' }">
      <iframe
        :key="generation"
        ref="frameRef"
        :src="frameUrl"
        :title="`${game.name} 游戏画面`"
        class="game-frame"
        allow="fullscreen; autoplay"
        allowfullscreen
        @load="sendInit"
      />
    </div>

    <!-- 横屏旋转引导蒙层（iOS / 微信兜底） -->
    <div v-if="showRotateOverlay" class="rotate-overlay">
      <img class="rotate-bg" :src="game.cover" alt="" />
      <div class="rotate-content">
        <div class="rotate-chip">
          <img :src="game.icon" alt="" />
          <span>{{ game.name }} · 横屏作品</span>
        </div>
        <div class="rotate-glyph" aria-hidden="true">
          <svg class="rotate-svg" width="32" height="32" viewBox="0 0 24 24" stroke="#ffffff" stroke-width="1.7">
            <rect x="6.5" y="3" width="11" height="18" rx="2.5" />
            <path d="M10.5 17.5h3" />
          </svg>
        </div>
        <h3>横过手机，沉浸开玩</h3>
        <p>《{{ game.name }}》专为横屏宽视野设计，旋转设备后自动进入全屏，双手操作更顺手。</p>
        <button type="button" class="rotate-primary" @click="tryFullscreenLandscape">开启全屏横屏</button>
        <div class="rotate-links">
          <button type="button" @click="dismissedRotate = true">暂不旋转，继续查看</button>
          <span class="sep">·</span>
          <button type="button" @click="handleExit">返回大厅</button>
        </div>
      </div>
    </div>

    <!-- 宿主菜单：竖屏弹底部 20px BottomSheet，横屏弹右侧 260px 窄抽屉 -->
    <div
      v-if="hostMenuOpen"
      class="host-scrim"
      :class="{ side: isLandscapeViewport }"
      @click="hostMenuOpen = false"
    >
      <div :class="isLandscapeViewport ? 'side-drawer' : 'host-sheet'" @click.stop>
        <div>
          <div v-if="!isLandscapeViewport" class="sheet-handle" />
          <div class="host-head">
            <img :src="game.icon" alt="" />
            <div class="host-head-text">
              <b>{{ game.name }}</b>
              <span>本机进度自动保存 · game.xiaban.xyz</span>
            </div>
          </div>
          <div class="host-actions" :class="{ two: isLandscapeViewport }">
            <button type="button" class="host-act" @click="restartGame">
              <span class="host-act-ico"><UiIcon name="refresh" :size="18" /></span>
              <span>重新开始</span>
            </button>
            <button type="button" class="host-act" @click="emit('toggleMute')">
              <span class="host-act-ico"><UiIcon :name="muted ? 'volumeOff' : 'volume'" :size="18" /></span>
              <span>{{ muted ? '已静音' : '音效开启' }}</span>
            </button>
            <button type="button" class="host-act" @click="emit('share', game)">
              <span class="host-act-ico"><UiIcon name="share" :size="18" /></span>
              <span>发给好友</span>
            </button>
            <button type="button" class="host-act" @click="hostMenuOpen = false">
              <span class="host-act-ico"><UiIcon name="play" :size="15" /></span>
              <span>继续游玩</span>
            </button>
          </div>
        </div>
        <button type="button" class="host-exit-btn" @click="handleExit">退出回到大厅</button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.player-root {
  position: fixed;
  inset: 0;
  z-index: 80;
  width: 100vw;
  height: 100dvh;
  background: #0e0e12;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.host-more-btn {
  position: fixed;
  top: calc(8px + env(safe-area-inset-top, 0px));
  right: calc(8px + env(safe-area-inset-right, 0px));
  z-index: 90;
  width: 44px;
  height: 44px;
  display: grid;
  place-items: center;
  color: #ffffff;
  filter: drop-shadow(0 1px 4px rgba(0, 0, 0, 0.65));
}
.host-more-btn:active { transform: scale(0.9); }

.loading-curtain {
  position: absolute;
  inset: 0;
  z-index: 82;
  background: var(--paper);
  color: var(--ink);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}
.loading-icon {
  width: 64px; height: 64px; border-radius: 16px;
  border: 1px solid var(--hairline);
}
.loading-curtain b { font-size: 16px; font-weight: 700; }
.loading-curtain small { font-size: 12.5px; color: var(--ink2); }

.frame-stage {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}
.game-frame {
  width: 100%;
  height: 100%;
  border: 0;
  display: block;
  background: #0e0e12;
}
@media (min-width: 760px) {
  .frame-stage.desktop-portrait .game-frame {
    width: min(460px, 100vw);
    height: min(860px, 100dvh);
    border-radius: 20px;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
  }
  .frame-stage.desktop-landscape .game-frame {
    width: min(1200px, 100vw);
    height: min(720px, 100dvh);
    border-radius: 18px;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
  }
}

.rotate-overlay {
  position: absolute; inset: 0; z-index: 86;
  display: flex; align-items: center; justify-content: center;
  padding: 28px 24px; text-align: center; color: #fff; background: #141418;
  overflow: hidden;
}
.rotate-bg {
  position: absolute; inset: -20px; width: calc(100% + 40px); height: calc(100% + 40px);
  object-fit: cover; filter: blur(24px) brightness(0.42); transform: scale(1.06);
}
.rotate-content {
  position: relative; z-index: 2; display: flex; flex-direction: column;
  align-items: center; max-width: 300px; width: 100%;
}
.rotate-chip {
  display: inline-flex; align-items: center; gap: 7px; padding: 5px 12px 5px 6px;
  border-radius: 999px; background: rgba(255, 255, 255, 0.14);
  border: 1px solid rgba(255, 255, 255, 0.16); font-size: 12px; font-weight: 600; margin-bottom: 18px;
}
.rotate-chip img { width: 22px; height: 22px; border-radius: 6px; }
.rotate-glyph {
  width: 74px; height: 74px; border-radius: 22px;
  background: rgba(255, 255, 255, 0.12); border: 1px solid rgba(255, 255, 255, 0.18);
  backdrop-filter: blur(12px); display: grid; place-items: center; margin-bottom: 18px;
}
.rotate-svg { animation: tiltPhone 2.4s ease-in-out infinite; }
@keyframes tiltPhone {
  0%, 18% { transform: rotate(0deg); }
  48%, 70% { transform: rotate(-90deg); }
  100% { transform: rotate(0deg); }
}
.rotate-content h3 { font-size: 20px; font-weight: 750; margin-bottom: 7px; }
.rotate-content p { font-size: 13px; color: rgba(255, 255, 255, 0.78); line-height: 1.58; margin-bottom: 22px; }
.rotate-primary {
  width: 100%; max-width: 248px; height: 46px; border-radius: 999px;
  background: #ffffff; color: #18181b; font-size: 14.5px; font-weight: 700; margin-bottom: 10px;
}
.rotate-links { display: flex; align-items: center; gap: 14px; font-size: 13px; color: rgba(255, 255, 255, 0.74); }
.rotate-links button { padding: 6px 2px; }
.rotate-links .sep { opacity: 0.35; }

.host-scrim {
  position: fixed; inset: 0; z-index: 92;
  background: rgba(16, 16, 20, 0.48);
  display: flex; align-items: flex-end; justify-content: center;
  animation: fadeIn 0.16s ease;
}
.host-scrim.side { align-items: stretch; justify-content: flex-end; }
.host-sheet {
  width: 100%; max-width: 460px; background: var(--surface); color: var(--ink);
  border-radius: 20px 20px 0 0; padding: 8px 18px calc(12px + env(safe-area-inset-bottom, 0px));
  box-shadow: var(--sheet-shadow);
}
.side-drawer {
  width: min(268px, 76vw); height: 100%; background: var(--surface); color: var(--ink);
  padding: 18px 16px calc(14px + env(safe-area-inset-bottom, 0px));
  display: flex; flex-direction: column; justify-content: space-between;
  box-shadow: -10px 0 36px rgba(0, 0, 0, 0.28);
}
.host-head {
  display: flex; align-items: center; gap: 10px;
  padding-bottom: 14px; border-bottom: 0.5px solid var(--hairline);
}
.host-head img { width: 40px; height: 40px; border-radius: 10px; border: 1px solid var(--hairline); flex: none; }
.host-head-text { min-width: 0; }
.host-head b { display: block; font-size: 15px; font-weight: 700; }
.host-head span { display: block; font-size: 11.5px; color: var(--ink2); margin-top: 1px; }
.host-actions { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; padding: 16px 0; }
.host-actions.two { grid-template-columns: repeat(2, 1fr); gap: 12px 8px; }
.host-act { display: flex; flex-direction: column; align-items: center; gap: 6px; font-size: 12px; color: var(--ink2); }
.host-act-ico {
  width: 48px; height: 48px; border-radius: 14px; background: var(--fill);
  display: grid; place-items: center; color: var(--ink);
}
.host-exit-btn {
  width: 100%; height: 46px; border-top: 0.5px solid var(--hairline);
  font-size: 15px; font-weight: 650; color: var(--ink);
}
</style>
