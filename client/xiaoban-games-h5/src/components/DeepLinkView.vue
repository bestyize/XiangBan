<script setup lang="ts">
import { GAMES, type GameItem } from '../data/games';
import UiIcon from './UiIcon.vue';

defineProps<{
  game: GameItem;
}>();

const emit = defineEmits<{
  backHall: [];
  launch: [game: GameItem];
}>();
</script>

<template>
  <main class="deeplink-stage">
    <button type="button" class="deeplink-back" @click="emit('backHall')">
      <UiIcon name="back" :size="16" />全部游戏 ({{ GAMES.length }})
    </button>

    <section class="deeplink-card">
      <div class="deeplink-cover">
        <img :src="game.cover" :alt="game.name" />
        <span class="cover-meta num">{{ game.orientLabel }} · {{ game.size }}</span>
      </div>
      <div class="deeplink-body">
        <div class="deeplink-from"><i />好友分享 · 直达作品</div>
        <h1 class="deeplink-title">{{ game.name }}</h1>
        <p class="deeplink-desc">{{ game.desc }}</p>
        <div class="deeplink-tags">
          <span>{{ game.genre }}</span>
          <span>{{ game.controls }}</span>
          <span class="num">v{{ game.version }}</span>
        </div>
        <button type="button" class="deeplink-play" @click="emit('launch', game)">
          <UiIcon name="play" :size="14" />立即进入游戏
        </button>
      </div>
    </section>

    <footer class="deeplink-foot">
      小伴游戏厅 · <span class="mono">game.xiaban.xyz</span> · 零广告 · 免登录
    </footer>
  </main>
</template>

<style scoped>
.deeplink-stage {
  min-height: 100dvh;
  padding: calc(14px + env(safe-area-inset-top, 0px)) 18px calc(24px + env(safe-area-inset-bottom, 0px));
  display: flex; flex-direction: column; justify-content: space-between;
}
.deeplink-back {
  display: inline-flex; align-items: center; gap: 4px; height: 36px; padding: 0 13px 0 8px;
  border-radius: 999px; background: var(--fill); color: var(--ink); font-size: 13px; font-weight: 600;
  align-self: flex-start;
}
.deeplink-card {
  background: var(--surface); border: 1px solid var(--hairline);
  border-radius: 22px; overflow: hidden; margin: 20px 0;
}
.deeplink-cover { position: relative; aspect-ratio: 16 / 9.5; background: #18181b; }
.deeplink-cover img { width: 100%; height: 100%; object-fit: cover; }
.cover-meta {
  position: absolute; right: 10px; bottom: 10px; padding: 3px 9px; border-radius: 999px;
  background: rgba(16, 16, 20, 0.72); color: #fff; font-size: 11px; font-weight: 600;
}
.deeplink-body { padding: 18px; }
.deeplink-from {
  display: inline-flex; align-items: center; gap: 6px; font-size: 12px; font-weight: 600;
  color: var(--ink2); margin-bottom: 8px;
}
.deeplink-from i { width: 6px; height: 6px; border-radius: 50%; background: var(--accent); display: block; }
.deeplink-title { font-size: 22px; font-weight: 750; margin-bottom: 6px; }
.deeplink-desc { font-size: 13.5px; color: var(--ink2); line-height: 1.6; margin-bottom: 14px; }
.deeplink-tags { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 16px; }
.deeplink-tags span {
  font-size: 11.5px; padding: 3px 9px; border-radius: 999px; background: var(--fill); color: var(--ink2);
}
.deeplink-play {
  width: 100%; height: 48px; border-radius: 999px;
  background: var(--accent); color: var(--accent-ink); font-size: 15px; font-weight: 700;
  display: inline-flex; align-items: center; justify-content: center; gap: 6px;
}
.deeplink-foot { text-align: center; font-size: 12px; color: var(--ink3); }
</style>
