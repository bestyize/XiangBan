<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { FolderOpen, RefreshCw, Save, Trash2 } from "lucide-vue-next";
import { NAlert, NButton, NPopconfirm, NSkeleton, NTag } from "naive-ui";
import type { ControlStore } from "../control-types.ts";
import { coverUrl } from "../../games/catalog.ts";
import type { GameManifest } from "../../games/protocol.ts";

const props = defineProps<{ store: ControlStore }>();
const games = ref<GameManifest[]>([]);
const loading = ref(true);
const catalogError = ref("");
const folderErrors = ref<string[]>([]);
const opening = ref(false);
const builtinGames = computed(() => games.value.filter((game) => game.source !== "user"));
const userGames = computed(() => games.value.filter((game) => game.source === "user"));

function descriptionFor(game: GameManifest) {
  if (game.id === "snake-arena") return "无尽成长或三分钟冲分，吃糖果长成大蛇，反击 AI。";
  return game.description;
}
const message = (error: unknown, fallback: string) => (error instanceof Error ? error.message : typeof error === "string" ? error : fallback);

async function loadGames() {
  loading.value = true;
  catalogError.value = "";
  try {
    const catalog = await props.store.listGames();
    games.value = catalog.games;
    folderErrors.value = catalog.errors;
  } catch (error) {
    catalogError.value = message(error, "游戏资料暂时无法读取。");
  } finally {
    loading.value = false;
  }
}

async function openGameSpace(gameId?: string) {
  if (!props.store.desktop || opening.value) return;
  opening.value = true;
  try {
    await props.store.openGames(gameId);
  } catch (error) {
    props.store.notify(message(error, "游戏空间暂时无法打开。"), "error");
  } finally {
    opening.value = false;
  }
}

async function openFolder() {
  try { await props.store.openUserGamesFolder(); } catch (error) { props.store.notify(message(error, "无法打开游戏文件夹"), "error"); }
}

async function removeGame(game: GameManifest) {
  try {
    await props.store.deleteUserGame(game.id);
    props.store.notify(`已删除游戏「${game.name}」`, "success");
  } catch (error) {
    props.store.notify(message(error, "游戏暂时无法删除。"), "error");
  }
  await loadGames();
}

onMounted(loadGames);
</script>

<template>
  <main class="ui-page games-page" data-testid="games-page">
    <NAlert v-if="!store.desktop" class="browser-note" type="info" :bordered="false">
      浏览器预览只展示游戏资料；请在小伴桌面应用中进入游戏空间。
    </NAlert>

    <section class="games-section" aria-labelledby="games-heading">
      <div class="section-heading">
        <div>
          <h2 id="games-heading">本地游戏</h2>
          <p class="muted">共 {{ builtinGames.length }} 款 · 点击开始游戏</p>
        </div>
        <div class="heading-actions">
          <NButton v-if="store.desktop" round data-testid="games-reload" @click="loadGames"><template #icon><RefreshCw :size="15" /></template>重新载入</NButton>
          <NTag v-if="!loading" :bordered="false" type="info">离线可玩</NTag>
        </div>
      </div>

      <NAlert v-if="catalogError" class="catalog-error" type="warning" :bordered="false">
        {{ catalogError }}
      </NAlert>

      <div v-if="loading" class="game-grid" aria-label="正在读取游戏资料">
        <article v-for="item in 6" :key="item" class="surface game-card-skeleton">
          <NSkeleton height="142px" />
          <NSkeleton text :repeat="2" />
        </article>
      </div>
      <div v-else-if="builtinGames.length" class="game-grid">
        <article v-for="game in builtinGames" :key="game.id" class="surface game-card" role="button" tabindex="0" :aria-label="`开始${game.name}`" :aria-disabled="opening || !store.desktop" @click="openGameSpace(game.id)" @keydown.enter.prevent="openGameSpace(game.id)" @keydown.space.prevent="openGameSpace(game.id)">
          <div class="game-cover">
            <img :src="coverUrl(game)" :alt="`${game.name}封面`" loading="lazy" />
            <NTag class="game-kind" :bordered="false">{{ game.orientation === "landscape" ? "横屏" : "轻松益智" }}</NTag>
          </div>
          <div class="game-content">
            <h3>{{ game.name }}</h3>
            <p class="muted">{{ descriptionFor(game) }}</p>
            <div v-if="game.save" class="game-footnote"><Save :size="14" /> 进度保存在本机</div>
          </div>
        </article>
      </div>
      <NAlert v-else-if="!catalogError" type="warning" :bordered="false">暂时没有可展示的游戏资料，请稍后重试。</NAlert>
    </section>

    <section v-if="store.desktop && !loading" class="games-section" aria-labelledby="user-games-heading" data-testid="user-games">
      <div class="section-heading">
        <div>
          <h2 id="user-games-heading">自创游戏</h2>
          <p class="muted">{{ userGames.length ? `共 ${userGames.length} 款 · 由小伴创作或你放进游戏文件夹` : "还没有自创游戏" }}</p>
        </div>
        <NButton round data-testid="games-folder" @click="openFolder"><template #icon><FolderOpen :size="15" /></template>打开游戏文件夹</NButton>
      </div>
      <NAlert v-if="folderErrors.length" class="catalog-error" type="warning" :show-icon="false" data-testid="user-game-errors">
        有 {{ folderErrors.length }} 个游戏文件夹无法使用：{{ folderErrors.join("；") }}
      </NAlert>
      <div v-if="userGames.length" class="game-grid">
        <div v-for="game in userGames" :key="game.id" class="game-card-wrapper">
          <article class="surface game-card" role="button" tabindex="0" :data-testid="`user-game-${game.id}`" :aria-label="`开始${game.name}`" :aria-disabled="opening" @click="openGameSpace(game.id)" @keydown.enter.prevent="openGameSpace(game.id)" @keydown.space.prevent="openGameSpace(game.id)">
            <div class="game-cover">
              <img :src="coverUrl(game)" :alt="`${game.name}封面`" loading="lazy" />
              <NTag class="game-kind" :bordered="false">自创</NTag>
            </div>
            <div class="game-content">
              <h3>{{ game.name }}</h3>
              <p class="muted">{{ game.description }}</p>
              <div v-if="game.save" class="game-footnote"><Save :size="14" /> 进度保存在本机</div>
            </div>
          </article>
          <NPopconfirm positive-text="删除" negative-text="取消" @positive-click="removeGame(game)">
            <template #trigger>
              <button class="game-delete-btn" title="删除这个自创游戏" :aria-label="`删除游戏 ${game.name}`" :data-testid="`delete-game-${game.id}`" @click.stop>
                <Trash2 :size="13" />
              </button>
            </template>
            确定要删除自创游戏「{{ game.name }}」吗？游戏文件夹和它的存档都会被清除。
          </NPopconfirm>
        </div>
      </div>
      <p v-else class="muted">想要自己的游戏？在聊天里让小伴为你写一个，或者点“打开游戏文件夹”，把做好的游戏文件夹（里面有 manifest.json 和 index.html）放进去，再点“重新载入”。</p>
    </section>
  </main>
</template>

<style scoped>
.games-page { display: flex; flex-direction: column; gap:var(--ui-section-gap); color: var(--ui-text-primary); }
.page-header { display: flex; align-items: center; justify-content: space-between; gap: 24px; margin-bottom: 5px; }
.page-header h1 { margin: 7px 0 4px; font-size: 24px; letter-spacing: -.035em; }
.eyebrow { display: flex; align-items: center; gap: 8px; color: var(--ui-primary-text); font-size: 13px; font-weight: 650; }
.page-subtitle, .muted { margin: 0; color: var(--ui-text-secondary); font-size:14px;line-height:1.65; }
.browser-note, .catalog-error { border-radius: 12px; }
.games-section { display: flex; flex-direction: column; gap: 14px; }
.section-heading { display: flex; align-items: center; justify-content: space-between; }
.section-heading .muted { margin-top: 3px; }
.heading-actions { display: flex; align-items: center; gap: 10px; }
.game-card-wrapper { position: relative; min-width: 0; }
.game-delete-btn { position: absolute; top: 8px; right: 8px; z-index: 3; width: 26px; height: 26px; border-radius: 50%; border: 0; background: rgba(0,0,0,.5); color: #fff; display: grid; place-items: center; cursor: pointer; opacity: .7; transition: all .2s; }
.game-delete-btn:hover, .game-delete-btn:focus-visible { opacity: 1; background: #dc2626; }
.game-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap:var(--ui-card-gap); }
.game-card{cursor:pointer;transition:transform 200ms cubic-bezier(.2,.8,.2,1),box-shadow 200ms}.game-card:hover{transform:translateY(-3px);box-shadow:0 14px 30px var(--ui-shadow-hover)}.game-card:focus-visible{outline:2px solid var(--ui-primary);outline-offset:2px}
.game-card, .game-card-skeleton { min-width: 0; overflow: hidden; border: 0; border-radius: var(--ui-radius-card); background: var(--ui-surface); box-shadow:none; }
.game-cover { position: relative; height: 145px; overflow: hidden; background: linear-gradient(135deg, var(--ui-primary-soft), var(--ui-border)); }
.game-cover img { display: block; width: 100%; height: 100%; object-fit: contain; }
.game-kind { position: absolute; right: 10px; bottom: 10px; background: color-mix(in srgb, var(--ui-surface) 92%, transparent); color: var(--ui-text-secondary); }
.game-content { padding: 13px 15px 15px; }
.game-content h3 { margin: 0 0 5px; color: var(--ui-text-primary); font-size: 14px; font-weight: 600; }
.game-content .muted { min-height: 42px; }
.game-footnote { display: flex; align-items: center; gap: 6px; margin-top: 9px; color: var(--ui-primary-text); font-size:12px; }
.game-card-skeleton { display: flex; flex-direction: column; gap: 12px; padding-bottom: 14px; }
.game-card-skeleton :deep(.n-skeleton) { margin: 0 13px; }
.game-card-skeleton :deep(.n-skeleton:first-child) { margin: 0; }
@media (max-width: 900px) { .game-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
@media (max-width: 580px) { .game-grid { grid-template-columns: 1fr; } .game-cover { height: 175px; } }
</style>
