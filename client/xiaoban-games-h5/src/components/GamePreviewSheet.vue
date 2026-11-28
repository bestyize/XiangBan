<script setup lang="ts">
import { formatScore, type GameItem } from '../data/games';
import type { GameRecord } from '../lib/storage';
import UiIcon from './UiIcon.vue';

const props = defineProps<{
  game: GameItem;
  record?: GameRecord;
  isFavorite: boolean;
}>();

const emit = defineEmits<{
  close: [];
  toggleFav: [id: string];
  share: [game: GameItem];
  launch: [game: GameItem];
}>();

function statRight(): string {
  if (props.record && props.record.bestScore > 0) {
    return formatScore(props.game, props.record.bestScore);
  }
  if (props.record?.playCount) {
    return `已玩 ${props.record.playCount} 次`;
  }
  return '尚未挑战';
}
</script>

<template>
  <div class="scrim" role="dialog" aria-modal="true" :aria-label="game.name" @click="emit('close')">
    <div class="sheet" @click.stop>
      <div class="sheet-handle" />
      <div class="sheet-cover">
        <img :src="game.cover" :alt="game.name" />
        <span class="sheet-cover-meta num">v{{ game.version }} · {{ game.size }} · 免登录</span>
      </div>
      <div class="sheet-head">
        <div class="sheet-icon"><img :src="game.icon" alt="" /></div>
        <div>
          <h3 class="sheet-title">{{ game.name }}</h3>
          <div class="sheet-tags">
            <span class="sheet-tag">{{ game.genre }}</span>
            <span class="sheet-tag">
              <i class="dot" :style="{ background: game.orient === 'landscape' ? '#d97706' : '#2fa36b' }" />
              {{ game.orient === 'landscape' ? '推荐横屏游玩' : '支持单手竖屏' }}
            </span>
            <span class="sheet-tag">{{ game.controls }}</span>
          </div>
        </div>
      </div>
      <p class="sheet-desc">{{ game.desc }}</p>
      <div class="sheet-stat">
        <span>{{ record?.hasSave ? '本机存档已就绪' : '即开即玩 · 零广告' }}</span>
        <strong class="num">{{ statRight() }}</strong>
      </div>
      <div class="sheet-actions">
        <button
          type="button"
          class="fav-pill-btn"
          :class="{ active: isFavorite }"
          :title="isFavorite ? '取消收藏' : '加入收藏'"
          aria-label="收藏游戏"
          @click="emit('toggleFav', game.id)"
        >
          <UiIcon :name="isFavorite ? 'starFill' : 'star'" :size="19" />
        </button>
        <button type="button" class="btn-share" @click="emit('share', game)">
          <UiIcon name="share" :size="16" />分享
        </button>
        <button type="button" class="btn-primary" @click="emit('launch', game)">
          <UiIcon name="play" :size="13" />{{ game.orient === 'landscape' ? '横屏开玩' : '立即开玩' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.sheet-cover {
  position: relative; border-radius: 14px; overflow: hidden;
  aspect-ratio: 16 / 9; background: #18181b; margin-bottom: 14px;
  border: 1px solid var(--hairline);
}
.sheet-cover img { width: 100%; height: 100%; object-fit: cover; }
.sheet-cover-meta {
  position: absolute; right: 9px; bottom: 9px; padding: 3px 8px; border-radius: 999px;
  background: rgba(16, 16, 20, 0.72); color: #fff; font-size: 10.5px; font-weight: 600;
}
.sheet-head { display: flex; align-items: center; gap: 12px; margin-bottom: 10px; }
.sheet-icon {
  width: 52px; height: 52px; border-radius: 13px; overflow: hidden; flex: none;
  border: 1px solid var(--hairline); background: var(--fill);
}
.sheet-icon img { width: 100%; height: 100%; object-fit: cover; }
.sheet-title { font-size: 18px; font-weight: 750; letter-spacing: -0.01em; }
.sheet-tags { display: flex; flex-wrap: wrap; gap: 5px; margin-top: 5px; }
.sheet-tag {
  font-size: 11px; font-weight: 550; padding: 2px 8px; border-radius: 999px;
  background: var(--fill); color: var(--ink2); display: inline-flex; align-items: center; gap: 4px;
}
.sheet-tag .dot { width: 5.5px; height: 5.5px; border-radius: 50%; }
.sheet-desc { font-size: 13.5px; line-height: 1.6; color: var(--ink2); margin-bottom: 12px; }
.sheet-stat {
  display: flex; align-items: center; justify-content: space-between; gap: 8px;
  padding: 10px 12px; border-radius: 12px; background: var(--fill);
  font-size: 12.5px; color: var(--ink2); margin-bottom: 14px;
}
.sheet-stat strong { color: var(--ink); font-weight: 650; }
.sheet-actions { display: flex; align-items: center; gap: 9px; }
.fav-pill-btn {
  width: 46px; height: 46px; border-radius: 999px; flex: none;
  background: var(--fill); color: var(--ink2); display: grid; place-items: center;
}
.fav-pill-btn.active { color: #d97706; }
.btn-share {
  height: 46px; padding: 0 16px; border-radius: 999px; flex: none;
  background: var(--fill); color: var(--ink); font-size: 13.5px; font-weight: 650;
  display: inline-flex; align-items: center; gap: 6px;
}
.btn-primary {
  height: 46px; padding: 0 18px; border-radius: 999px; flex: 1;
  background: var(--accent); color: var(--accent-ink); font-size: 14.5px; font-weight: 700;
  display: inline-flex; align-items: center; justify-content: center; gap: 6px;
}
.btn-primary:active, .btn-share:active, .fav-pill-btn:active { transform: scale(0.97); }
</style>
