<script setup lang="ts">
import { computed, onBeforeUnmount, onDeactivated, onMounted, ref, watch } from 'vue';
import { NAlert, NPopconfirm, NSwitch } from 'naive-ui';
import { Download, EyeOff, Monitor, MoreHorizontal, Pin, Plus, Settings2, Sparkles, Trash2 } from 'lucide-vue-next';
import type { ControlStore } from '../control-types.ts';
import { EXPRESSIONS, stillFrame, type ExpressionName } from '../../companion/expressions.ts';
import StudioActions from '../components/StudioActions.vue';
import PetSettingsDialog, { type PetSettingsPatch } from '../components/PetSettingsDialog.vue';
import PetProactivityPanel, { type EnvironmentContext, type GreetingSettings } from '../components/PetProactivityPanel.vue';
import ResourceShareActions from '../components/ResourceShareActions.vue';
import PetActor from '../../pets/PetActor.vue';
import { createPetClient } from '../../pets/client.ts';
import { definitionOf, type PetDefinition, type PetInstance, type PetTheme } from '../../pets/model.ts';
import { petMarkup } from '../../pets/render.ts';
import { desktop as nativeDesktop, invoke, listen } from '../../bridge.ts';
import { openPetSettingsWindow, openProactivityWindow } from '../configuration/windows.ts';

const imageCache = new WeakMap<PetDefinition, Map<ExpressionName, string>>();
function petImage(pet: PetDefinition, expression: ExpressionName = 'happy') {
  let byExpr = imageCache.get(pet);
  if (!byExpr) { byExpr = new Map(); imageCache.set(pet, byExpr); }
  let image = byExpr.get(expression);
  if (!image) {
    image = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(
      petMarkup(pet, stillFrame(expression), 'gallery', expression).replace('viewBox="0 0 320 280"', 'viewBox="0 -72 320 352"'),
    )}`;
    byExpr.set(expression, image);
  }
  return image;
}

const props = defineProps<{ store: ControlStore; settingsRequest?: { id: string } | null }>();
const emit = defineEmits<{ settingsOpened: [] }>();

const client = createPetClient();
const { catalog, ready, busy, error } = client;

const filter = ref<'all' | 'desk' | 'mine'>('all');
const hover = ref('');
const editing = ref<string | null>(null);
const proactivityOpen = ref(false);

type MenuKind = 'theme' | 'add' | 'manage' | 'deskMore';
const openMenu = ref<{ type: MenuKind; key: string; up: boolean } | null>(null);
const isMenuOpen = (type: MenuKind, key: string) => openMenu.value?.type === type && openMenu.value.key === key;
function closeMenus() { openMenu.value = null; }
function toggleMenu(event: Event, type: MenuKind, key: string) {
  event.stopPropagation();
  if (isMenuOpen(type, key)) { closeMenus(); return; }
  const target = event.currentTarget as HTMLElement | null;
  const rect = target?.getBoundingClientRect?.();
  const up = rect ? (window.innerHeight - rect.bottom < 190) : false;
  openMenu.value = { type, key, up };
}
function onKey(e: KeyboardEvent) { if (e.key === 'Escape') closeMenus(); }

const themes = computed(() => filter.value === 'mine' ? catalog.value.themes.filter(t => t.source === 'custom') : catalog.value.themes);
const instances = computed(() => catalog.value.layout.instances);
const visibleCount = computed(() => instances.value.filter(p => p.enabled).length);
const current = computed(() => instances.value.find(p => p.id === editing.value));
const currentTheme = computed(() => catalog.value.themes.find(t => t.id === current.value?.themeId));
const onDeskOf = (themeId: string, petId: string) => instances.value.filter(p => p.themeId === themeId && p.petId === petId);
const galleryMood = (themeId: string, pet: PetDefinition) =>
  onDeskOf(themeId, pet.id)[0]?.expression ?? (pet.poses ? 'idle' : 'happy');

const deskGroups = computed(() => {
  const groups = catalog.value.themes
    .map(theme => ({ theme, items: instances.value.filter(p => p.themeId === theme.id) }))
    .filter(group => group.items.length);
  const orphans = instances.value.filter(p => !catalog.value.themes.some(t => t.id === p.themeId));
  return orphans.length ? [...groups, { theme: undefined, items: orphans }] : groups;
});

const greetingSettings = ref<GreetingSettings>({ enabled: false, frequency: 'natural', contextEnabled: false, calendarRegion: null });
const greetingReady = ref(false);
const greetingBusy = ref(false);
const environment = ref<EnvironmentContext | null>(null);
const freqNames: Record<string, string> = { quiet: '安静', natural: '自然', lively: '活泼' };
const proactivityPillText = computed(() =>
  greetingSettings.value.enabled
    ? `主动陪伴 · ${freqNames[greetingSettings.value.frequency] ?? '自然'}`
    : '主动陪伴',
);

async function loadGreeting() {
  if (!props.store.desktop) return;
  try {
    const result = await invoke<GreetingSettings>('get_proactivity_settings');
    if (result) { greetingSettings.value = result; greetingReady.value = true; }
  } catch (e) { props.store.notify(e, 'error'); }
}
async function changeGreeting(patch: Partial<GreetingSettings>) {
  if (!greetingReady.value || greetingBusy.value) return;
  greetingBusy.value = true;
  try {
    greetingSettings.value = await invoke<GreetingSettings>('configure_proactivity', { settings: { ...greetingSettings.value, ...patch } });
    environment.value = null;
    props.store.notify('感知与陪伴设置已保存');
  } catch (e) { props.store.notify(e, 'error'); }
  finally { greetingBusy.value = false; }
}
async function inspectEnvironment() {
  try { environment.value = await invoke<EnvironmentContext>('get_proactivity_context'); }
  catch (e) { props.store.notify(e, 'error'); }
}

async function openProactivity() {
  closeMenus();
  if (nativeDesktop && props.store.desktop) {
    try { await openProactivityWindow(); return; } catch { /* Fall back to inline dialog when native window is unavailable. */ }
  }
  proactivityOpen.value = true;
}

async function openManage(id: string) {
  closeMenus();
  if (!instances.value.some(p => p.id === id)) {
    props.store.notify('这个伙伴已不在桌面上', 'info');
    return;
  }
  if (nativeDesktop && props.store.desktop) {
    try { await openPetSettingsWindow(id); return; } catch { /* Fall back to inline dialog when native window is unavailable. */ }
  }
  editing.value = id;
}

function managePet(event: Event, themeId: string, petId: string) {
  const items = onDeskOf(themeId, petId);
  if (items.length === 1) void openManage(items[0].id);
  else toggleMenu(event, 'manage', `${themeId}/${petId}`);
}

async function act(action: () => Promise<unknown>, success?: string) {
  try { await action(); if (success) props.store.notify(success); }
  catch (e) { props.store.notify(e, 'error'); }
}

const placements = [
  { value: 'topmost' as const, label: '始终置顶', hint: '显示在其他窗口上方，随时陪着你', badge: '推荐', icon: Pin },
  { value: 'desktop' as const, label: '贴在桌面', hint: '放在桌面图标下方，不遮挡其他窗口', badge: '', icon: Monitor },
];

async function add(themeId: string, petId: string, layer: PetInstance['layer']) {
  closeMenus();
  await act(async () => {
    await client.add(themeId, petId, layer);
    if (!props.store.settings.companionEnabled) await props.store.setCompanion(true);
  }, '已添加到桌面，可拖动位置，右键管理');
}

async function toggleAllInstances(show: boolean) {
  await act(async () => {
    for (const item of instances.value) {
      if (item.enabled !== show) await client.update(item.id, { enabled: show });
    }
  }, show ? '已显示全部桌面伙伴' : '已隐藏全部桌面伙伴');
}

function setInstanceLayer(item: PetInstance, layer: PetInstance['layer']) {
  if (item.layer === layer || busy.value) return;
  void act(() => client.update(item.id, { layer }), layer === 'topmost' ? '已切换为始终置顶' : '已切换为贴在桌面');
}

function centerInstance(id: string) {
  closeMenus();
  void act(() => client.update(id, { x: 0.5, y: 0.5 }), '已移到屏幕正中央');
}

function removeInstance(id: string) {
  closeMenus();
  void act(() => client.remove(id), '已从桌面移除');
}

function create(base?: PetTheme) {
  void props.store.startCreation('pet').then(() => {
    if (base) {
      props.store.creationDraft = `请以宠物主题「${base.name}」（${base.id}）为参考，创作我的自定义宠物主题。先读取 pet_read_format 和现有主题，按我的要求调整：`;
    }
  });
}

async function saveSettings(patch: PetSettingsPatch) {
  const id = current.value?.id;
  if (!id) return;
  await act(async () => {
    await client.update(id, patch);
    if (editing.value === id) editing.value = null;
  });
}
function removeCurrent() { const id = current.value?.id; editing.value = null; if (id) void act(() => client.remove(id)); }
function centerCurrent() { if (current.value) centerInstance(current.value.id); }
function createFromCurrent() { const theme = currentTheme.value; editing.value = null; create(theme); }

function instanceHint(item: PetInstance) {
  if (!item.enabled) return '已隐藏';
  const layer = item.layer === 'desktop' ? '贴桌面' : '置顶';
  return item.monitor ? `${layer} · ${item.monitor}` : `${layer} · ${item.size}px`;
}

watch(() => props.settingsRequest, async request => {
  if (!request) return;
  await client.load();
  if (props.settingsRequest !== request) return;
  await openManage(request.id);
  emit('settingsOpened');
}, { immediate: true });

function manage(e: Event) {
  const id = (e as CustomEvent<{ id: string }>).detail?.id;
  void client.load().then(() => { if (id) void openManage(id); });
}

let offProactivity: (() => void) | undefined;
onMounted(() => {
  void client.load();
  void loadGreeting();
  window.addEventListener('pets-manage', manage);
  window.addEventListener('keydown', onKey);
  if (nativeDesktop && props.store.desktop) {
    void listen<GreetingSettings | null>('proactivity-changed', value => {
      if (value) { greetingSettings.value = value; greetingReady.value = true; }
      else void loadGreeting();
    }).then(off => { offProactivity = off; });
  }
});
onDeactivated(() => { closeMenus(); hover.value = ''; editing.value = null; proactivityOpen.value = false; });
onBeforeUnmount(() => {
  offProactivity?.();
  client.dispose();
  window.removeEventListener('pets-manage', manage);
  window.removeEventListener('keydown', onKey);
});
</script>

<template>
  <section class="pets-page" data-testid="companion-page" @keydown.esc.capture="closeMenus()">
    <div v-if="openMenu" class="menu-dismiss" @click="closeMenus()" />

    <StudioActions>
      <ResourceShareActions :store="store" kind="pet" variant="xb" import-label="导入" :disabled="!ready || busy" @imported="async () => { await client.load(); filter = 'mine'; }" />
      <button type="button" class="xb-btn-primary" data-testid="pet-create" @click="create()"><Sparkles :size="14" />让小伴创作</button>
    </StudioActions>

    <div class="pt-toolbar">
      <div class="xb-seg" role="group" aria-label="筛选">
        <button type="button" :aria-pressed="filter === 'all'" data-testid="pet-filter-all" @click="closeMenus(); filter = 'all'">全部</button>
        <button type="button" :aria-pressed="filter === 'desk'" data-testid="pet-filter-desk" @click="closeMenus(); filter = 'desk'">在桌面上 <b class="cnt">{{ instances.length }}</b></button>
        <button type="button" :aria-pressed="filter === 'mine'" data-testid="pet-filter-mine" @click="closeMenus(); filter = 'mine'">我的</button>
      </div>
      <div class="pt-tools">
        <button v-if="store.desktop" type="button" class="proactivity-pill" :class="{ 'is-on': greetingSettings.enabled }" aria-haspopup="dialog" title="配置悬停问候与环境上下文" aria-description="配置悬停问候与环境上下文" data-testid="pet-proactivity-open" @click="openProactivity">
          <i class="p-dot" />{{ proactivityPillText }}
        </button>
        <span class="pt-label">桌面伙伴</span>
        <NSwitch :value="store.settings.companionEnabled" aria-label="显示全部桌面伙伴" data-testid="pet-master-switch" @update:value="value => act(() => store.setCompanion(value))" />
      </div>
    </div>

    <div class="pt-scroll">
      <div v-if="!store.settings.companionEnabled && instances.length" class="pt-banner" data-testid="pet-master-off-banner">
        <EyeOff :size="14" />
        <span>桌面伙伴总开关已关闭，当前 {{ instances.length }} 只伙伴均已暂时隐藏。</span>
        <button type="button" class="xb-link banner-act" @click="act(() => store.setCompanion(true))">一键显示</button>
      </div>

      <NAlert v-if="error" type="error" :show-icon="false" class="pt-alert">{{ error }}<button type="button" class="xb-link" @click="client.load()">重新载入</button></NAlert>

      <div v-if="ready && filter === 'mine' && !themes.length" class="xb-empty" data-testid="pet-custom-empty">
        <span class="xb-empty-icon"><Sparkles :size="24" /></span>
        <h3>你想象中的小伙伴，在这里诞生。</h3>
        <p>让 AI 创作你喜欢的小伙伴，也可以导入别人分享的 ZIP 宠物资源包。</p>
        <button type="button" class="xb-btn-primary" @click="create()">开始创作</button>
      </div>

      <!-- 全部 / 我的：纯净中性画框图鉴卡片 -->
      <template v-if="filter !== 'desk'">
        <section v-for="theme in themes" :key="theme.id" class="pet-theme-detail" :aria-label="theme.name" :data-testid="`pet-theme-${theme.id}`">
          <header class="th-head" :title="theme.description" :aria-description="theme.description">
            <h2>{{ theme.name }}</h2>
            <span class="sub">{{ theme.pets.length }} 款</span>
            <div class="theme-actions">
              <button type="button" class="xb-icon-btn is-sm" :aria-label="`${theme.name}更多操作`" :aria-expanded="isMenuOpen('theme', theme.id)" @click="toggleMenu($event, 'theme', theme.id)">
                <MoreHorizontal :size="17" />
              </button>
              <div v-if="isMenuOpen('theme', theme.id)" class="theme-menu xb-menu" role="menu">
                <button type="button" role="menuitem" class="xb-menu-item" @click="closeMenus(); create(theme)">基于此主题创作</button>
                <button v-if="theme.source === 'builtin'" type="button" role="menuitem" class="xb-menu-item" :disabled="busy" title="导出参考 JSON" data-testid="pet-export-reference" @click="closeMenus(); act(() => client.exportTheme(theme.id))">
                  <Download :size="14" />导出参考 JSON
                </button>
                <ResourceShareActions
                  v-if="theme.source === 'custom'"
                  :store="store"
                  kind="pet"
                  variant="menu"
                  :id="theme.id"
                  :disabled="busy"
                  :title="`导出或分享整套主题（含 ${theme.pets.length} 款形象），不包含桌面位置、昵称或心情`"
                />
                <template v-if="theme.source === 'custom'">
                  <div class="xb-menu-sep" />
                  <NPopconfirm @positive-click="closeMenus(); act(() => client.deleteTheme(theme.id))">
                    <template #trigger>
                      <button type="button" role="menuitem" class="xb-menu-item is-danger" :disabled="busy" title="删除主题" aria-label="删除主题">
                        <Trash2 :size="14" />删除主题
                      </button>
                    </template>
                    删除这个自定义主题？已添加的宠物需先移除。
                  </NPopconfirm>
                </template>
              </div>
            </div>
          </header>

          <div class="pet-cards">
            <article
              v-for="p in theme.pets"
              :key="p.id"
              class="pc pet-card"
              :class="{ 'is-menu-open': isMenuOpen('add', `${theme.id}/${p.id}`) || isMenuOpen('manage', `${theme.id}/${p.id}`) }"
              :data-testid="`pet-card-${p.id}`"
              @pointerenter="hover = `${theme.id}/${p.id}`"
              @pointerleave="hover = ''"
            >
              <div class="card-stage" :title="p.description" :aria-description="p.description">
                <button
                  v-if="onDeskOf(theme.id, p.id).length"
                  type="button"
                  class="stage-pill right is-clickable"
                  :class="{ 'is-muted': !store.settings.companionEnabled }"
                  :title="`桌面上有 ${onDeskOf(theme.id, p.id).length} 只${p.name}`"
                  @click="managePet($event, theme.id, p.id)"
                >
                  <i class="live-dot" />{{ store.settings.companionEnabled ? `${onDeskOf(theme.id, p.id).length} 只` : '已隐藏' }}
                </button>
                <PetActor
                  v-if="hover === `${theme.id}/${p.id}`"
                  :definition="p"
                  :theme-id="theme.id"
                  :motion="onDeskOf(theme.id, p.id)[0]?.motion"
                  :size="180"
                  gallery
                  :expression="galleryMood(theme.id, p)"
                  :paused="store.settings.paused"
                />
                <img v-else :src="petImage(p, galleryMood(theme.id, p))" :alt="p.name" width="150" height="150" />
              </div>

              <div class="card-foot">
                <b class="card-name" :title="p.description" :aria-description="p.description">{{ p.name }}</b>
                <template v-if="onDeskOf(theme.id, p.id).length">
                  <button
                    type="button"
                    class="card-cta-manage"
                    aria-haspopup="dialog"
                    :aria-expanded="isMenuOpen('manage', `${theme.id}/${p.id}`)"
                    :data-testid="`pet-manage-${onDeskOf(theme.id, p.id)[0].id}`"
                    @click="managePet($event, theme.id, p.id)"
                  >
                    管理<span v-if="onDeskOf(theme.id, p.id).length > 1" aria-hidden="true">▾</span>
                  </button>
                  <button
                    type="button"
                    class="card-cta-plus"
                    :disabled="!ready || busy"
                    :title="`再添加一只${p.name}`"
                    :aria-label="`再添加一只${p.name}`"
                    aria-haspopup="menu"
                    :aria-expanded="isMenuOpen('add', `${theme.id}/${p.id}`)"
                    :data-testid="`pet-add-${p.id}`"
                    @click="toggleMenu($event, 'add', `${theme.id}/${p.id}`)"
                  >
                    <Plus :size="14" />
                  </button>
                </template>
                <button
                  v-else
                  type="button"
                  class="card-cta-add"
                  :disabled="!ready || busy"
                  aria-haspopup="menu"
                  :aria-expanded="isMenuOpen('add', `${theme.id}/${p.id}`)"
                  :data-testid="`pet-add-${p.id}`"
                  @click="toggleMenu($event, 'add', `${theme.id}/${p.id}`)"
                >
                  <Plus :size="13" />添加
                </button>

                <!-- 轻量就地层级菜单（替代全屏遮罩弹窗） -->
                <div
                  v-if="isMenuOpen('add', `${theme.id}/${p.id}`)"
                  class="card-menu xb-menu"
                  :class="{ 'is-up': openMenu?.up }"
                  role="menu"
                  :aria-label="`将${p.name}放到桌面`"
                  data-testid="pet-placement-menu"
                >
                  <div class="xb-menu-label">放到桌面</div>
                  <button
                    v-for="option in placements"
                    :key="option.value"
                    type="button"
                    role="menuitem"
                    class="xb-menu-item"
                    :disabled="busy || !ready"
                    :title="option.hint"
                    :aria-description="option.hint"
                    :data-testid="`pet-placement-${option.value}`"
                    @click="add(theme.id, p.id, option.value)"
                  >
                    <component :is="option.icon" :size="14" />
                    {{ option.label }}
                    <span v-if="option.badge" class="xb-menu-hint">{{ option.badge }}</span>
                  </button>
                </div>

                <!-- 同款多实例区分管理菜单 -->
                <div
                  v-if="isMenuOpen('manage', `${theme.id}/${p.id}`)"
                  class="instance-menu xb-menu"
                  :class="{ 'is-up': openMenu?.up }"
                  role="menu"
                  :aria-label="`管理${p.name}`"
                >
                  <div class="xb-menu-label">桌面上的实例（{{ onDeskOf(theme.id, p.id).length }} 只）</div>
                  <button
                    v-for="instance in onDeskOf(theme.id, p.id)"
                    :key="instance.id"
                    type="button"
                    role="menuitem"
                    class="xb-menu-item"
                    @click="openManage(instance.id)"
                  >
                    <Settings2 :size="14" />
                    <span class="menu-item-name">{{ instance.name }}</span>
                    <span class="xb-menu-hint">{{ instanceHint(instance) }}</span>
                  </button>
                </div>
              </div>
            </article>
          </div>
        </section>
      </template>

      <!-- 在桌面上：两行清晰实例管理卡片 -->
      <section v-else class="desktop-pets" data-testid="desktop-pets">
        <div v-if="!instances.length" class="desktop-pets-empty">
          还没有小伙伴，挑一只喜欢的添加到桌面吧。
          <button type="button" class="xb-link" @click="closeMenus(); filter = 'all'">去挑选伙伴</button>
        </div>

        <template v-else>
          <div class="desk-summary">
            <span>共 <strong>{{ instances.length }}</strong> 只在桌面上 · <strong>{{ visibleCount }}</strong> 只显示中</span>
            <button type="button" class="xb-btn-ghost desk-toggle-all" :disabled="busy" data-testid="pet-toggle-all" @click="toggleAllInstances(visibleCount === 0)">
              {{ visibleCount === 0 ? '全部显示' : '全部隐藏' }}
            </button>
          </div>

          <section v-for="group in deskGroups" :key="group.theme?.id ?? 'orphans'" class="pet-theme-detail">
            <header class="th-head">
              <h2>{{ group.theme?.name ?? '作品暂不可用' }}</h2>
              <span class="sub">{{ group.items.length }} 只</span>
            </header>

            <div class="pet-cards">
              <article
                v-for="p in group.items"
                :key="p.id"
                class="pc pet-card instance"
                :class="{ off: !p.enabled || !store.settings.companionEnabled, 'is-menu-open': isMenuOpen('deskMore', p.id) }"
                :data-testid="`pet-instance-${p.id}`"
                @contextmenu.prevent.stop="openManage(p.id)"
                @pointerenter="hover = p.id"
                @pointerleave="hover = ''"
              >
                <div class="card-stage">
                  <span class="stage-pill left">{{ EXPRESSIONS[p.expression].label }}</span>
                  <span v-if="!p.enabled || p.monitor" class="stage-pill right">{{ p.enabled ? p.monitor : '已隐藏' }}</span>
                  <template v-if="definitionOf(catalog.themes, p)">
                    <PetActor v-if="hover === p.id" :definition="definitionOf(catalog.themes, p)!" :theme-id="p.themeId" :motion="p.motion" :size="180" gallery :expression="p.expression" :links="p.links" :paused="store.settings.paused" />
                    <img v-else :src="petImage(definitionOf(catalog.themes, p)!, p.expression)" :alt="p.name" width="150" height="150" />
                  </template>
                  <span v-else class="gone">作品暂不可用</span>
                </div>

                <div class="inst-head">
                  <b class="card-name" :title="`${p.name}（${definitionOf(catalog.themes, p)?.name ?? '作品暂不可用'}）`">{{ p.name }}</b>
                  <NSwitch :value="p.enabled" size="small" :disabled="busy" :aria-label="`显示${p.name}`" :data-testid="`pet-visible-${p.id}`" @update:value="enabled => act(() => client.update(p.id, { enabled }))" />
                </div>

                <div class="inst-sub">
                  <span>{{ definitionOf(catalog.themes, p)?.name ?? '作品暂不可用' }}</span>
                  <span>·</span>
                  <span>{{ p.size }}px</span>
                </div>

                <div class="inst-controls">
                  <div class="xb-seg is-xs" role="radiogroup" :aria-label="`${p.name}显示层级`">
                    <button type="button" role="radio" :aria-checked="p.layer === 'topmost'" :disabled="busy" title="浮在所有窗口上方" :data-testid="`pet-desk-layer-topmost-${p.id}`" @click="setInstanceLayer(p, 'topmost')">置顶</button>
                    <button type="button" role="radio" :aria-checked="p.layer === 'desktop'" :disabled="busy" title="贴在桌面图标下方" :data-testid="`pet-desk-layer-desktop-${p.id}`" @click="setInstanceLayer(p, 'desktop')">贴桌面</button>
                  </div>
                  <button type="button" class="xb-icon-btn" title="伙伴设置" aria-haspopup="dialog" :data-testid="`pet-manage-${p.id}`" @click="openManage(p.id)">
                    <Settings2 :size="14" />
                  </button>
                  <div class="more-wrap">
                    <button type="button" class="xb-icon-btn" :aria-label="`${p.name}更多操作`" aria-haspopup="menu" :aria-expanded="isMenuOpen('deskMore', p.id)" :data-testid="`pet-desk-more-${p.id}`" @click="toggleMenu($event, 'deskMore', p.id)">
                      <MoreHorizontal :size="15" />
                    </button>
                    <div v-if="isMenuOpen('deskMore', p.id)" class="card-menu xb-menu" :class="{ 'is-up': openMenu?.up }" role="menu">
                      <button type="button" role="menuitem" class="xb-menu-item" @click="openManage(p.id)">伙伴设置</button>
                      <button type="button" role="menuitem" class="xb-menu-item" @click="centerInstance(p.id)">移到屏幕中间</button>
                      <button v-if="definitionOf(catalog.themes, p)" type="button" role="menuitem" class="xb-menu-item" :disabled="busy" @click="add(p.themeId, p.petId, p.layer)">再添加一只</button>
                      <div class="xb-menu-sep" />
                      <button type="button" role="menuitem" class="xb-menu-item is-danger" :disabled="busy" :data-testid="`pet-desk-remove-${p.id}`" @click="removeInstance(p.id)">从桌面移除</button>
                    </div>
                  </div>
                </div>
              </article>
            </div>
          </section>
        </template>
      </section>
    </div>

    <!-- 浏览器预览或降级时的内嵌伙伴设置弹窗 -->
    <PetSettingsDialog
      v-if="current"
      show
      :instance="current"
      :themes="catalog.themes"
      :busy="busy"
      :paused="store.settings.paused"
      @update:show="value => { if (!value) editing = null; }"
      @save="saveSettings"
      @remove="removeCurrent"
      @center="centerCurrent"
      @create="createFromCurrent"
    >
      <template #extra>
        <div v-if="currentTheme?.source === 'custom'" class="pet-share-panel" data-testid="pet-instance-share">
          <b title="打包当前所属的整个主题，不包含这只宠物的个人设置">分享自定义主题「{{ currentTheme.name }}」</b>
          <ResourceShareActions :store="store" kind="pet" :id="current!.themeId" :disabled="busy" />
        </div>
      </template>
    </PetSettingsDialog>

    <!-- 浏览器预览或降级时的内嵌主动陪伴弹窗 -->
    <PetProactivityPanel
      v-if="store.desktop"
      :show="proactivityOpen"
      :settings="greetingSettings"
      :ready="greetingReady"
      :busy="greetingBusy"
      :environment="environment"
      @update:show="proactivityOpen = $event"
      @change="changeGreeting"
      @inspect="inspectEnvironment"
    />
  </section>
</template>

<style scoped>
.pets-page{container-type:inline-size;height:100%;min-height:0;min-width:0;display:flex;flex-direction:column;overflow:hidden}
.pt-toolbar{flex:none;display:flex;align-items:center;gap:10px;flex-wrap:wrap;padding:16px 28px 10px}
.pt-toolbar .cnt{font-weight:500;color:var(--ui-text-secondary);margin-left:3px;font-variant-numeric:tabular-nums}
.pt-tools{margin-left:auto;display:flex;align-items:center;gap:10px}
.pt-label{font-size:12px;color:var(--ui-text-secondary)}
.proactivity-pill{display:inline-flex;align-items:center;gap:6px;height:28px;padding:0 10px;border:0;border-radius:999px;background:var(--ui-control);color:var(--ui-text-secondary);font-size:12px;font-weight:500;cursor:pointer}
.proactivity-pill:hover{background:var(--ui-control-hover);color:var(--ui-text-primary)}
.proactivity-pill.is-on{color:var(--ui-text-primary)}
.proactivity-pill .p-dot{width:6px;height:6px;border-radius:50%;background:var(--ui-text-disabled);flex:none}
.proactivity-pill.is-on .p-dot{background:var(--ui-success)}

.pt-scroll{flex:1;min-height:0;overflow-y:auto;padding:4px 28px 32px;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent;overscroll-behavior:contain}
.pt-banner{margin-top:8px;padding:9px 14px;border-radius:10px;background:var(--ui-surface-sunken);color:var(--ui-text-secondary);display:flex;align-items:center;gap:9px;font-size:12.5px}
.pt-banner .banner-act{margin-left:auto}
.pt-alert{margin-top:14px;border-radius:10px}

.pet-theme-detail{margin-top:24px}
.pet-theme-detail:first-child{margin-top:10px}
.th-head{display:flex;align-items:center;gap:9px;min-width:0}
.th-head h2{margin:0;font-size:15px;font-weight:600;white-space:nowrap}
.th-head .sub{font-size:12px;color:var(--ui-text-secondary);min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.theme-actions{margin-left:auto;position:relative;display:flex;align-items:center;gap:6px;flex:none}

/* 4 → 3 → 2 列自适应画廊 */
.pet-cards{margin-top:12px;display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:18px}
@container (max-width:780px){.pet-cards{grid-template-columns:repeat(3,minmax(0,1fr));gap:16px}}
@container (max-width:540px){
  .pet-cards{grid-template-columns:repeat(2,minmax(0,1fr));gap:12px}
  .pt-toolbar{padding:12px 16px 8px}
  .pt-scroll{padding:4px 16px 24px}
}

/* 纯净中性浅画框卡片 */
.pc{position:relative;min-width:0;border-radius:16px;padding:8px 8px 10px;background:var(--ui-surface);box-shadow:var(--ui-shadow-card);display:flex;flex-direction:column;gap:9px;transition:transform .2s ease,box-shadow .2s ease}
.pc:hover{transform:translateY(-2px);box-shadow:0 0 0 1px var(--ui-border-strong),0 10px 24px var(--ui-shadow-hover)}
.pc.is-menu-open{z-index:40}
.pc.off .card-stage{opacity:.46;filter:grayscale(.55)}

.card-stage{position:relative;aspect-ratio:1.08;border-radius:12px;display:flex;align-items:center;justify-content:center;background:radial-gradient(ellipse 76% 62% at 50% 34%,#fff 0%,#faf8f5 58%,#f1ede7 100%);box-shadow:inset 0 1px 0 rgba(255,255,255,.92),inset 0 0 0 1px rgba(120,106,92,.065);overflow:hidden}
html[data-theme=dark] .card-stage{background:radial-gradient(ellipse 74% 58% at 50% 36%,#302e2c 0%,#242322 64%,#1c1b1a 100%);box-shadow:inset 0 1px 0 rgba(255,255,255,.05),inset 0 0 0 1px rgba(255,255,255,.06)}
.card-stage img,.card-stage :deep(.pet-actor){position:absolute!important;inset:10px;width:calc(100% - 20px)!important;height:calc(100% - 20px)!important;object-fit:contain;transition:transform .24s ease}
.pc:hover .card-stage img,.pc:hover .card-stage :deep(.pet-actor){transform:translateY(-3px)}
.card-stage .gone{font-size:12px;color:var(--ui-text-tertiary)}

.stage-pill{position:absolute;top:8px;height:21px;padding:0 7px;border:0;border-radius:999px;background:color-mix(in srgb,var(--ui-surface) 90%,transparent);-webkit-backdrop-filter:blur(10px);backdrop-filter:blur(10px);box-shadow:0 1px 3px rgba(0,0,0,.06),inset 0 0 0 1px var(--ui-line);font-size:10.5px;font-weight:500;color:var(--ui-text-secondary);display:inline-flex;align-items:center;gap:4px;z-index:3;white-space:nowrap}
.stage-pill.left{left:8px}
.stage-pill.right{right:8px}
.stage-pill.is-clickable{cursor:pointer}
.stage-pill.is-clickable:hover{background:var(--ui-surface);color:var(--ui-text-primary)}
.stage-pill .live-dot{width:5.5px;height:5.5px;border-radius:50%;background:var(--ui-success)}
.stage-pill.is-muted .live-dot{background:var(--ui-text-disabled)}

/* 图鉴卡底栏（全中性克制控件） */
.card-foot{display:flex;align-items:center;gap:6px;padding:0 3px;min-height:28px;position:relative}
.card-name{flex:1;min-width:0;font-size:13px;font-weight:500;color:var(--ui-text-primary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.card-cta-add,.card-cta-manage{height:28px;padding:0 10px;border:0;border-radius:8px;flex:none;background:var(--ui-control);color:var(--ui-text-primary);font-size:12px;font-weight:500;display:inline-flex;align-items:center;gap:4px;cursor:pointer}
.card-cta-add:hover:not(:disabled),.card-cta-manage:hover:not(:disabled),.pc:hover .card-cta-add:not(:disabled){background:var(--ui-control-hover)}
.card-cta-plus{width:28px;height:28px;padding:0;border:0;border-radius:8px;flex:none;background:transparent;color:var(--ui-text-secondary);display:inline-grid;place-items:center;cursor:pointer}
.card-cta-plus:hover:not(:disabled){background:var(--ui-control);color:var(--ui-text-primary)}

/* 在桌面上实例卡 */
.desk-summary{margin-top:10px;padding:8px 12px;border-radius:10px;background:var(--ui-surface-sunken);display:flex;align-items:center;gap:10px;font-size:12px;color:var(--ui-text-secondary)}
.desk-summary strong{color:var(--ui-text-primary);font-weight:600}
.desk-toggle-all{margin-left:auto}
.inst-head{display:flex;align-items:center;gap:6px;padding:0 3px}
.inst-head .n-switch{flex:none}
.inst-sub{padding:0 3px;margin-top:-5px;font-size:11px;color:var(--ui-text-tertiary);display:flex;align-items:center;gap:5px;white-space:nowrap;overflow:hidden}
.inst-controls{display:flex;align-items:center;gap:5px;padding:0 2px;position:relative}
.inst-controls .xb-seg{flex:1}
.inst-controls .xb-seg>button{flex:1;height:24px;padding:0 4px;font-size:11.5px}
.more-wrap{position:relative;flex:none}

.desktop-pets-empty{margin-top:22px;padding:30px;border-radius:14px;background:var(--ui-surface-sunken);text-align:center;font-size:13px;color:var(--ui-text-secondary)}
.desktop-pets-empty .xb-link{margin-left:10px}
.pet-share-panel{display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:8px;padding:10px 14px;border-radius:12px;background:var(--ui-configuration-group)}
.pet-share-panel b{font-size:12.5px;font-weight:500;color:var(--ui-text-primary)}

/* 浮层菜单定位 */
.menu-dismiss{position:fixed;inset:0;z-index:30}
.theme-menu,.card-menu,.instance-menu{position:absolute;z-index:41;right:0;top:calc(100% + 6px);min-width:184px;max-height:260px;overflow:auto}
.card-menu.is-up,.instance-menu.is-up{top:auto;bottom:calc(100% + 6px)}
.theme-menu .xb-menu-item,.card-menu .xb-menu-item,.instance-menu .xb-menu-item{font-size:12.5px;min-height:32px}
.menu-item-name{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.pet-card button:focus-visible{outline:2px solid var(--ui-btn);outline-offset:2px}
</style>
