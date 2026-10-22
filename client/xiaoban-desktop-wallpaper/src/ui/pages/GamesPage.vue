<script setup lang="ts">
import ResourceShareActions from "../components/ResourceShareActions.vue";
import { computed, onActivated, onMounted, ref } from "vue";
import { Save, Sparkles, Square, Trash2 } from "lucide-vue-next";
import { NAlert, NPopconfirm, NSkeleton } from "naive-ui";
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
const tabs: { id: 'builtin' | 'custom'; label: string }[] = [{ id: 'builtin', label: '官方游戏' }, { id: 'custom', label: '我的游戏' }];

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
  <main class="xb-page games-page" data-testid="games-page" aria-label="游玩">
    <header class="xb-page-head games-head">
      <div class="xb-tabs-lg" role="tablist" aria-label="游戏分类">
        <button v-for="tab in tabs" :key="tab.id" type="button" role="tab" :aria-selected="category === tab.id" :data-testid="`games-tab-${tab.id}`" @click="category = tab.id">{{ tab.label }}</button>
      </div>
      <div class="xb-page-actions">
        <ResourceShareActions :store="store" kind="game" variant="xb" import-label="导入" :disabled="opening" @imported="imported"/>
        <button type="button" class="xb-btn-primary" :disabled="!store.desktop" data-testid="games-create" @click="store.startCreation('game')"><Sparkles :size="14"/>让小伴创作</button>
      </div>
    </header>

    <div class="games-body xb-scroll">
      <div v-if="store.game.desk?.active" class="game-running-actions"><button type="button" class="xb-btn" data-testid="games-stop-desk" @click="stopDeskGame"><Square :size="13"/>停止桌面游戏</button></div>
      <section class="games-section" :aria-label="category==='builtin'?'官方游戏':'我的游戏'" :data-testid="category==='custom'?'user-games':'official-games'">
        <NAlert v-if="catalogError" class="catalog-error" type="warning" :bordered="false">{{ catalogError }}<button type="button" class="xb-link" @click="loadGames">重新载入</button></NAlert>
        <NAlert v-if="category==='custom'&&folderErrors.length" class="catalog-error" type="warning" :show-icon="false" data-testid="user-game-errors">有 {{ folderErrors.length }} 个游戏文件夹无法使用：{{ folderErrors.join('；') }}</NAlert>
        <div v-if="loading" class="game-grid" aria-label="正在读取游戏资料">
          <article v-for="item in 6" :key="item" class="game-card-skeleton"><NSkeleton height="142px" :sharp="false"/><NSkeleton text :repeat="2"/></article>
        </div>
        <div v-else-if="visibleGames.length" class="game-grid">
          <div v-for="game in visibleGames" :key="game.id" class="game-card-wrapper">
            <article class="game-card" role="button" tabindex="0" :data-testid="game.source==='user'?`user-game-${game.id}`:undefined" :aria-label="`开始${game.name}`" :aria-disabled="opening||!store.desktop" @click="openGameSpace(game.id)" @keydown.enter.prevent="openGameSpace(game.id)" @keydown.space.prevent="openGameSpace(game.id)">
              <div class="game-cover">
                <img :src="coverUrl(game)" :alt="`${game.name}封面`" loading="lazy"/>
                <span class="game-kind">{{ game.source==='user'?(game.layer==='desktop'?'桌面层':'自定义'):(game.orientation==='landscape'?'横屏':game.orientation==='portrait'?'竖屏':'轻松益智') }}</span>
              </div>
              <div class="game-content">
                <h3>{{ game.name }}</h3><p>{{ descriptionFor(game) }}</p>
                <div v-if="game.source==='user'&&game.layer==='desktop'" class="game-footnote">浮在桌面上运行 · 托盘菜单可随时停止</div>
                <div v-if="game.save" class="game-footnote"><Save :size="13"/> 进度保存在本机</div>
              </div>
            </article>
            <ResourceShareActions v-if="game.source==='user'" class="game-share" :store="store" kind="game" :id="game.id" :disabled="opening"/>
            <NPopconfirm v-if="game.source==='user'" positive-text="删除" negative-text="取消" @positive-click="removeGame(game)">
              <template #trigger><button type="button" class="game-delete-btn" title="删除这个自创游戏" :aria-label="`删除游戏 ${game.name}`" :data-testid="`delete-game-${game.id}`" @click.stop><Trash2 :size="13"/></button></template>
              确定要删除自创游戏「{{ game.name }}」吗？游戏文件夹和它的存档都会被清除。
            </NPopconfirm>
          </div>
        </div>
        <div v-else-if="!catalogError" class="xb-empty games-empty">
          <span class="xb-empty-icon"><Sparkles :size="24"/></span>
          <p>{{ category==='custom'?'还没有自定义游戏，可从右上角导入或让小伴创作。':'暂时没有可展示的游戏资料，请稍后重试。' }}</p>
        </div>
      </section>
    </div>
  </main>
</template>

<style scoped>
/* Local tokens for the card edge and the orientation badge; they follow light and dark mode. */
.games-page{--game-edge:0 0 0 1px var(--ui-border);flex:1;min-height:0;min-width:0;display:flex;flex-direction:column;gap:18px;padding:22px 28px 0}
.games-head{flex-shrink:0}
.games-body{flex:1;min-height:0;display:flex;flex-direction:column;gap:16px;margin:0 -4px;padding:4px 4px 28px}
.game-running-actions{display:flex;justify-content:flex-end}
.games-section{display:flex;flex-direction:column;gap:14px}
.catalog-error{border-radius:12px}
.game-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:18px}
.game-card-wrapper{position:relative;min-width:0;display:flex;flex-direction:column;gap:6px}
.game-card{flex:1;min-width:0;display:flex;flex-direction:column;gap:0;padding:8px 8px 4px;border-radius:16px;background:var(--ui-surface);box-shadow:var(--game-edge),0 1px 2px rgba(0,0,0,.04),0 3px 10px rgba(0,0,0,.03);cursor:pointer;transition:transform 200ms cubic-bezier(.2,.8,.2,1),box-shadow 200ms}
.game-card:hover{transform:translateY(-3px);box-shadow:0 0 0 1px var(--ui-border-strong),0 2px 4px rgba(0,0,0,.05),0 14px 28px var(--ui-shadow-hover)}
.game-card:focus-visible{outline:2px solid var(--ui-btn);outline-offset:2px}
.game-cover{position:relative;height:140px;overflow:hidden;border-radius:12px;background:linear-gradient(135deg,var(--ui-soft),var(--ui-control))}
.game-cover img{display:block;width:100%;height:100%;object-fit:contain}
.game-kind{position:absolute;right:8px;bottom:8px;height:20px;padding:0 7px;display:inline-flex;align-items:center;border-radius:6px;background:color-mix(in srgb,var(--ui-surface) 90%,transparent);color:var(--ui-text-secondary);font-size:11px;font-weight:500;backdrop-filter:blur(8px)}
.game-content{padding:10px 6px 8px}
.game-content h3{margin:0 0 4px;color:var(--ui-text-primary);font-size:14px;font-weight:600}
.game-content p{margin:0;min-height:40px;color:var(--ui-text-secondary);font-size:12.5px;line-height:1.6}
.game-footnote{display:flex;align-items:center;gap:6px;margin-top:6px;color:var(--ui-ink);font-size:12px}
.game-share{padding:0 2px}
.game-delete-btn{position:absolute;top:14px;right:14px;z-index:3;width:24px;height:24px;border-radius:50%;border:0;background:rgba(0,0,0,.45);color:#fff;display:grid;place-items:center;cursor:pointer;opacity:.7;transition:all .2s}
.game-delete-btn:hover,.game-delete-btn:focus-visible{opacity:1;background:var(--ui-danger)}
.game-card-skeleton{display:flex;flex-direction:column;gap:12px;padding:8px 8px 14px;border-radius:16px;background:var(--ui-surface);box-shadow:var(--game-edge)}
.game-card-skeleton :deep(.n-skeleton){border-radius:12px}
.games-empty{padding:56px 24px}
@media (max-width:900px){.game-grid{grid-template-columns:repeat(2,minmax(0,1fr))}}
@media (max-width:580px){.game-grid{grid-template-columns:1fr}.game-cover{height:175px}}
</style>
