<script setup lang="ts">
import ResourceLibraryPage from '../components/ResourceLibraryPage.vue';
import ResourceShareActions from "../components/ResourceShareActions.vue";
import { computed, onActivated, onMounted, ref } from "vue";
import { Save, Sparkles, Square, Trash2 } from "lucide-vue-next";
import { NAlert, NButton, NPopconfirm, NSkeleton, NTag, NTabs, NTabPane } from "naive-ui";
import type { ControlStore } from "../control-types.ts";
import { coverUrl } from "../../games/catalog.ts";
import type { GameManifest } from "../../games/protocol.ts";

defineOptions({name:'GamesPage'});

const props = defineProps<{ store: ControlStore }>();
const category=ref<'builtin'|'custom'>('builtin');
const games = ref<GameManifest[]>([]);
const loading = ref(true);
const catalogError = ref("");
const folderErrors = ref<string[]>([]);
const opening = ref(false);
const builtinGames = computed(() => games.value.filter((game) => game.source !== "user"));
const userGames = computed(() => games.value.filter((game) => game.source === "user"));
const visibleGames=computed(()=>category.value==='builtin'?builtinGames.value:userGames.value);

function descriptionFor(game: GameManifest) {
  if (game.id === "snake-arena") return "无尽成长或三分钟冲分，吃糖果长成大蛇，反击 AI。";
  return game.description;
}
const message = (error: unknown, fallback: string) => (error instanceof Error ? error.message : typeof error === "string" ? error : fallback);

async function loadGames() {
  loading.value = games.value.length === 0;
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

async function stopDeskGame() {
  try {
    await props.store.stopDeskGame();
  } catch (error) {
    props.store.notify(message(error, "桌面游戏暂时无法停止。"), "error");
  }
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

async function imported(){await loadGames();category.value='custom';}
onMounted(loadGames);
let opened = false;
onActivated(() => { if (opened) void loadGames(); else opened = true; });
</script>

<template>
  <ResourceLibraryPage title="游戏空间" :active-tab="category" class="games-page" data-testid="games-page">
    <template #actions><ResourceShareActions :store="store" kind="game" import-label="导入游戏" :disabled="opening" @imported="imported"/><NButton type="primary" round :disabled="!store.desktop" data-testid="games-create" @click="store.startCreation('game')"><template #icon><Sparkles :size="15"/></template>创作游戏</NButton></template>
    <template #tabs><NTabs v-model:value="category" type="line" size="large"><NTabPane name="builtin"><template #tab>官方游戏</template></NTabPane><NTabPane name="custom"><template #tab>自定义游戏</template></NTabPane></NTabs></template>
    <div v-if="store.game.desk?.active" class="game-running-actions"><NButton round data-testid="games-stop-desk" @click="stopDeskGame"><template #icon><Square :size="15"/></template>停止桌面游戏</NButton></div>
    <section class="games-section" :aria-label="category==='builtin'?'官方游戏':'自定义游戏'" :data-testid="category==='custom'?'user-games':'official-games'">
      <NAlert v-if="catalogError" class="catalog-error" type="warning" :bordered="false">{{ catalogError }}<NButton text @click="loadGames">重新载入</NButton></NAlert>
      <NAlert v-if="category==='custom'&&folderErrors.length" class="catalog-error" type="warning" :show-icon="false" data-testid="user-game-errors">有 {{ folderErrors.length }} 个游戏文件夹无法使用：{{ folderErrors.join('；') }}</NAlert>
      <div v-if="loading" class="game-grid" aria-label="正在读取游戏资料">
        <article v-for="item in 6" :key="item" class="surface game-card-skeleton"><NSkeleton height="142px"/><NSkeleton text :repeat="2"/></article>
      </div>
      <div v-else-if="visibleGames.length" class="game-grid">
        <div v-for="game in visibleGames" :key="game.id" class="game-card-wrapper">
          <article class="surface game-card" role="button" tabindex="0" :data-testid="game.source==='user'?`user-game-${game.id}`:undefined" :aria-label="`开始${game.name}`" :aria-disabled="opening||!store.desktop" @click="openGameSpace(game.id)" @keydown.enter.prevent="openGameSpace(game.id)" @keydown.space.prevent="openGameSpace(game.id)">
            <div class="game-cover">
              <img :src="coverUrl(game)" :alt="`${game.name}封面`" loading="lazy"/>
              <NTag class="game-kind" :bordered="false">{{ game.source==='user'?(game.layer==='desktop'?'桌面层':'自定义'):(game.orientation==='landscape'?'横屏':game.orientation==='portrait'?'竖屏':'轻松益智') }}</NTag>
            </div>
            <div class="game-content">
              <h3>{{ game.name }}</h3><p class="muted">{{ descriptionFor(game) }}</p>
              <div v-if="game.source==='user'&&game.layer==='desktop'" class="game-footnote">浮在桌面上运行 · 托盘菜单可随时停止</div>
              <div v-if="game.save" class="game-footnote"><Save :size="14"/> 进度保存在本机</div>
            </div>
          </article>
          <ResourceShareActions v-if="game.source==='user'" :store="store" kind="game" :id="game.id" :disabled="opening"/>
          <NPopconfirm v-if="game.source==='user'" positive-text="删除" negative-text="取消" @positive-click="removeGame(game)">
            <template #trigger><button class="game-delete-btn" title="删除这个自创游戏" :aria-label="`删除游戏 ${game.name}`" :data-testid="`delete-game-${game.id}`" @click.stop><Trash2 :size="13"/></button></template>
            确定要删除自创游戏「{{ game.name }}」吗？游戏文件夹和它的存档都会被清除。
          </NPopconfirm>
        </div>
      </div>
      <NAlert v-else-if="!catalogError" :type="category==='custom'?'info':'warning'" :bordered="false">{{ category==='custom'?'还没有自定义游戏，可从右上角导入或创作。':'暂时没有可展示的游戏资料，请稍后重试。' }}</NAlert>
    </section>
  </ResourceLibraryPage>
</template>

<style scoped>
.game-running-actions{display:flex;justify-content:flex-end}
.muted { margin: 0; color: var(--ui-text-secondary); font-size:14px;line-height:1.65; }
.catalog-error { border-radius: 12px; }
.games-section { display: flex; flex-direction: column; gap: 14px; }
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

