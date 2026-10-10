<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { MousePointer2, Save } from "lucide-vue-next";
import { NAlert, NButton, NSkeleton, NSwitch, NTag } from "naive-ui";
import type { ControlStore } from "../control-types.ts";

interface GameManifest {
  id: string;
  version: string;
  name: string;
  description: string;
  cover: string;
  save?: boolean;
  orientation?: "landscape" | "portrait";
}

const props = defineProps<{ store: ControlStore }>();
const games = ref<GameManifest[]>([]);
const loading = ref(true);
const catalogError = ref("");
const busy = ref(false);
const toggleError = ref("");
const opening = ref(false);

const doubleClickLabel = computed(() => {
  if (!props.store.desktop) return "桌面应用中可用";
  if (!props.store.game.desktopDoubleClickSupported) return "当前平台不可用";
  return props.store.settings.gameDesktopDoubleClick ? "已开启" : "未开启";
});

const doubleClickDescription = computed(() => {
  if (!props.store.desktop) return "请在小伴桌面应用中使用此入口。浏览器预览不支持桌面空白处双击。";
  if (!props.store.game.desktopDoubleClickSupported) return "当前桌面平台无法可靠识别桌面空白处的双击，请双击游戏卡片进入游戏。";
  return "在桌面空白处双击即可打开游戏空间。此快捷入口默认关闭，可随时在这里调整。";
});

function descriptionFor(game: GameManifest) {
  if (game.id === "snake-arena") return "无尽成长或三分钟冲分，吃糖果长成大蛇，反击 AI。";
  return game.description;
}

async function loadGames() {
  loading.value = true;
  catalogError.value = "";
  try {
    const catalogResponse = await fetch("/game-packs/catalog.json");
    if (!catalogResponse.ok) throw new Error("游戏目录暂时无法读取。");
    const catalog = (await catalogResponse.json()) as { games?: unknown };
    if (!Array.isArray(catalog.games)) throw new Error("游戏目录格式暂时无法读取。");
    const results = await Promise.all(
      catalog.games.map(async (id) => {
        if (typeof id !== "string" || !/^[a-z0-9-]+$/.test(id)) return null;
        const response = await fetch(`/game-packs/${id}/manifest.json`);
        if (!response.ok) return null;
        const manifest = (await response.json()) as GameManifest;
        if (manifest.id !== id || !manifest.name || !manifest.description || !manifest.cover) return null;
        return manifest;
      }),
    );
    games.value = results.filter((game): game is GameManifest => game !== null);
    if (games.value.length !== catalog.games.length) catalogError.value = "部分游戏资料暂时无法显示。";
  } catch (error) {
    catalogError.value = error instanceof Error ? error.message : "游戏资料暂时无法读取。";
  } finally {
    loading.value = false;
  }
}

async function setShortcut(enabled: boolean) {
  if (!props.store.desktop || !props.store.game.desktopDoubleClickSupported || busy.value) return;
  busy.value = true;
  toggleError.value = "";
  try {
    await props.store.setDoubleClick(enabled);
  } catch (error) {
    // A native failure must leave the displayed value tied to persisted store state.
    toggleError.value = error instanceof Error ? error.message : "设置没有保存，请稍后重试。";
    try { await props.store.refreshSettings(); } catch { /* Keep the last known setting. */ }
    props.store.notify(toggleError.value, "error");
  } finally {
    busy.value = false;
  }
}

async function openGameSpace(gameId?: string) {
  if (!props.store.desktop || opening.value) return;
  opening.value = true;
  try {
    await props.store.openGames(gameId);
  } catch (error) {
    props.store.notify(error instanceof Error ? error.message : "游戏空间暂时无法打开。", "error");
  } finally {
    opening.value = false;
  }
}

onMounted(loadGames);
</script>

<template>
  <main class="ui-page games-page" data-testid="games-page">
    <NAlert v-if="!store.desktop" class="browser-note" type="info" :bordered="false">
      浏览器预览只展示游戏资料；请在小伴桌面应用中进入游戏空间。
    </NAlert>

    <section class="surface shortcut-card" aria-labelledby="shortcut-title">
      <div class="shortcut-icon"><MousePointer2 :size="20" /></div>
      <div class="shortcut-copy">
        <div class="shortcut-heading">
          <h2 id="shortcut-title">桌面双击快捷入口</h2>
          <NTag :type="!store.desktop || !store.game.desktopDoubleClickSupported ? 'default' : store.settings.gameDesktopDoubleClick ? 'success' : 'info'" round>
            {{ doubleClickLabel }}
          </NTag>
        </div>
        <p class="muted">{{ doubleClickDescription }}</p>
        <NAlert v-if="store.game.desktopEntryError" class="native-error" type="warning" :bordered="false">
          {{ store.game.desktopEntryError }}
        </NAlert>
        <NAlert v-if="toggleError" class="native-error" type="error" :bordered="false">
          {{ toggleError }} 当前设置已恢复为上次保存的状态。
        </NAlert>
      </div>
      <div class="shortcut-control">
        <span class="switch-caption">{{ store.settings.gameDesktopDoubleClick ? "开启" : "关闭" }}</span>
        <NSwitch
          :value="store.settings.gameDesktopDoubleClick"
          :loading="busy"
          :disabled="!store.desktop || !store.game.desktopDoubleClickSupported || busy"
          aria-label="桌面双击快捷入口"
          @update:value="setShortcut"
        />
      </div>
    </section>

    <section class="games-section" aria-labelledby="games-heading">
      <div class="section-heading">
        <div>
          <h2 id="games-heading">本地游戏</h2>
          <p class="muted">共 {{ games.length }} 款 · 双击开始游戏</p>
        </div>
        <NTag v-if="!loading" round :bordered="false" type="info">离线可玩</NTag>
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
      <div v-else-if="games.length" class="game-grid">
        <article v-for="game in games" :key="game.id" class="surface game-card" role="button" tabindex="0" :aria-label="`开始${game.name}`" :aria-disabled="opening || !store.desktop" @dblclick="openGameSpace(game.id)" @keydown.enter.prevent="openGameSpace(game.id)" @keydown.space.prevent="openGameSpace(game.id)">
          <div class="game-cover">
            <img :src="`/game-packs/${game.id}/${game.cover}`" :alt="`${game.name}封面`" loading="lazy" />
            <NTag class="game-kind" round :bordered="false">{{ game.orientation === "landscape" ? "横屏" : "轻松益智" }}</NTag>
          </div>
          <div class="game-content">
            <h3>{{ game.name }}</h3>
            <p class="muted">{{ descriptionFor(game) }}</p>
            <div v-if="game.save" class="game-footnote"><Save :size="14" /> 进度保存在本机</div>
          </div>
        </article>
      </div>
      <NAlert v-else type="warning" :bordered="false">暂时没有可展示的游戏资料，请稍后重试。</NAlert>
    </section>
  </main>
</template>

<style scoped>
.games-page { display: flex; flex-direction: column; gap: 22px; color: #26364a; }
.page-header { display: flex; align-items: center; justify-content: space-between; gap: 24px; margin-bottom: 5px; }
.page-header h1 { margin: 7px 0 4px; font-size: 27px; letter-spacing: -.035em; }
.eyebrow { display: flex; align-items: center; gap: 8px; color: #3976c4; font-size: 13px; font-weight: 650; }
.page-subtitle, .muted { margin: 0; color: #78879a; font-size: 13px; line-height: 1.65; }
.shortcut-card { display: flex; align-items: flex-start; gap: 15px; padding: 19px 20px; border: 1px solid #e8eef6; border-radius: 15px; background: #fff; box-shadow: 0 5px 20px #27476a0a; }
.shortcut-icon { display: grid; width: 40px; height: 40px; flex: 0 0 auto; place-items: center; border-radius: 12px; color: #3976c4; background: #eff6ff; }
.shortcut-copy { flex: 1; min-width: 0; }
.shortcut-heading { display: flex; align-items: center; gap: 10px; margin: 1px 0 5px; }
.shortcut-heading h2, .section-heading h2 { margin: 0; font-size: 15px; font-weight: 680; }
.shortcut-control { display: flex; align-items: center; gap: 10px; padding: 5px 0 0 8px; }
.switch-caption { color: #63758b; font-size: 12px; }
.native-error { margin-top: 10px; }
.browser-note, .catalog-error { border-radius: 12px; }
.games-section { display: flex; flex-direction: column; gap: 14px; }
.section-heading { display: flex; align-items: center; justify-content: space-between; }
.section-heading .muted { margin-top: 3px; }
.game-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 15px; }
.game-card{cursor:pointer}.game-card:focus-visible{outline:2px solid #2563eb;outline-offset:2px}
.game-card, .game-card-skeleton { min-width: 0; overflow: hidden; border: 1px solid #e8eef6; border-radius: 14px; background: #fff; box-shadow: 0 4px 16px #27476a08; }
.game-cover { position: relative; height: 145px; overflow: hidden; background: linear-gradient(135deg, #eef5ff, #f8fbff); }
.game-cover img { display: block; width: 100%; height: 100%; object-fit: contain; }
.game-kind { position: absolute; right: 10px; bottom: 10px; background: #ffffffeb; color: #526980; }
.game-content { padding: 13px 15px 15px; }
.game-content h3 { margin: 0 0 5px; color: #2a3a50; font-size: 15px; font-weight: 680; }
.game-content .muted { min-height: 42px; }
.game-footnote { display: flex; align-items: center; gap: 6px; margin-top: 9px; color: #5581b2; font-size: 11px; }
.game-card-skeleton { display: flex; flex-direction: column; gap: 12px; padding-bottom: 14px; }
.game-card-skeleton :deep(.n-skeleton) { margin: 0 13px; }
.game-card-skeleton :deep(.n-skeleton:first-child) { margin: 0; }
@media (max-width: 900px) { .game-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
@media (max-width: 580px) { .page-header { align-items: flex-start; flex-direction: column; gap: 15px; } .shortcut-card { flex-wrap: wrap; padding: 15px; } .shortcut-copy { flex-basis: calc(100% - 55px); } .shortcut-control { width: 100%; justify-content: flex-end; } .game-grid { grid-template-columns: 1fr; } .game-cover { height: 175px; } }
</style>
