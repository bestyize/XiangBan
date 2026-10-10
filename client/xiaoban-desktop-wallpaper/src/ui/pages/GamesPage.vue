<script setup lang="ts">
import { onMounted, ref } from "vue";
import { Save } from "lucide-vue-next";
import { NAlert, NSkeleton, NTag } from "naive-ui";
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
const opening = ref(false);

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

    <section class="games-section" aria-labelledby="games-heading">
      <div class="section-heading">
        <div>
          <h2 id="games-heading">本地游戏</h2>
          <p class="muted">共 {{ games.length }} 款 · 点击开始游戏</p>
        </div>
        <NTag v-if="!loading" :bordered="false" type="info">离线可玩</NTag>
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
        <article v-for="game in games" :key="game.id" class="surface game-card" role="button" tabindex="0" :aria-label="`开始${game.name}`" :aria-disabled="opening || !store.desktop" @click="openGameSpace(game.id)" @keydown.enter.prevent="openGameSpace(game.id)" @keydown.space.prevent="openGameSpace(game.id)">
          <div class="game-cover">
            <img :src="`/game-packs/${game.id}/${game.cover}`" :alt="`${game.name}封面`" loading="lazy" />
            <NTag class="game-kind" :bordered="false">{{ game.orientation === "landscape" ? "横屏" : "轻松益智" }}</NTag>
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
.games-page { display: flex; flex-direction: column; gap:var(--ui-section-gap); color: var(--ui-text-primary); }
.page-header { display: flex; align-items: center; justify-content: space-between; gap: 24px; margin-bottom: 5px; }
.page-header h1 { margin: 7px 0 4px; font-size: 24px; letter-spacing: -.035em; }
.eyebrow { display: flex; align-items: center; gap: 8px; color: var(--ui-primary-text); font-size: 13px; font-weight: 650; }
.page-subtitle, .muted { margin: 0; color: var(--ui-text-secondary); font-size:14px;line-height:1.65; }
.browser-note, .catalog-error { border-radius: 12px; }
.games-section { display: flex; flex-direction: column; gap: 14px; }
.section-heading { display: flex; align-items: center; justify-content: space-between; }
.section-heading .muted { margin-top: 3px; }
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
