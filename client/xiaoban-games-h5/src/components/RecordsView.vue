<script setup lang="ts">
import { computed } from 'vue';
import { GAMES, formatScore, type GameItem } from '../data/games';
import { formatPlayTime, formatRelativeTime, type GameRecord } from '../lib/storage';
import UiIcon from './UiIcon.vue';

const props = defineProps<{
  records: Record<string, GameRecord>;
  userName?: string | null;
}>();

const emit = defineEmits<{
  launch: [game: GameItem];
  goHall: [];
}>();

const playedGames = computed(() => {
  return GAMES.filter((g) => props.records[g.id] && props.records[g.id].playCount > 0).sort(
    (a, b) => (props.records[b.id]?.lastPlayedAt || 0) - (props.records[a.id]?.lastPlayedAt || 0)
  );
});

const savedCount = computed(() =>
  GAMES.filter((g) => props.records[g.id]?.hasSave).length
);

const totalSeconds = computed(() =>
  Object.values(props.records).reduce((sum, r) => sum + (r.playSeconds || 0), 0)
);

function recordMeta(g: GameItem): string {
  const rec = props.records[g.id];
  if (!rec) return '';
  const parts: string[] = [];
  if (rec.bestScore > 0) parts.push(`最高 ${formatScore(g, rec.bestScore)}`);
  else parts.push(`游玩 ${rec.playCount} 次`);
  if (rec.lastPlayedAt) parts.push(formatRelativeTime(rec.lastPlayedAt));
  return parts.join(' · ');
}
</script>

<template>
  <header class="app-topbar">
    <div class="app-brand">
      <span class="app-brand-name">战绩与存档</span>
    </div>
    <span class="local-pill"><i />{{ userName ? '账号已登录' : '本机保存' }}</span>
  </header>

  <main class="records-main">
    <section class="profile-hero">
      <div class="profile-top">
        <div class="profile-avatar"><UiIcon name="gamepad" :size="20" /></div>
        <div>
          <div class="profile-title">{{ userName || '掌机访客' }}</div>
          <div class="profile-sub">{{ userName ? '相伴统一账号 · 本机存档实时保留' : '免登录 · 进度保存在当前浏览器' }}</div>
        </div>
      </div>
      <div class="stat-grid">
        <div class="stat-box">
          <div class="stat-val num">{{ playedGames.length }}/{{ GAMES.length }}</div>
          <div class="stat-lbl">已玩游戏</div>
        </div>
        <div class="stat-box">
          <div class="stat-val num">{{ savedCount }}</div>
          <div class="stat-lbl">本机存档</div>
        </div>
        <div class="stat-box">
          <div class="stat-val num">{{ formatPlayTime(totalSeconds) }}</div>
          <div class="stat-lbl">累计游玩</div>
        </div>
      </div>
    </section>

    <div class="sec-head">
      <h2 class="sec-title">游玩印记</h2>
      <span class="sec-count">按最近游玩</span>
    </div>

    <div v-if="playedGames.length" class="record-card">
      <div v-for="g in playedGames" :key="g.id" class="rec-row">
        <img :src="g.icon" :alt="g.name" />
        <div class="rec-info">
          <div class="rec-name">
            <span>{{ g.name }}</span>
            <span v-if="records[g.id]?.hasSave" class="mini-tag">有存档</span>
          </div>
          <div class="rec-meta num">{{ recordMeta(g) }}</div>
        </div>
        <button type="button" class="rec-go" @click="emit('launch', g)">续玩</button>
      </div>
    </div>

    <div v-else class="empty-box">
      <p>还没有游玩纪录，去大厅挑一款开一局吧。</p>
      <button type="button" class="empty-btn" @click="emit('goHall')">去大厅挑游戏</button>
    </div>

    <div class="sec-head">
      <h2 class="sec-title">相伴小站</h2>
      <span class="sec-count mono">xiaban.xyz</span>
    </div>
    <div class="sister-grid">
      <a class="sister-card" href="https://people.xiaban.xyz" target="_blank" rel="noopener">
        <div class="sister-top">
          <span class="sister-ico"><UiIcon name="spark" :size="16" /></span>
          <UiIcon name="arrowUpRight" :size="14" />
        </div>
        <b>人格贩卖机</b>
        <small>2 分钟趣味盲盒测评与双人合盘</small>
      </a>
      <a class="sister-card" href="https://chat.xiangban.xyz" target="_blank" rel="noopener">
        <div class="sister-top">
          <span class="sister-ico"><UiIcon name="chat" :size="16" /></span>
          <UiIcon name="arrowUpRight" :size="14" />
        </div>
        <b>小伴社区</b>
        <small>交流攻略、发现社区创意作品</small>
      </a>
    </div>
  </main>
</template>

<style scoped>
.records-main { padding: 6px 16px 24px; }
.local-pill {
  font-size: 11px; font-weight: 600; padding: 3px 9px; border-radius: 999px;
  background: var(--fill); color: var(--ink2); display: inline-flex; align-items: center; gap: 5px;
}
.local-pill i { width: 6px; height: 6px; border-radius: 50%; background: var(--ok); display: block; }

.profile-hero {
  background: var(--surface); border: 1px solid var(--hairline);
  border-radius: 18px; padding: 15px; margin-bottom: 14px;
}
.profile-top { display: flex; align-items: center; gap: 11px; margin-bottom: 13px; }
.profile-avatar {
  width: 44px; height: 44px; border-radius: 13px;
  background: var(--fill); display: grid; place-items: center; color: var(--ink);
}
.profile-title { font-size: 16px; font-weight: 750; }
.profile-sub { font-size: 12px; color: var(--ink2); margin-top: 2px; }
.stat-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }
.stat-box { background: var(--fill); border-radius: 12px; padding: 10px 10px 9px; }
.stat-val { font-size: 19px; font-weight: 800; letter-spacing: -0.02em; }
.stat-lbl { font-size: 11px; color: var(--ink2); margin-top: 2px; }

.sec-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 9px; }
.sec-title { font-size: 15px; font-weight: 700; }
.sec-count { font-size: 12px; color: var(--ink3); }

.record-card {
  background: var(--surface); border: 1px solid var(--hairline);
  border-radius: 16px; overflow: hidden; margin-bottom: 16px;
}
.rec-row { display: flex; align-items: center; gap: 10px; padding: 11px 12px; position: relative; }
.rec-row + .rec-row::before {
  content: ""; position: absolute; top: 0; left: 60px; right: 12px; height: 0.5px; background: var(--hairline);
}
.rec-row img { width: 40px; height: 40px; border-radius: 10px; border: 1px solid var(--hairline); flex: none; }
.rec-info { flex: 1; min-width: 0; }
.rec-name { font-size: 14px; font-weight: 650; display: flex; align-items: center; gap: 6px; }
.mini-tag {
  font-size: 10px; font-weight: 600; padding: 1px 5px; border-radius: 5px;
  background: var(--fill); color: var(--ink2);
}
.rec-meta { font-size: 11.5px; color: var(--ink2); margin-top: 2px; }
.rec-go {
  height: 30px; padding: 0 12px; border-radius: 999px;
  background: var(--fill); color: var(--ink); font-size: 12px; font-weight: 650; flex: none;
}

.empty-box {
  background: var(--surface); border: 1px solid var(--hairline);
  border-radius: 16px; padding: 28px 18px; text-align: center; margin-bottom: 16px;
}
.empty-box p { font-size: 13px; color: var(--ink2); margin-bottom: 14px; }
.empty-btn {
  height: 36px; padding: 0 18px; border-radius: 999px;
  background: var(--accent); color: var(--accent-ink); font-size: 13px; font-weight: 650;
}

.sister-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 9px; }
.sister-card {
  background: var(--surface); border: 1px solid var(--hairline); border-radius: 15px;
  padding: 12px; text-align: left; display: flex; flex-direction: column; gap: 6px;
  text-decoration: none; color: inherit;
}
.sister-top { display: flex; align-items: center; justify-content: space-between; color: var(--ink3); }
.sister-ico {
  width: 30px; height: 30px; border-radius: 9px; background: var(--fill);
  display: grid; place-items: center; color: var(--ink);
}
.sister-card b { font-size: 13.5px; font-weight: 700; }
.sister-card small { font-size: 11.5px; color: var(--ink2); line-height: 1.45; }
</style>
