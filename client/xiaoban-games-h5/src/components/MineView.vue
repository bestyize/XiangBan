<script setup lang="ts">
import { computed, ref } from 'vue';
import { ACCENTS, GAMES, type GameItem } from '../data/games';
import type { AuthSession } from '../lib/accountApi';
import {
  estimateStorageBytes,
  exportBackupCode,
  formatBytes,
  importBackupCode,
  type GameRecord,
  type StorageAdapter
} from '../lib/storage';
import UiIcon from './UiIcon.vue';

const props = defineProps<{
  dark: boolean;
  accentId: string;
  muted: boolean;
  rotateGuide: boolean;
  favorites: string[];
  records: Record<string, GameRecord>;
  session: AuthSession | null;
  storage: StorageAdapter;
}>();

const emit = defineEmits<{
  toggleTheme: [];
  cycleAccent: [];
  toggleMute: [];
  toggleRotateGuide: [];
  openAuth: [];
  logout: [];
  goRecords: [];
  launch: [game: GameItem];
  clearAll: [];
  imported: [count: number];
  toast: [msg: string];
}>();

const subSheet = ref<null | 'favorites' | 'saves' | 'backup'>(null);
const confirmClear = ref(false);
const backupExportText = ref('');
const backupImportText = ref('');

const playedCount = computed(() =>
  GAMES.filter((g) => (props.records[g.id]?.playCount || 0) > 0).length
);
const savedGames = computed(() =>
  GAMES.filter((g) => props.records[g.id]?.hasSave)
);
const favGames = computed(() =>
  GAMES.filter((g) => props.favorites.includes(g.id))
);
const accentName = computed(() =>
  (ACCENTS.find((a) => a.id === props.accentId) || ACCENTS[0]).name
);
const storageSizeText = computed(() => formatBytes(estimateStorageBytes(props.storage)));
const avatarInitial = computed(() => (props.session?.user.username || '伴').slice(0, 1).toUpperCase());

function openBackup() {
  backupExportText.value = exportBackupCode(props.storage);
  backupImportText.value = '';
  subSheet.value = 'backup';
}

async function copyBackupCode() {
  try {
    await navigator.clipboard.writeText(backupExportText.value);
    emit('toast', '已复制存档备份码，可粘贴到备忘录或新手机导入');
  } catch {
    emit('toast', '请长按文本框全选复制存档码');
  }
}

function submitImport() {
  if (!backupImportText.value.trim()) {
    emit('toast', '请先粘贴 XB1: 开头的存档码');
    return;
  }
  const res = importBackupCode(props.storage, backupImportText.value);
  if (!res.ok) {
    emit('toast', res.error || '导入失败');
    return;
  }
  subSheet.value = null;
  emit('imported', res.count);
}

function confirmClearAction() {
  confirmClear.value = false;
  emit('clearAll');
}
</script>

<template>
  <header class="app-topbar solid">
    <div class="app-brand">
      <span class="app-brand-name">我的</span>
    </div>
    <div class="top-actions">
      <button type="button" class="icon-btn" title="切换深浅色" aria-label="切换深浅色" @click="emit('toggleTheme')">
        <UiIcon :name="dark ? 'sun' : 'moon'" :size="19" />
      </button>
    </div>
  </header>

  <main class="mine-main">
    <!-- 顶部通栏资料块 + 三宫格（对齐方案 1 通栏精修） -->
    <section class="mine-slab">
      <div class="mine-profile">
        <div class="mine-avatar" :class="{ logged: !!session }">
          <span v-if="session">{{ avatarInitial }}</span>
          <UiIcon v-else name="user" :size="26" />
        </div>
        <div class="mine-info">
          <div class="mine-name-row">
            <span class="mine-name">{{ session ? session.user.username : '掌机访客' }}</span>
            <span class="mine-pill-tag"><i />{{ session ? '已登录' : '免登录' }}</span>
          </div>
          <div class="mine-sub num">
            {{ session ? `ID: ${session.user.account_id} · 相伴统一账号` : `本机游玩 · ${savedGames.length} 份存档已保存` }}
          </div>
        </div>
        <button
          type="button"
          class="mine-auth-btn"
          :class="{ primary: !session }"
          @click="session ? emit('logout') : emit('openAuth')"
        >
          {{ session ? '退出' : '登录同步' }}
        </button>
      </div>
      <div class="mine-divider" />
      <div class="mine-quick">
        <button type="button" class="mine-quick-btn" @click="subSheet = 'favorites'">
          <span class="mine-quick-val num">{{ favGames.length }}</span>
          <span class="mine-quick-lbl"><UiIcon name="star" :size="13" />收藏</span>
        </button>
        <button type="button" class="mine-quick-btn" @click="subSheet = 'saves'">
          <span class="mine-quick-val num">{{ savedGames.length }}</span>
          <span class="mine-quick-lbl"><UiIcon name="save" :size="13" />存档</span>
        </button>
        <button type="button" class="mine-quick-btn" @click="emit('goRecords')">
          <span class="mine-quick-val num">{{ playedCount }}/{{ GAMES.length }}</span>
          <span class="mine-quick-lbl"><UiIcon name="trophy" :size="13" />印记</span>
        </button>
      </div>
    </section>

    <div class="mine-gap" />

    <!-- 通栏组 1：游玩偏好 -->
    <section class="mine-group" aria-label="游玩偏好">
      <button type="button" class="mine-row" @click="emit('toggleTheme')">
        <span class="mine-row-left">
          <span class="mine-row-ico"><UiIcon :name="dark ? 'sun' : 'moon'" :size="19" /></span>
          <span class="mine-row-label">外观模式</span>
        </span>
        <span class="mine-row-right">
          <span>{{ dark ? '掌机深色' : '暖瓷浅色' }}</span>
          <UiIcon name="chevron" :size="15" />
        </span>
      </button>
      <button type="button" class="mine-row" @click="emit('cycleAccent')">
        <span class="mine-row-left">
          <span class="mine-row-ico"><UiIcon name="palette" :size="19" /></span>
          <span class="mine-row-label">主题强调色</span>
        </span>
        <span class="mine-row-right">
          <i class="accent-dot-preview" />
          <span>{{ accentName }}</span>
          <UiIcon name="chevron" :size="15" />
        </span>
      </button>
      <button type="button" class="mine-row" @click="emit('toggleMute')">
        <span class="mine-row-left">
          <span class="mine-row-ico"><UiIcon :name="muted ? 'volumeOff' : 'volume'" :size="19" /></span>
          <span class="mine-row-label">游戏默认音效</span>
        </span>
        <span class="mine-row-right">
          <span class="xb-switch" :class="{ on: !muted }" aria-hidden="true" />
        </span>
      </button>
      <button type="button" class="mine-row" @click="emit('toggleRotateGuide')">
        <span class="mine-row-left">
          <span class="mine-row-ico"><UiIcon name="rotate" :size="19" /></span>
          <span class="mine-row-label">横屏游戏旋转提示</span>
        </span>
        <span class="mine-row-right">
          <span class="xb-switch" :class="{ on: rotateGuide }" aria-hidden="true" />
        </span>
      </button>
    </section>

    <div class="mine-gap" />

    <!-- 通栏组 2：存档与数据 -->
    <section class="mine-group" aria-label="存档与数据">
      <button type="button" class="mine-row" @click="openBackup">
        <span class="mine-row-left">
          <span class="mine-row-ico"><UiIcon name="download" :size="19" /></span>
          <span class="mine-row-label">导出 / 恢复存档码</span>
        </span>
        <span class="mine-row-right">
          <span class="num">{{ savedGames.length }} 份</span>
          <UiIcon name="chevron" :size="15" />
        </span>
      </button>
      <button type="button" class="mine-row" @click="confirmClear = true">
        <span class="mine-row-left">
          <span class="mine-row-ico danger"><UiIcon name="trash" :size="19" /></span>
          <span class="mine-row-label danger">清理本机存档与缓存</span>
        </span>
        <span class="mine-row-right">
          <span class="num">{{ storageSizeText }}</span>
          <UiIcon name="chevron" :size="15" />
        </span>
      </button>
    </section>

    <div class="mine-gap" />

    <!-- 通栏组 3：相伴生态与关于 -->
    <section class="mine-group" aria-label="相伴生态与关于">
      <a class="mine-row" href="https://chat.xiangban.xyz" target="_blank" rel="noopener">
        <span class="mine-row-left">
          <span class="mine-row-ico"><UiIcon name="chat" :size="19" /></span>
          <span class="mine-row-label">小伴社区</span>
        </span>
        <span class="mine-row-right">
          <span class="mono sm-host">chat.xiangban.xyz</span>
          <UiIcon name="chevron" :size="15" />
        </span>
      </a>
      <a class="mine-row" href="https://people.xiaban.xyz" target="_blank" rel="noopener">
        <span class="mine-row-left">
          <span class="mine-row-ico"><UiIcon name="spark" :size="19" /></span>
          <span class="mine-row-label">人格贩卖机</span>
        </span>
        <span class="mine-row-right">
          <span class="mono sm-host">people.xiaban.xyz</span>
          <UiIcon name="chevron" :size="15" />
        </span>
      </a>
      <a class="mine-row" href="https://xiaban.xyz/app" target="_blank" rel="noopener">
        <span class="mine-row-left">
          <span class="mine-row-ico"><UiIcon name="gamepad" :size="19" /></span>
          <span class="mine-row-label">下载小伴 App</span>
        </span>
        <span class="mine-row-right">
          <span>离线畅玩 · AI 创作</span>
          <UiIcon name="chevron" :size="15" />
        </span>
      </a>
      <button
        type="button"
        class="mine-row"
        @click="emit('toast', '小伴游戏厅 v1.0 · 纯静态免登录 · game.xiaban.xyz')"
      >
        <span class="mine-row-left">
          <span class="mine-row-ico"><UiIcon name="info" :size="19" /></span>
          <span class="mine-row-label">关于小伴游戏厅</span>
        </span>
        <span class="mine-row-right">
          <span class="mono sm-host">game.xiaban.xyz</span>
          <UiIcon name="chevron" :size="15" />
        </span>
      </button>
    </section>
  </main>

  <!-- 收藏 / 存档 / 备份码 BottomSheet -->
  <div v-if="subSheet" class="scrim" role="dialog" aria-modal="true" @click="subSheet = null">
    <div class="sheet" @click.stop>
      <div class="sheet-handle" />
      <template v-if="subSheet === 'favorites' || subSheet === 'saves'">
        <div class="sub-head">
          <h3>{{ subSheet === 'favorites' ? '我的收藏' : '本机存档管理' }}</h3>
          <span class="num">{{ (subSheet === 'favorites' ? favGames : savedGames).length }} 款</span>
        </div>
        <div v-if="(subSheet === 'favorites' ? favGames : savedGames).length" class="sub-list">
          <div
            v-for="g in subSheet === 'favorites' ? favGames : savedGames"
            :key="g.id"
            class="sub-row"
          >
            <img :src="g.icon" :alt="g.name" />
            <div class="sub-info">
              <b>{{ g.name }}</b>
              <small>{{ subSheet === 'favorites' ? g.tagline : '本机存档已保存' }}</small>
            </div>
            <button type="button" class="sub-go" @click="subSheet = null; emit('launch', g)">
              {{ subSheet === 'favorites' ? '开玩' : '续玩' }}
            </button>
          </div>
        </div>
        <p v-else class="sub-empty">
          {{ subSheet === 'favorites' ? '暂无收藏，在游戏详情抽屉点星标即可加入。' : '还没有写入本机存档，进游戏玩一会儿会自动保存。' }}
        </p>
      </template>

      <template v-else-if="subSheet === 'backup'">
        <div class="sub-head">
          <h3>导出 / 恢复存档码</h3>
          <span class="num">纯前端跨机迁移</span>
        </div>
        <p class="backup-tip">复制下方存档码发给自己，在另一台手机的浏览器里粘贴即可无损带走全部游戏进度。</p>
        <textarea class="backup-area mono" :value="backupExportText" readonly rows="3" />
        <button type="button" class="backup-btn primary" @click="copyBackupCode">一键复制当前存档码</button>
        <div class="backup-sep" />
        <textarea
          v-model="backupImportText"
          class="backup-area mono"
          placeholder="在此粘贴 XB1: 开头的存档码以恢复进度…"
          rows="3"
        />
        <button type="button" class="backup-btn" @click="submitImport">从存档码恢复进度</button>
      </template>
    </div>
  </div>

  <!-- 24px 居中一次确认清理弹窗（规范 4.1.1） -->
  <div v-if="confirmClear" class="dialog-scrim" role="dialog" aria-modal="true" @click="confirmClear = false">
    <div class="dialog-card" @click.stop>
      <h4>清理本机存档与缓存？</h4>
      <p>将清除当前浏览器中保存的游戏进度与最高分纪录（共 {{ storageSizeText }}）；如想保留进度，可先使用「导出存档码」备份。</p>
      <div class="dialog-actions">
        <button type="button" class="dialog-danger" @click="confirmClearAction">确认清理</button>
        <button type="button" class="dialog-cancel" @click="confirmClear = false">取消</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.mine-main { padding-bottom: 24px; }
.mine-slab { background: var(--surface); }
.mine-profile { padding: 8px 16px 18px; display: flex; align-items: center; gap: 13px; }
.mine-avatar {
  width: 60px; height: 60px; border-radius: 50%; flex: none;
  display: grid; place-items: center; font-size: 22px; font-weight: 750;
  background: var(--fill); color: var(--ink2); box-shadow: 0 0 0 1px var(--hairline);
}
.mine-avatar.logged {
  background: linear-gradient(135deg, #ff5b7d 0%, #f9744b 100%);
  color: #fff; box-shadow: 0 4px 14px rgba(238, 79, 109, 0.25);
}
.mine-info { flex: 1; min-width: 0; }
.mine-name-row { display: flex; align-items: center; gap: 6px; min-width: 0; }
.mine-name {
  font-size: 20px; line-height: 26px; font-weight: 700; letter-spacing: -0.015em;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.mine-pill-tag {
  font-size: 10.5px; font-weight: 600; padding: 1px 7px; border-radius: 999px; flex: none;
  background: var(--fill); color: var(--ink2); display: inline-flex; align-items: center; gap: 4px;
}
.mine-pill-tag i { width: 5.5px; height: 5.5px; border-radius: 50%; background: var(--ok); display: block; }
.mine-sub {
  font-size: 12.5px; color: var(--ink2); margin-top: 3px;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.mine-auth-btn {
  height: 33px; padding: 0 13px; border-radius: 999px; flex: none;
  background: var(--fill); color: var(--ink); font-size: 12.5px; font-weight: 650;
}
.mine-auth-btn.primary { background: var(--accent); color: var(--accent-ink); }
.mine-divider { height: 0.5px; margin: 0 16px; background: var(--hairline); }
.mine-quick { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); padding: 13px 10px 15px; }
.mine-quick-btn {
  display: flex; flex-direction: column; align-items: center; gap: 5px;
  padding: 6px 4px; border-radius: 12px;
}
.mine-quick-btn:active { background: var(--fill); }
.mine-quick-val { font-size: 18px; font-weight: 750; line-height: 22px; color: var(--ink); }
.mine-quick-lbl { font-size: 11.5px; color: var(--ink2); display: inline-flex; align-items: center; gap: 4px; }

.mine-gap { height: 8px; background: var(--paper); }
.mine-group { background: var(--surface); }
.mine-row {
  width: 100%; min-height: 52px; padding: 0 16px;
  display: flex; align-items: center; justify-content: space-between; gap: 12px;
  text-align: left; position: relative; text-decoration: none; color: inherit;
}
.mine-row + .mine-row::before {
  content: ""; position: absolute; top: 0; left: 48px; right: 16px; height: 0.5px; background: var(--hairline);
}
.mine-row:active { background: var(--fill); }
.mine-row-left { display: flex; align-items: center; gap: 12px; min-width: 0; }
.mine-row-ico { width: 20px; height: 20px; color: var(--ink2); display: grid; place-items: center; flex: none; }
.mine-row-ico.danger { color: var(--danger); }
.mine-row-label { font-size: 15px; font-weight: 500; color: var(--ink); white-space: nowrap; }
.mine-row-label.danger { color: var(--danger); }
.mine-row-right { display: flex; align-items: center; gap: 6px; color: var(--ink3); font-size: 13px; min-width: 0; }
.sm-host { font-size: 11.5px; }
.accent-dot-preview { width: 12px; height: 12px; border-radius: 50%; background: var(--accent); flex: none; }

.xb-switch {
  width: 44px; height: 25px; border-radius: 999px; background: var(--fill-hover);
  position: relative; transition: background 0.16s ease; flex: none;
}
.xb-switch::after {
  content: ""; position: absolute; top: 2.5px; left: 2.5px; width: 20px; height: 20px;
  border-radius: 50%; background: #fff; box-shadow: 0 1px 3px rgba(0, 0, 0, 0.22);
  transition: transform 0.16s cubic-bezier(0.22, 1, 0.36, 1);
}
.xb-switch.on { background: var(--accent); }
.xb-switch.on::after { transform: translateX(19px); }

.sub-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
.sub-head h3 { font-size: 16px; font-weight: 750; }
.sub-head span { font-size: 12px; color: var(--ink3); }
.sub-list {
  background: var(--paper); border: 1px solid var(--hairline); border-radius: 14px; overflow: hidden;
}
.sub-row { display: flex; align-items: center; gap: 10px; padding: 10px 12px; position: relative; }
.sub-row + .sub-row::before {
  content: ""; position: absolute; top: 0; left: 60px; right: 12px; height: 0.5px; background: var(--hairline);
}
.sub-row img { width: 38px; height: 38px; border-radius: 10px; border: 1px solid var(--hairline); flex: none; }
.sub-info { flex: 1; min-width: 0; }
.sub-info b { display: block; font-size: 14px; font-weight: 650; }
.sub-info small {
  display: block; font-size: 11.5px; color: var(--ink2);
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-top: 1px;
}
.sub-go {
  height: 30px; padding: 0 12px; border-radius: 999px;
  background: var(--fill); color: var(--ink); font-size: 12px; font-weight: 650; flex: none;
}
.sub-empty { text-align: center; color: var(--ink3); padding: 26px 8px; font-size: 13px; }

.backup-tip { font-size: 12.5px; color: var(--ink2); line-height: 1.55; margin-bottom: 10px; }
.backup-area {
  width: 100%; padding: 10px 12px; border-radius: 12px; background: var(--fill);
  font-size: 11.5px; line-height: 1.5; resize: none; margin-bottom: 10px; word-break: break-all;
}
.backup-btn {
  width: 100%; height: 42px; border-radius: 999px;
  background: var(--fill); color: var(--ink); font-size: 13.5px; font-weight: 650;
}
.backup-btn.primary { background: var(--accent); color: var(--accent-ink); }
.backup-sep { height: 1px; background: var(--hairline); margin: 14px 0; }

.dialog-actions { display: flex; flex-direction: column; gap: 8px; }
.dialog-danger {
  width: 100%; height: 46px; border-radius: 999px; background: var(--danger); color: #fff;
  font-size: 14.5px; font-weight: 700;
}
.dialog-cancel { width: 100%; height: 42px; border-radius: 999px; color: var(--ink2); font-size: 14px; font-weight: 600; }
</style>
