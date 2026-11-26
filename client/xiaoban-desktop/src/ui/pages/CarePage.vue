<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { NAlert, NSwitch } from 'naive-ui';
import {
  ArrowUpRight, Bell, Bot, ChevronRight, CloudSun, Coffee, Droplet,
  Ellipsis, Moon, Pin, Play, Plus, Settings2, Sparkles, Volume2,
} from 'lucide-vue-next';
import { invoke } from '../../bridge';
import {
  describeAgentRun, describeCareRule, describeRuleNotify, newCareRule,
  resolveRuleNotify, weatherIsFresh,
  type CareCity, type CareKind, type CareRule, type CareStatus,
} from '../../care-model';
import type { ControlStore } from '../control-types';
import CareEditorDialog from '../care/CareEditorDialog.vue';
import { clone, createCareState } from '../care/state';
import {
  formatCountdown, isQuietNow, isRuleCompleted, nextRunDate,
  nextRunLabel, parseQuickCareInput,
} from '../care/schedule';

const props = defineProps<{ store: ControlStore }>();
const care = createCareState(props.store.desktop, props.store.notify);
const { status, preferences, preferencesDirty, draft, editing, busy, runningAgentIds } = care;

const weatherEnabled = computed(() => status.value.config.weatherSource.provider !== 'disabled');
const cityQuery = ref('');
const cities = ref<CareCity[]>([]);
const searchDone = ref(false);
const locating = ref(false);
const located = ref(false);
const editorDirtyText = ref(false);

const tab = ref<'reminders' | 'tasks'>('reminders');
const activeFilter = ref<'all' | 'today' | 'routine' | 'paused'>('all');
const showCompleted = ref(false);
const prefsOpen = ref(false);
const prefsSavedFlash = ref(false);
const openMenuId = ref<string | null>(null);
const confirmDeleteId = ref<string | null>(null);

const quickText = ref('');
const quickSound = ref(false);
const quickVoice = ref(false);
const quickSticky = ref(false);

const now = ref(new Date());
let clockTimer: ReturnType<typeof setInterval> | undefined;
let savedFlashTimer: ReturnType<typeof setTimeout> | undefined;

const kindIcon = { custom: Droplet, 'off-work': Moon, rest: Coffee, weather: CloudSun, agent: Bot } as const;

const defaultTextByKind: Record<CareKind, string> = {
  custom: newCareRule('custom').text,
  'off-work': newCareRule('off-work').text,
  rest: newCareRule('rest').text,
  weather: '',
  agent: newCareRule('agent').text,
};

const reminderTemplates: { kind: CareKind; title: string; desc: string; tag: string; preset: Partial<CareRule> & { kind: CareKind } }[] = [
  {
    kind: 'custom', title: '工作日定时喝水', desc: '09:00–18:00 每 90 分钟静音气泡', tag: '静音气泡',
    preset: { kind: 'custom', repeat: 'interval', intervalEveryMinutes: 90, windowStart: '09:00', windowEnd: '18:00', weekdays: [1, 2, 3, 4, 5], text: '喝杯温水，顺便活动一下手腕～', sound: false, voice: false, sticky: false },
  },
  {
    kind: 'rest', title: '护眼 20-20-20', desc: '累计用电脑 20 分钟眺望远处', tag: '久坐累计',
    preset: { kind: 'rest', intervalMinutes: 20, text: '看屏幕满 20 分钟啦，抬头看远处放松眼睛 20 秒。', sound: false, voice: false, sticky: false },
  },
  {
    kind: 'weather', title: '早安语音天气', desc: '每天 08:00 朗读所在城市天气', tag: '语音朗读',
    preset: { kind: 'weather', repeat: 'daily', time: '08:00', text: '早安！今日天气：', sound: false, voice: true, sticky: false },
  },
];

const taskTemplates: { kind: CareKind; title: string; desc: string; tag: string; preset: Partial<CareRule> & { kind: CareKind } }[] = [
  {
    kind: 'agent', title: '每日文件变动总结', desc: '工作日 09:15 自动梳理昨日改动', tag: '静默汇报',
    preset: { kind: 'agent', repeat: 'weekdays', weekdays: [1, 2, 3, 4, 5], time: '09:15', text: '查看工作目录昨天新增或修改的文件，用三句话总结重点。', sound: false, voice: false, sticky: false },
  },
  {
    kind: 'agent', title: '每周五写周报', desc: '周五 17:00 汇总本周成果草稿', tag: '提示音',
    preset: { kind: 'agent', repeat: 'weekdays', weekdays: [5], time: '17:00', text: '整理本周完成的功能与待验收项，生成一份简明周报草稿。', sound: true, voice: false, sticky: false },
  },
  {
    kind: 'agent', title: '关注网页动态简报', desc: '每天 10:00 检查发布页更新要点', tag: '静音气泡',
    preset: { kind: 'agent', repeat: 'daily', time: '10:00', text: '查看我关注的更新日志网页，如有新版本则提炼核心变化。', sound: false, voice: false, sticky: false },
  },
];

const allReminders = computed(() => status.value.rules.filter(rule => rule.kind !== 'agent'));
const completedReminders = computed(() => allReminders.value.filter(rule => isRuleCompleted(rule, now.value)));
const activeReminders = computed(() => allReminders.value.filter(rule => !isRuleCompleted(rule, now.value)));
const tasks = computed(() => status.value.rules.filter(rule => rule.kind === 'agent'));

function sortByNextRun(list: CareRule[]): CareRule[] {
  return [...list].sort((a, b) => {
    if (a.enabled !== b.enabled) return a.enabled ? -1 : 1;
    const ta = nextRunDate(a, now.value)?.getTime() ?? Number.MAX_SAFE_INTEGER;
    const tb = nextRunDate(b, now.value)?.getTime() ?? Number.MAX_SAFE_INTEGER;
    return ta - tb;
  });
}

const enabledReminders = computed(() => sortByNextRun(activeReminders.value.filter(rule => rule.enabled)));
const pausedReminders = computed(() => activeReminders.value.filter(rule => !rule.enabled));

const filteredReminders = computed(() => {
  if (activeFilter.value === 'paused') return pausedReminders.value;
  if (activeFilter.value === 'today') {
    const todayEnd = new Date(now.value.getFullYear(), now.value.getMonth(), now.value.getDate(), 23, 59, 59).getTime();
    return enabledReminders.value.filter(rule => {
      const next = nextRunDate(rule, now.value);
      return next ? next.getTime() <= todayEnd : false;
    });
  }
  if (activeFilter.value === 'routine') {
    return enabledReminders.value.filter(rule => rule.kind === 'rest' || rule.kind === 'weather' || rule.repeat === 'daily' || rule.repeat === 'interval');
  }
  return sortByNextRun(activeReminders.value);
});

const nextSpotlight = computed(() => {
  for (const rule of enabledReminders.value) {
    const date = nextRunDate(rule, now.value);
    if (date) {
      const pad = (n: number) => String(n).padStart(2, '0');
      return {
        rule,
        date,
        clock: `${pad(date.getHours())}:${pad(date.getMinutes())}`,
        countdown: formatCountdown(date, now.value),
        notifyText: describeRuleNotify(resolveRuleNotify(rule, status.value.config)),
      };
    }
  }
  return null;
});

const isPausedGlobal = computed(() => !status.value.config.enabled);
const isQuietActive = computed(() => !isPausedGlobal.value && isQuietNow(status.value.config, now.value));
const topStatusText = computed(() => {
  if (isPausedGlobal.value) return '已全局暂停';
  if (isQuietActive.value) return `免打扰生效中 · ${preferences.quietStart}–${preferences.quietEnd}`;
  if (preferences.quietEnabled) return `免打扰 ${preferences.quietStart}–${preferences.quietEnd}`;
  return '全天候提醒';
});

const quickParsed = computed(() => parseQuickCareInput(quickText.value, tab.value === 'tasks', status.value.config, now.value));
const quickEffectiveNotify = computed(() => {
  const parsed = quickParsed.value;
  return {
    sound: quickSound.value || Boolean(parsed?.rule.sound),
    voice: quickVoice.value || Boolean(parsed?.rule.voice),
    sticky: quickSticky.value || Boolean(parsed?.stickyHint),
  };
});

function ruleWeatherBadge(rule: CareRule): string | null {
  if (rule.kind !== 'weather' || !status.value.weather || !weatherIsFresh(status.value.weather)) return null;
  if (rule.city && status.value.weather.city.name !== rule.city.name) return null;
  return status.value.weather.text.slice(0, 26);
}

function triggerSavedFlash() {
  prefsSavedFlash.value = true;
  if (savedFlashTimer) clearTimeout(savedFlashTimer);
  savedFlashTimer = setTimeout(() => { prefsSavedFlash.value = false; }, 1600);
}

async function autoSavePrefs() {
  preferencesDirty.value = true;
  if (preferences.weatherSource.provider === 'custom') return;
  const ok = await care.savePreferences(true);
  if (ok) triggerSavedFlash();
}

async function togglePrefFlag(key: 'enabled' | 'sound' | 'onceSound' | 'voice' | 'onceVoice' | 'quietEnabled', value?: boolean) {
  preferences[key] = typeof value === 'boolean' ? value : !preferences[key];
  await autoSavePrefs();
}

async function saveCustomWeatherPrefs() {
  const ok = await care.savePreferences(false);
  if (ok) triggerSavedFlash();
}

function open(kind: CareKind = 'custom', rule?: CareRule) {
  care.open(kind, rule);
  editorDirtyText.value = Boolean(rule);
  cityQuery.value = '';
  cities.value = [];
  searchDone.value = false;
  located.value = false;
  if (kind === 'weather' && !rule?.city) void prefillCity();
}

function openFromTemplate(preset: Partial<CareRule> & { kind: CareKind }) {
  care.openPreset(preset);
  editorDirtyText.value = false;
  cityQuery.value = '';
  cities.value = [];
  searchDone.value = false;
  located.value = false;
  if (preset.kind === 'weather') void prefillCity();
}

function changeKind(kind: CareKind) {
  const defaults = newCareRule(kind);
  const keepText = editorDirtyText.value && draft.value.text.trim() && !Object.values(defaultTextByKind).includes(draft.value.text);
  draft.value = {
    ...draft.value,
    kind,
    text: keepText ? draft.value.text : defaults.text,
    time: defaults.time,
    city: kind === 'weather' ? draft.value.city : null,
  };
  if (kind === 'agent' && draft.value.repeat === 'interval') draft.value.repeat = 'daily';
  cities.value = [];
  located.value = false;
  if (kind === 'weather') void prefillCity();
}

// Fills the city from the network location. A city the user already chose is never replaced by the automatic one.
async function prefillCity(manual = false) {
  if (!weatherEnabled.value || locating.value || (!manual && draft.value.city)) return;
  if (!props.store.desktop) { if (manual) props.store.notify('自动定位需要桌面程序；浏览器预览不访问网络', 'info'); return; }
  locating.value = true;
  try {
    const city = await invoke<CareCity>('detect_care_city');
    if (editing.value && draft.value.kind === 'weather' && (manual || !draft.value.city)) {
      draft.value.city = clone(city);
      located.value = true;
      cities.value = [];
    }
  } catch (error) {
    if (manual) props.store.notify(error instanceof Error ? error.message : String(error), 'error');
  } finally {
    locating.value = false;
  }
}

async function searchCities() {
  if (!weatherEnabled.value) return;
  if (!props.store.desktop) { props.store.notify('城市搜索需要桌面程序；浏览器预览不访问天气网络', 'info'); return; }
  if (!cityQuery.value.trim()) { props.store.notify('请输入城市名', 'error'); return; }
  await care.run(async () => {
    cities.value = await invoke<CareCity[]>('search_care_cities', { query: cityQuery.value.trim() });
    searchDone.value = true;
  });
}

async function checkWeather() {
  if (!weatherEnabled.value || !draft.value.city) { props.store.notify('请先启用天气来源并选择城市', 'error'); return; }
  if (!props.store.desktop) { props.store.notify('天气查询需要桌面程序', 'info'); return; }
  await care.run(async () => {
    care.receive(await invoke<CareStatus>('check_care_weather', { city: clone(draft.value.city) }));
    props.store.notify('天气查询已完成', 'success');
  });
}

async function submitQuickAdd() {
  const parsed = quickParsed.value;
  if (!parsed) {
    props.store.notify('先输入你想被提醒的内容吧', 'info');
    return;
  }
  const rule = clone(parsed.rule);
  rule.sound = quickSound.value ? true : rule.sound;
  rule.voice = quickVoice.value ? true : rule.voice;
  rule.sticky = quickSticky.value ? true : rule.sticky;
  const ok = await care.quickAdd(rule);
  if (ok) {
    quickText.value = '';
    quickSound.value = false;
    quickVoice.value = false;
    quickSticky.value = false;
  }
}

async function toggleRuleChannel(rule: CareRule, channel: 'sound' | 'voice' | 'sticky') {
  const effective = resolveRuleNotify(rule, status.value.config);
  const next = !effective[channel];
  const labels = { sound: '提示音', voice: '语音朗读', sticky: '常驻强提醒' };
  await care.patchRule(rule, { [channel]: next }, `已为这条提醒${next ? '开启' : '关闭'}${labels[channel]}`);
}

async function snoozeSpotlight(rule: CareRule) {
  const target = Math.floor(Date.now() / 1000) + 10 * 60;
  await care.patchRule(rule, { notBefore: target }, '已推迟 10 分钟');
}

async function openAgentSession(sessionId: string | null | undefined) {
  if (!sessionId) {
    props.store.navigate('chat');
    return;
  }
  props.store.navigate('chat');
  try {
    if (props.store.chat && typeof props.store.chat.select === 'function') {
      await props.store.chat.select(sessionId);
    }
  } catch (error) {
    props.store.notify(error instanceof Error ? error.message : String(error), 'error');
  }
}

function toggleRowMenu(id: string) {
  confirmDeleteId.value = null;
  openMenuId.value = openMenuId.value === id ? null : id;
}

async function handleDeleteClick(rule: CareRule) {
  if (confirmDeleteId.value !== rule.id) {
    confirmDeleteId.value = rule.id;
    return;
  }
  openMenuId.value = null;
  confirmDeleteId.value = null;
  await care.remove(rule);
}

function closeMenus() {
  openMenuId.value = null;
  confirmDeleteId.value = null;
  prefsOpen.value = false;
}

function handleDocumentClick() {
  if (openMenuId.value) { openMenuId.value = null; confirmDeleteId.value = null; }
  if (prefsOpen.value) prefsOpen.value = false;
}

onMounted(() => {
  void care.initialize();
  clockTimer = setInterval(() => { now.value = new Date(); }, 30_000);
  document.addEventListener('click', handleDocumentClick);
});

onUnmounted(() => {
  if (clockTimer) clearInterval(clockTimer);
  if (savedFlashTimer) clearTimeout(savedFlashTimer);
  document.removeEventListener('click', handleDocumentClick);
  care.dispose();
});

defineExpose({ open, prefillCity, changeKind });
</script>

<template>
  <main class="xb-page care-page" data-testid="care-page">
    <header class="care-head" data-tauri-drag-region>
      <div class="care-inner care-head-row">
        <div class="xb-tabs-lg" role="tablist" aria-label="提醒类型">
          <button type="button" role="tab" data-testid="care-tab-reminders" :aria-selected="tab === 'reminders'" @click="tab = 'reminders'; closeMenus()">
            提醒 <span class="care-count">{{ activeReminders.length }}</span>
          </button>
          <button type="button" role="tab" data-testid="care-tab-tasks" :aria-selected="tab === 'tasks'" @click="tab = 'tasks'; closeMenus()">
            定时任务 <span class="care-count">{{ tasks.length }}</span>
          </button>
        </div>

        <div class="care-head-actions">
          <button
            type="button"
            class="care-status-pill"
            :class="{ 'is-paused': isPausedGlobal, 'is-quiet': isQuietActive }"
            title="点击调整免打扰或全局总开关"
            @click.stop="prefsOpen = !prefsOpen"
          >
            <span class="care-status-dot" />
            <span>{{ topStatusText }}</span>
          </button>

          <button
            type="button"
            class="care-icon-tool"
            data-testid="care-prefs-toggle"
            :aria-expanded="prefsOpen"
            aria-label="全局提醒偏好"
            title="全局默认偏好、免打扰与天气来源"
            @click.stop="prefsOpen = !prefsOpen"
          >
            <Settings2 :size="17" />
          </button>

          <button
            v-if="tab === 'reminders'"
            type="button"
            class="xb-btn-primary"
            data-testid="care-add"
            @click="open('custom')"
          >
            <Plus :size="14" />新建提醒
          </button>
          <button
            v-else
            type="button"
            class="xb-btn-primary"
            data-testid="care-add-task"
            @click="open('agent')"
          >
            <Plus :size="14" />新建任务
          </button>

          <!-- 全局默认偏好浮层（平时收起，不占右侧栏） -->
          <section
            v-show="prefsOpen"
            class="care-prefs-pop"
            data-testid="care-prefs-popover"
            aria-label="全局提醒偏好"
            @click.stop
          >
            <div class="cp-head">
              <strong>全局提醒偏好</strong>
              <span class="cp-saved" :class="{ 'is-show': prefsSavedFlash }">已自动保存</span>
            </div>

            <div class="cp-sec">
              <div class="cp-row" title="关闭后暂停全部桌面提醒气泡与定时任务">
                <div class="cp-row-text">
                  <span>启用提醒调度</span>
                  <small>关闭后暂停全部气泡与定时任务</small>
                </div>
                <NSwitch :value="preferences.enabled" :disabled="busy" @update:value="togglePrefFlag('enabled', $event)" />
              </div>
            </div>

            <div class="cp-sec">
              <span class="cp-sec-label">新建提醒默认方式（每条可单独改）</span>
              <div class="cp-row" title="所有提醒默认播放提示音">
                <span class="cp-name">默认提示音</span>
                <NSwitch :value="preferences.sound" :disabled="busy" @update:value="togglePrefFlag('sound', $event)" />
              </div>
              <div class="cp-row" title="只提醒一次的事默认响一声，即使上面关闭">
                <span class="cp-name">仅一次默认响铃</span>
                <NSwitch :value="preferences.onceSound" data-testid="care-once-sound" :disabled="busy" @update:value="togglePrefFlag('onceSound', $event)" />
              </div>
              <div class="cp-row" title="使用「设置 → 声音」的音色朗读；在线音色会发送文案给在线语音服务">
                <span class="cp-name">默认语音播报</span>
                <NSwitch :value="preferences.voice" data-testid="care-voice-switch" :disabled="busy" @update:value="togglePrefFlag('voice', $event)" />
              </div>
              <div class="cp-row" title="仅一次提醒默认朗读出来；在线失败回退系统朗读">
                <span class="cp-name">仅一次默认朗读</span>
                <NSwitch :value="preferences.onceVoice" data-testid="care-once-voice" :disabled="busy" @update:value="togglePrefFlag('onceVoice', $event)" />
              </div>
            </div>

            <div class="cp-sec">
              <div class="cp-row" title="支持跨午夜，期间的有效提醒将延后，定时任务仍在后台运行">
                <div class="cp-row-text">
                  <span>免打扰时段</span>
                  <small>跨午夜有效，期间提醒顺延</small>
                </div>
                <NSwitch :value="preferences.quietEnabled" :disabled="busy" @update:value="togglePrefFlag('quietEnabled', $event)" />
              </div>
              <div v-if="preferences.quietEnabled" class="cp-time-pair">
                <input v-model="preferences.quietStart" type="time" aria-label="免打扰开始" @change="autoSavePrefs" />
                <span>至</span>
                <input v-model="preferences.quietEnd" type="time" aria-label="免打扰结束" @change="autoSavePrefs" />
              </div>
            </div>

            <div class="cp-sec">
              <span class="cp-sec-label">天气数据来源</span>
              <select
                v-model="preferences.weatherSource.provider"
                class="cp-select"
                aria-label="天气来源"
                @change="autoSavePrefs"
              >
                <option value="open-meteo">Open-Meteo（默认 · 个人非商业）</option>
                <option value="custom">自定义兼容 HTTPS 来源</option>
                <option value="disabled">关闭（不访问天气网络）</option>
              </select>
              <template v-if="preferences.weatherSource.provider === 'custom'">
                <input v-model="preferences.weatherSource.forecastUrl" type="url" class="cp-input" placeholder="天气 API · https://..." @input="preferencesDirty = true" />
                <input v-model="preferences.weatherSource.geocodingUrl" type="url" class="cp-input" placeholder="城市搜索 API · https://..." @input="preferencesDirty = true" />
                <button type="button" class="xb-btn-primary cp-save-btn" :disabled="busy" data-testid="care-save-preferences" @click="saveCustomWeatherPrefs">保存来源地址</button>
              </template>
              <p class="cp-note">仅发送所选城市坐标；新建天气提醒自动定位时会向 ipwho.is / ip.sb 查询公网 IP 所在城市，关闭后不访问网络。</p>
            </div>
          </section>
        </div>
      </div>
    </header>

    <div class="care-scroll">
      <div class="care-inner care-stack">
        <NAlert v-if="!store.desktop" type="info" :show-icon="false">浏览器预览：初始没有提醒，可临时体验新建与编辑，本次预览不保存、不在后台触发。</NAlert>
        <NAlert v-if="status.error" type="error">{{ status.error }}</NAlert>
        <NAlert v-if="status.weatherError" type="warning">{{ status.weatherError }}</NAlert>

        <!-- 1. 一句话快建栏 -->
        <form class="care-qc" data-testid="care-quick-capture" @submit.prevent="submitQuickAdd">
          <div class="qc-row">
            <Sparkles :size="15" class="qc-ico" />
            <input
              v-model="quickText"
              type="text"
              class="qc-input"
              data-testid="care-quick-input"
              :placeholder="tab === 'reminders' ? '一句话添加提醒，例如「30分钟后关火」「工作日每90分钟喝水」「明天9点交材料」…' : '一句话新建定时任务，例如「每天 9:15 总结昨天改动的文件」「周五 17:00 写周报」…'"
              autocomplete="off"
            />
            <div class="qc-actions">
              <button type="button" class="qc-ch" :aria-pressed="quickSound" title="这条提醒带提示音" @click="quickSound = !quickSound"><Bell :size="13" /></button>
              <button type="button" class="qc-ch" :aria-pressed="quickVoice" title="这条提醒语音朗读" @click="quickVoice = !quickVoice"><Volume2 :size="13" /></button>
              <button type="button" class="qc-ch" :aria-pressed="quickSticky" title="这条提醒常驻不自动消失" @click="quickSticky = !quickSticky"><Pin :size="13" /></button>
              <button type="submit" class="qc-submit" :disabled="busy || !quickText.trim()" data-testid="care-quick-submit">添加</button>
            </div>
          </div>
          <div v-if="quickParsed" class="qc-parsed">
            <span>识别为</span>
            <span class="qc-tag">{{ quickParsed.label }}</span>
            <span class="qc-tag is-neutral">{{ describeRuleNotify(quickEffectiveNotify) }}</span>
            <span class="qc-enter">按 Enter 快速添加</span>
          </div>
        </form>

        <!-- 2. 焦点卡：全局暂停 / 免打扰中 / 下一条即将到来 -->
        <section v-if="isPausedGlobal" class="care-spotlight is-paused">
          <div class="spot-body">
            <span class="spot-eyebrow">提醒调度已全局暂停</span>
            <span class="spot-title">所有桌面提醒气泡与定时任务均已暂缓</span>
            <span class="spot-meta">你可以随时恢复，或继续在下方编辑和管理规则。</span>
          </div>
          <button type="button" class="xb-btn-primary" :disabled="busy" @click="togglePrefFlag('enabled', true)">立即恢复</button>
        </section>

        <section v-else-if="isQuietActive" class="care-spotlight is-quiet">
          <div class="spot-body">
            <span class="spot-eyebrow">免打扰生效中 · {{ preferences.quietStart }} – {{ preferences.quietEnd }}</span>
            <span class="spot-title">关怀提醒将在免打扰结束后补发有效摘要</span>
            <span class="spot-meta">定时任务仍会按时在后台完成并写入聊天记录。</span>
          </div>
          <button type="button" class="spot-btn-soft" :disabled="busy" @click="togglePrefFlag('quietEnabled', false)">关闭免打扰</button>
        </section>

        <section v-else-if="tab === 'reminders' && nextSpotlight" class="care-spotlight" aria-label="下一条提醒">
          <div class="spot-clock">
            <strong>{{ nextSpotlight.clock }}</strong>
            <span>{{ nextSpotlight.countdown }}</span>
          </div>
          <div class="spot-body">
            <span class="spot-eyebrow">下一条提醒</span>
            <span class="spot-title">{{ nextSpotlight.rule.text || '根据真实天气生成提醒' }}</span>
            <span class="spot-meta">
              <span>{{ describeCareRule(nextSpotlight.rule) }}</span>
              <span>·</span>
              <span>{{ nextSpotlight.notifyText }}</span>
            </span>
          </div>
          <div class="spot-actions">
            <button type="button" class="spot-btn-soft" :disabled="busy" @click="care.preview(nextSpotlight.rule)">试看气泡</button>
            <button type="button" class="spot-btn-ghost" :disabled="busy" @click="snoozeSpotlight(nextSpotlight.rule)">推迟 10 分</button>
          </div>
        </section>

        <!-- 3A. 关怀提醒 Tab -->
        <template v-if="tab === 'reminders'">
          <template v-if="activeReminders.length">
            <div class="care-filters" role="group" aria-label="筛选提醒">
              <button type="button" class="filter-chip" :aria-pressed="activeFilter === 'all'" @click="activeFilter = 'all'">全部 ({{ activeReminders.length }})</button>
              <button type="button" class="filter-chip" :aria-pressed="activeFilter === 'today'" @click="activeFilter = 'today'">今日待触发</button>
              <button type="button" class="filter-chip" :aria-pressed="activeFilter === 'routine'" @click="activeFilter = 'routine'">周期与天气</button>
              <button type="button" class="filter-chip" :aria-pressed="activeFilter === 'paused'" @click="activeFilter = 'paused'">已暂停</button>
              <span class="filter-hint">悬停行可直切提示音 / 朗读 / 常驻</span>
            </div>

            <template v-if="activeFilter === 'all'">
              <div v-if="enabledReminders.length" class="care-group-block">
                <div class="care-group-title">进行中 · {{ enabledReminders.length }}</div>
                <section class="xb-group care-list" aria-label="进行中的提醒">
                  <article
                    v-for="rule in enabledReminders"
                    :key="rule.id"
                    class="r-row"
                    :class="{ 'menu-open': openMenuId === rule.id }"
                    :data-testid="`care-rule-${rule.id}`"
                    @click="open(rule.kind, rule)"
                  >
                    <span class="r-ico" :class="`is-${rule.kind}`" aria-hidden="true"><component :is="kindIcon[rule.kind]" :size="17" /></span>
                    <div class="r-body">
                      <span class="r-title">{{ rule.text || '根据真实天气生成提醒' }}</span>
                      <div class="r-sub">
                        <span>{{ describeCareRule(rule) }}</span>
                        <span v-if="ruleWeatherBadge(rule)" class="weather-pill">{{ ruleWeatherBadge(rule) }}</span>
                        <span v-if="rule.notBefore > Date.now() / 1000" class="snooze-tag">已推迟</span>
                      </div>
                    </div>

                    <div class="channel-strip" aria-label="这条提醒的方式" @click.stop>
                      <button
                        type="button"
                        class="ch-btn"
                        :class="{ 'is-on': resolveRuleNotify(rule, status.config).sound }"
                        :disabled="busy"
                        :title="resolveRuleNotify(rule, status.config).sound ? '提示音已开（点击关闭）' : '开启这条提醒的提示音'"
                        @click="toggleRuleChannel(rule, 'sound')"
                      ><Bell :size="13" /></button>
                      <button
                        type="button"
                        class="ch-btn"
                        :class="{ 'is-on': resolveRuleNotify(rule, status.config).voice }"
                        :disabled="busy"
                        :title="resolveRuleNotify(rule, status.config).voice ? '语音朗读已开（点击关闭）' : '开启这条提醒的语音朗读'"
                        @click="toggleRuleChannel(rule, 'voice')"
                      ><Volume2 :size="13" /></button>
                      <button
                        type="button"
                        class="ch-btn"
                        :class="{ 'is-on-neutral': resolveRuleNotify(rule, status.config).sticky }"
                        :disabled="busy"
                        :title="resolveRuleNotify(rule, status.config).sticky ? '常驻强提醒已开（点击改为自动收起）' : '设为常驻强提醒'"
                        @click="toggleRuleChannel(rule, 'sticky')"
                      ><Pin :size="13" /></button>
                    </div>

                    <div class="r-right" @click.stop>
                      <span class="r-next" :data-testid="`care-next-${rule.id}`">{{ nextRunLabel(rule, now) }}</span>
                      <NSwitch size="small" :value="rule.enabled" :disabled="busy" :aria-label="`启用${rule.text}`" @update:value="care.toggle(rule, $event)" />
                      <div class="more-wrap">
                        <button type="button" class="row-more-btn" aria-label="更多操作" @click="toggleRowMenu(rule.id)"><Ellipsis :size="15" /></button>
                        <div v-if="openMenuId === rule.id" class="xb-menu r-menu" role="menu">
                          <button type="button" class="xb-menu-item" @click="openMenuId = null; open(rule.kind, rule)">编辑提醒</button>
                          <button type="button" class="xb-menu-item" @click="openMenuId = null; care.preview(rule)">试看气泡效果</button>
                          <button type="button" class="xb-menu-item" @click="openMenuId = null; care.duplicateRule(rule)">创建副本</button>
                          <div class="xb-menu-sep" />
                          <button type="button" class="xb-menu-item is-danger" @click="handleDeleteClick(rule)">
                            {{ confirmDeleteId === rule.id ? '确认删除？' : '删除' }}
                          </button>
                        </div>
                      </div>
                    </div>
                  </article>
                </section>
              </div>

              <div v-if="pausedReminders.length" class="care-group-block">
                <div class="care-group-title">已暂停 · {{ pausedReminders.length }}</div>
                <section class="xb-group care-list" aria-label="已暂停的提醒">
                  <article
                    v-for="rule in pausedReminders"
                    :key="rule.id"
                    class="r-row is-muted"
                    :class="{ 'menu-open': openMenuId === rule.id }"
                    :data-testid="`care-rule-${rule.id}`"
                    @click="open(rule.kind, rule)"
                  >
                    <span class="r-ico" :class="`is-${rule.kind}`" aria-hidden="true"><component :is="kindIcon[rule.kind]" :size="17" /></span>
                    <div class="r-body">
                      <span class="r-title">{{ rule.text || '根据真实天气生成提醒' }}</span>
                      <div class="r-sub"><span>{{ describeCareRule(rule) }}</span></div>
                    </div>
                    <div class="r-right" @click.stop>
                      <span class="r-next is-paused" :data-testid="`care-next-${rule.id}`">已暂停</span>
                      <NSwitch size="small" :value="rule.enabled" :disabled="busy" :aria-label="`启用${rule.text}`" @update:value="care.toggle(rule, $event)" />
                      <div class="more-wrap">
                        <button type="button" class="row-more-btn" aria-label="更多操作" @click="toggleRowMenu(rule.id)"><Ellipsis :size="15" /></button>
                        <div v-if="openMenuId === rule.id" class="xb-menu r-menu" role="menu">
                          <button type="button" class="xb-menu-item" @click="openMenuId = null; open(rule.kind, rule)">编辑提醒</button>
                          <button type="button" class="xb-menu-item" @click="openMenuId = null; care.duplicateRule(rule)">创建副本</button>
                          <div class="xb-menu-sep" />
                          <button type="button" class="xb-menu-item is-danger" @click="handleDeleteClick(rule)">
                            {{ confirmDeleteId === rule.id ? '确认删除？' : '删除' }}
                          </button>
                        </div>
                      </div>
                    </div>
                  </article>
                </section>
              </div>
            </template>

            <section v-else-if="filteredReminders.length" class="xb-group care-list">
              <article
                v-for="rule in filteredReminders"
                :key="rule.id"
                class="r-row"
                :class="{ 'is-muted': !rule.enabled, 'menu-open': openMenuId === rule.id }"
                :data-testid="`care-rule-${rule.id}`"
                @click="open(rule.kind, rule)"
              >
                <span class="r-ico" :class="`is-${rule.kind}`" aria-hidden="true"><component :is="kindIcon[rule.kind]" :size="17" /></span>
                <div class="r-body">
                  <span class="r-title">{{ rule.text || '根据真实天气生成提醒' }}</span>
                  <div class="r-sub"><span>{{ describeCareRule(rule) }}</span></div>
                </div>
                <div class="r-right" @click.stop>
                  <span class="r-next" :class="{ 'is-paused': !rule.enabled }" :data-testid="`care-next-${rule.id}`">{{ nextRunLabel(rule, now) }}</span>
                  <NSwitch size="small" :value="rule.enabled" :disabled="busy" @update:value="care.toggle(rule, $event)" />
                </div>
              </article>
            </section>
            <div v-else class="xb-group care-filter-empty">当前筛选条件下暂无提醒</div>

            <!-- 已完成折叠条 -->
            <div v-if="completedReminders.length" class="care-group-block">
              <div class="done-bar">
                <button type="button" class="done-toggle" :aria-expanded="showCompleted" @click="showCompleted = !showCompleted">
                  <ChevronRight :size="13" class="done-chev" :class="{ 'is-open': showCompleted }" />
                  已完成（{{ completedReminders.length }}）
                </button>
                <button type="button" class="done-clear" :disabled="busy" @click="care.removeMany(completedReminders)">清空</button>
              </div>
              <section v-if="showCompleted" class="xb-group care-list">
                <article v-for="rule in completedReminders" :key="rule.id" class="r-row is-muted" @click="open(rule.kind, rule)">
                  <span class="r-ico" :class="`is-${rule.kind}`" aria-hidden="true"><component :is="kindIcon[rule.kind]" :size="17" /></span>
                  <div class="r-body">
                    <span class="r-title">{{ rule.text }}</span>
                    <div class="r-sub"><span>{{ describeCareRule(rule) }}</span></div>
                  </div>
                  <div class="r-right" @click.stop>
                    <span class="r-next is-paused">已完成</span>
                    <button type="button" class="xb-btn-ghost is-danger" :disabled="busy" @click="care.remove(rule)">删除</button>
                  </div>
                </article>
              </section>
            </div>

            <!-- 轻量快捷模板条 -->
            <div class="tpl-strip">
              <span class="tpl-strip-label">快捷模板</span>
              <button
                v-for="item in reminderTemplates"
                :key="item.title"
                type="button"
                class="tpl-pill"
                :data-testid="`care-template-${item.kind}`"
                :disabled="item.kind === 'weather' && !weatherEnabled"
                @click="openFromTemplate(item.preset)"
              >
                ＋ {{ item.title }}
              </button>
            </div>
          </template>

          <!-- 空状态：展开 3 列场景模板卡 -->
          <section v-else class="care-empty">
            <span class="empty-icon"><Droplet :size="22" /></span>
            <h2>还没有设置提醒</h2>
            <p>在上方直接输入一句话即可秒建，或挑选一个常用模板开始；每条提醒都能单独设定静音、响铃或语音朗读。</p>
            <div class="tpl-grid">
              <button
                v-for="item in reminderTemplates"
                :key="item.title"
                type="button"
                class="tpl-card"
                :data-testid="`care-template-${item.kind}`"
                :disabled="item.kind === 'weather' && !weatherEnabled"
                @click="openFromTemplate(item.preset)"
              >
                <strong>{{ item.title }}</strong>
                <span>{{ item.desc }}</span>
                <small>默认方式：{{ item.tag }}</small>
              </button>
            </div>
          </section>
        </template>

        <!-- 3B. 定时任务 Tab：舒展卡片流 -->
        <template v-else>
          <template v-if="tasks.length">
            <section class="task-grid" aria-label="定时任务">
              <article
                v-for="rule in tasks"
                :key="rule.id"
                class="task-card"
                :data-testid="`care-rule-${rule.id}`"
              >
                <div class="task-top" @click="open('agent', rule)">
                  <span class="r-ico is-agent" aria-hidden="true"><Bot :size="17" /></span>
                  <div class="r-body">
                    <span class="r-title">{{ rule.text || '定时任务' }}</span>
                    <div class="r-sub">
                      <span>{{ describeCareRule(rule) }}</span>
                      <span>·</span>
                      <span>完成后：{{ describeRuleNotify(resolveRuleNotify(rule, status.config)) }}</span>
                    </div>
                  </div>
                  <div class="channel-strip" @click.stop>
                    <button type="button" class="ch-btn" :class="{ 'is-on': resolveRuleNotify(rule, status.config).sound }" :disabled="busy" title="完成后播放提示音" @click="toggleRuleChannel(rule, 'sound')"><Bell :size="13" /></button>
                    <button type="button" class="ch-btn" :class="{ 'is-on': resolveRuleNotify(rule, status.config).voice }" :disabled="busy" title="完成后语音播报摘要" @click="toggleRuleChannel(rule, 'voice')"><Volume2 :size="13" /></button>
                    <button type="button" class="ch-btn" :class="{ 'is-on-neutral': resolveRuleNotify(rule, status.config).sticky }" :disabled="busy" title="结果气泡常驻" @click="toggleRuleChannel(rule, 'sticky')"><Pin :size="13" /></button>
                  </div>
                  <div class="r-right" @click.stop>
                    <span class="r-next" :class="{ 'is-paused': !rule.enabled }" :data-testid="`care-next-${rule.id}`">{{ nextRunLabel(rule, now) }}</span>
                    <NSwitch size="small" :value="rule.enabled" :disabled="busy" :aria-label="`启用${rule.text}`" @update:value="care.toggle(rule, $event)" />
                    <div class="more-wrap">
                      <button type="button" class="row-more-btn is-visible" aria-label="更多操作" @click="toggleRowMenu(rule.id)"><Ellipsis :size="15" /></button>
                      <div v-if="openMenuId === rule.id" class="xb-menu r-menu" role="menu">
                        <button type="button" class="xb-menu-item" @click="openMenuId = null; open('agent', rule)">编辑任务</button>
                        <button type="button" class="xb-menu-item" @click="openMenuId = null; care.runAgentNow(rule)">立即试跑一次</button>
                        <button type="button" class="xb-menu-item" @click="openMenuId = null; care.preview(rule)">预览气泡样式</button>
                        <div class="xb-menu-sep" />
                        <button type="button" class="xb-menu-item is-danger" @click="handleDeleteClick(rule)">
                          {{ confirmDeleteId === rule.id ? '确认删除？' : '删除' }}
                        </button>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="task-run-box">
                  <div class="task-run-head">
                    <span class="run-dot" :class="runningAgentIds.includes(rule.id) ? 'is-running' : (status.agentRuns?.[rule.id]?.ok === false ? 'is-err' : 'is-ok')" />
                    <span v-if="runningAgentIds.includes(rule.id)">正在后台只读执行…</span>
                    <span v-else-if="status.agentRuns?.[rule.id]" :data-testid="`care-agent-run-${rule.id}`">{{ describeAgentRun(status.agentRuns[rule.id], now) }}</span>
                    <span v-else>尚未运行 · 到点自动执行或点击右侧试跑</span>
                    <div class="task-run-actions">
                      <button type="button" class="mini-link is-neutral" :disabled="busy || runningAgentIds.includes(rule.id)" data-testid="care-run-now" @click="care.runAgentNow(rule)">
                        <Play :size="12" />立即试跑
                      </button>
                      <button
                        v-if="status.agentRuns?.[rule.id]?.session"
                        type="button"
                        class="mini-link"
                        data-testid="care-open-chat"
                        @click="openAgentSession(status.agentRuns[rule.id].session)"
                      >
                        查看完整对话 <ArrowUpRight :size="12" />
                      </button>
                    </div>
                  </div>
                  <p v-if="status.agentRuns?.[rule.id]?.summary" class="task-run-summary">{{ status.agentRuns[rule.id].summary }}</p>
                </div>
              </article>
            </section>

            <div class="tpl-strip">
              <span class="tpl-strip-label">快捷任务模板</span>
              <button
                v-for="item in taskTemplates"
                :key="item.title"
                type="button"
                class="tpl-pill"
                @click="openFromTemplate(item.preset)"
              >
                ＋ {{ item.title }}
              </button>
            </div>
          </template>

          <section v-else class="care-empty">
            <span class="empty-icon"><Bot :size="22" /></span>
            <h2>还没有定时任务</h2>
            <p>到点让小伴按你的说明自动整理信息，完整结果自动沉淀到聊天「定时任务」会话中。</p>
            <div class="tpl-grid">
              <button
                v-for="item in taskTemplates"
                :key="item.title"
                type="button"
                class="tpl-card"
                @click="openFromTemplate(item.preset)"
              >
                <strong>{{ item.title }}</strong>
                <span>{{ item.desc }}</span>
                <small>默认方式：{{ item.tag }}</small>
              </button>
            </div>
          </section>
        </template>

        <footer class="care-footnote">
          <span>{{ status.pendingCount ? `有 ${status.pendingCount} 条待展示提醒 · ` : '' }}按本机时间运行，游戏、免打扰或锁屏时自动暂缓</span>
          <span
            class="care-footnote-tip"
            title="普通提醒显示约 10 秒，语音播报显示约 30 秒，常驻强提醒持续显示直到手动关闭或稍后 10 分钟提醒；暂缓期间只补发 30 分钟内仍有效的摘要；应用完全退出后停止调度。"
          >运行规则</span>
        </footer>
      </div>
    </div>

    <CareEditorDialog
      v-model:show="editing"
      v-model:city-query="cityQuery"
      :draft="draft"
      :config="status.config"
      :weather-enabled="weatherEnabled"
      :activity-available="status.activityAvailable"
      :busy="busy"
      :cities="cities"
      :search-done="searchDone"
      :locating="locating"
      :located="located"
      @change-kind="changeKind"
      @search-cities="searchCities"
      @select-city="draft.city = clone($event); cities = []"
      @locate-city="prefillCity(true)"
      @check-weather="checkWeather"
      @preview="care.preview(draft)"
      @save="care.save"
    />
  </main>
</template>

<style scoped>
.care-page {
  --c-blue-bg: #EBF3FC;   --c-blue-fg: #1F6AA8;
  --c-green-bg: #ECF6EA;  --c-green-fg: #2E7A3E;
  --c-purple-bg: #F2EEFC; --c-purple-fg: #5E43B8;
  --c-amber-bg: #FEF3E2;  --c-amber-fg: #A35B0A;
  --c-indigo-bg: #ECF0FC; --c-indigo-fg: #3451B2;
  display: flex;
  flex-direction: column;
  height: 100%;
  min-width: 0;
  padding: 0;
  container: remind-stage / inline-size;
}
html[data-theme="dark"] .care-page {
  --c-blue-bg: #1D2D42;   --c-blue-fg: #8FC1F2;
  --c-green-bg: #1E3526;  --c-green-fg: #8CD69F;
  --c-purple-bg: #2B2544; --c-purple-fg: #BDB0F4;
  --c-amber-bg: #3A2B16;  --c-amber-fg: #F2C07E;
  --c-indigo-bg: #202945; --c-indigo-fg: #A2B8FF;
}
.care-inner {
  width: 100%;
  max-width: 860px;
  margin: 0 auto;
  padding: 0 28px;
  box-sizing: border-box;
}
.care-head { flex: none; padding: 20px 0 12px; }
.care-head-row { display: flex; align-items: center; gap: 14px; }
.care-count { margin-left: 4px; font-size: 13px; font-weight: 500; color: var(--ui-text-tertiary); font-variant-numeric: tabular-nums; }
.care-head-actions { margin-left: auto; display: flex; align-items: center; gap: 8px; position: relative; }

.care-status-pill {
  height: 30px;
  padding: 0 10px;
  border: 0;
  border-radius: 999px;
  background: var(--ui-control);
  color: var(--ui-text-secondary);
  font: inherit;
  font-size: 12px;
  font-weight: 500;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  cursor: pointer;
  transition: background .14s, color .14s;
}
.care-status-pill:hover { background: var(--ui-control-hover); color: var(--ui-text-primary); }
.care-status-pill.is-quiet { background: var(--c-purple-bg); color: var(--c-purple-fg); }
.care-status-pill.is-paused { background: var(--c-amber-bg); color: var(--c-amber-fg); }
.care-status-dot { width: 6px; height: 6px; border-radius: 50%; background: currentColor; }

.care-icon-tool {
  width: 32px;
  height: 32px;
  border: 0;
  border-radius: 9px;
  background: transparent;
  color: var(--ui-text-secondary);
  display: grid;
  place-items: center;
  cursor: pointer;
  transition: background .14s, color .14s;
}
.care-icon-tool:hover, .care-icon-tool[aria-expanded="true"] { background: var(--ui-control); color: var(--ui-text-primary); }

/* 全局偏好浮层 */
.care-prefs-pop {
  position: absolute;
  right: 0;
  top: calc(100% + 8px);
  width: 312px;
  border-radius: 14px;
  background: var(--ui-surface-raised);
  backdrop-filter: blur(20px);
  box-shadow: var(--ui-shadow-dialog);
  border: 1px solid var(--ui-border);
  z-index: 45;
  overflow: hidden;
}
.cp-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 15px 9px;
  border-bottom: 1px solid var(--ui-line);
}
.cp-head strong { font-size: 13px; font-weight: 600; color: var(--ui-text-primary); }
.cp-saved { font-size: 11px; color: var(--ui-success-text); opacity: 0; transition: opacity .2s; }
.cp-saved.is-show { opacity: 1; }
.cp-sec {
  padding: 10px 15px;
  border-top: 1px solid var(--ui-line);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.cp-sec:first-of-type { border-top: 0; }
.cp-sec-label { font-size: 11px; font-weight: 600; color: var(--ui-text-tertiary); }
.cp-row { display: flex; align-items: center; justify-content: space-between; gap: 10px; min-height: 24px; }
.cp-row-text { display: flex; flex-direction: column; min-width: 0; }
.cp-row-text span, .cp-name { font-size: 12.5px; font-weight: 500; color: var(--ui-text-primary); }
.cp-row-text small { font-size: 11px; color: var(--ui-text-tertiary); }
.cp-time-pair { display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--ui-text-secondary); }
.cp-time-pair input[type="time"], .cp-select, .cp-input {
  border: 0;
  border-radius: 7px;
  padding: 0 9px;
  height: 30px;
  background: var(--ui-control);
  color: var(--ui-text-primary);
  font: inherit;
  font-size: 12px;
  outline: none;
}
.cp-time-pair input[type="time"] { flex: 1; height: 28px; }
.cp-select, .cp-input { width: 100%; box-sizing: border-box; }
.cp-save-btn { align-self: flex-end; height: 28px; font-size: 12px; }
.cp-note { margin: 0; font-size: 11px; line-height: 1.45; color: var(--ui-text-tertiary); }

.care-scroll { flex: 1; min-height: 0; overflow-y: auto; padding: 4px 0 28px; }
.care-stack { display: flex; flex-direction: column; gap: 18px; }

/* 快建栏 */
.care-qc {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px 12px 10px 14px;
  border-radius: 14px;
  background: var(--ui-surface-sunken);
  border: 1px solid var(--ui-border);
  transition: border-color .16s, box-shadow .16s, background .16s;
}
.care-qc:focus-within {
  background: var(--ui-surface);
  border-color: var(--ui-border-strong);
  box-shadow: 0 6px 20px rgba(0, 0, 0, .05);
}
.qc-row { display: flex; align-items: center; gap: 10px; }
.qc-ico { color: var(--ui-ink); flex: none; }
.qc-input {
  flex: 1;
  min-width: 0;
  border: 0;
  background: transparent;
  outline: none;
  font: inherit;
  font-size: 13.5px;
  color: var(--ui-text-primary);
  padding: 4px 0;
}
.qc-input::placeholder { color: var(--ui-text-tertiary); }
.qc-actions { display: flex; align-items: center; gap: 4px; flex: none; }
.qc-ch {
  width: 28px;
  height: 28px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: var(--ui-text-tertiary);
  display: grid;
  place-items: center;
  cursor: pointer;
  transition: background .13s, color .13s;
}
.qc-ch:hover { background: var(--ui-control); color: var(--ui-text-primary); }
.qc-ch[aria-pressed="true"] { background: var(--ui-soft); color: var(--ui-ink); }
.qc-submit {
  height: 28px;
  padding: 0 11px;
  border: 0;
  border-radius: 7px;
  background: var(--ui-text-primary);
  color: var(--ui-surface);
  font: inherit;
  font-size: 12px;
  font-weight: 500;
  margin-left: 4px;
  cursor: pointer;
}
.qc-submit:disabled { opacity: .4; cursor: not-allowed; }
.qc-parsed {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  padding-top: 6px;
  border-top: 1px solid var(--ui-line);
  font-size: 11.5px;
  color: var(--ui-text-secondary);
}
.qc-tag {
  height: 21px;
  padding: 0 8px;
  border-radius: 6px;
  background: var(--ui-soft);
  color: var(--ui-ink);
  font-weight: 500;
  display: inline-flex;
  align-items: center;
}
.qc-tag.is-neutral { background: var(--ui-control); color: var(--ui-text-secondary); }
.qc-enter { margin-left: auto; color: var(--ui-text-tertiary); }

/* 焦点卡 */
.care-spotlight {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 15px 18px;
  border-radius: 16px;
  background: linear-gradient(135deg, var(--ui-soft) 0%, var(--ui-surface-sunken) 65%);
  border: 1px solid var(--ui-border);
}
.care-spotlight.is-quiet { background: var(--c-purple-bg); border-color: transparent; }
.care-spotlight.is-paused { background: var(--c-amber-bg); border-color: transparent; }
.spot-clock {
  display: flex;
  flex-direction: column;
  min-width: 82px;
  flex: none;
  padding-right: 16px;
  border-right: 1px solid var(--ui-line);
}
.spot-clock strong {
  font-size: 24px;
  font-weight: 700;
  line-height: 1.1;
  color: var(--ui-ink);
  font-variant-numeric: tabular-nums;
}
.spot-clock span { font-size: 11.5px; color: var(--ui-text-secondary); margin-top: 3px; }
.spot-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 3px; }
.spot-eyebrow { font-size: 11.5px; font-weight: 600; color: var(--ui-ink); }
.care-spotlight.is-quiet .spot-eyebrow { color: var(--c-purple-fg); }
.care-spotlight.is-paused .spot-eyebrow { color: var(--c-amber-fg); }
.spot-title { font-size: 15px; font-weight: 600; color: var(--ui-text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.spot-meta { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; font-size: 12px; color: var(--ui-text-secondary); }
.spot-actions { display: flex; align-items: center; gap: 6px; flex: none; }
.spot-btn-soft, .spot-btn-ghost {
  height: 30px;
  padding: 0 11px;
  border: 0;
  border-radius: 8px;
  font: inherit;
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;
}
.spot-btn-soft { background: var(--ui-surface); color: var(--ui-text-primary); box-shadow: var(--ui-shadow-card); }
.spot-btn-soft:hover { background: var(--ui-surface-alt); }
.spot-btn-ghost { background: transparent; color: var(--ui-text-secondary); }
.spot-btn-ghost:hover { background: var(--ui-control); color: var(--ui-text-primary); }

/* 筛选条与分组 */
.care-filters { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
.filter-chip {
  height: 27px;
  padding: 0 11px;
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: var(--ui-text-secondary);
  font: inherit;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
}
.filter-chip:hover { background: var(--ui-control); color: var(--ui-text-primary); }
.filter-chip[aria-pressed="true"] { background: var(--ui-text-primary); color: var(--ui-surface); }
.filter-hint { margin-left: auto; font-size: 11.5px; color: var(--ui-text-tertiary); }

.care-group-block { display: flex; flex-direction: column; gap: 8px; }
.care-group-title { padding: 0 4px; font-size: 12px; font-weight: 600; color: var(--ui-text-tertiary); }
.care-list { overflow: visible; }
.care-filter-empty { padding: 28px; text-align: center; color: var(--ui-text-tertiary); font-size: 13px; }

/* 提醒行 */
.r-row {
  position: relative;
  display: flex;
  align-items: center;
  gap: 13px;
  padding: 13px 16px;
  border-top: 1px solid var(--ui-line);
  cursor: pointer;
  transition: background .13s;
}
.r-row:first-child { border-top: 0; border-radius: 13px 13px 0 0; }
.r-row:last-child { border-radius: 0 0 13px 13px; }
.r-row:only-child { border-radius: 13px; }
.r-row:hover { background: var(--ui-surface-hover); }
.r-row.is-muted .r-ico { opacity: .52; filter: saturate(.45); }
.r-row.is-muted .r-title { color: var(--ui-text-secondary); }

.r-ico {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  display: grid;
  place-items: center;
  flex: none;
}
.r-ico.is-custom   { background: var(--c-blue-bg);   color: var(--c-blue-fg); }
.r-ico.is-rest     { background: var(--c-green-bg);  color: var(--c-green-fg); }
.r-ico.is-off-work { background: var(--c-purple-bg); color: var(--c-purple-fg); }
.r-ico.is-weather  { background: var(--c-amber-bg);  color: var(--c-amber-fg); }
.r-ico.is-agent    { background: var(--c-indigo-bg); color: var(--c-indigo-fg); }

.r-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 3px; }
.r-title { font-size: 14px; font-weight: 500; color: var(--ui-text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.r-sub { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; font-size: 12px; color: var(--ui-text-secondary); }
.weather-pill {
  padding: 1px 7px;
  border-radius: 999px;
  background: var(--c-amber-bg);
  color: var(--c-amber-fg);
  font-size: 11px;
  font-weight: 500;
}
.snooze-tag {
  padding: 1px 6px;
  border-radius: 5px;
  background: var(--ui-soft);
  color: var(--ui-ink);
  font-size: 11px;
}

/* 行内单条触达方式微标 */
.channel-strip {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 2px;
  border-radius: 8px;
  flex: none;
  transition: background .14s;
}
.r-row:hover .channel-strip, .task-card:hover .channel-strip, .channel-strip:focus-within { background: var(--ui-control); }
.ch-btn {
  width: 24px;
  height: 24px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--ui-text-tertiary);
  display: grid;
  place-items: center;
  cursor: pointer;
  opacity: 0;
  transition: opacity .13s, background .13s, color .13s;
}
.r-row:hover .ch-btn, .task-card:hover .ch-btn, .channel-strip:focus-within .ch-btn { opacity: .65; }
.ch-btn:hover { opacity: 1 !important; background: var(--ui-surface); color: var(--ui-text-primary); }
.ch-btn.is-on { opacity: 1 !important; color: var(--ui-ink); background: var(--ui-soft); }
.ch-btn.is-on-neutral { opacity: 1 !important; color: var(--ui-text-secondary); background: var(--ui-control); }

.r-right { display: flex; align-items: center; gap: 10px; flex: none; }
.r-next { font-size: 12px; color: var(--ui-text-secondary); white-space: nowrap; font-variant-numeric: tabular-nums; min-width: 68px; text-align: right; }
.r-next.is-paused { color: var(--ui-text-tertiary); }

.more-wrap { position: relative; }
.row-more-btn {
  width: 26px;
  height: 26px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: var(--ui-text-tertiary);
  display: grid;
  place-items: center;
  cursor: pointer;
  opacity: 0;
  transition: opacity .13s, background .13s, color .13s;
}
.r-row:hover .row-more-btn, .r-row:focus-within .row-more-btn, .r-row.menu-open .row-more-btn, .row-more-btn.is-visible { opacity: 1; }
.row-more-btn:hover, .r-row.menu-open .row-more-btn { background: var(--ui-control); color: var(--ui-text-primary); }
.r-menu { position: absolute; right: 0; top: calc(100% + 5px); min-width: 176px; z-index: 40; }

/* 定时任务卡片流 */
.task-grid { display: flex; flex-direction: column; gap: 12px; }
.task-card {
  border-radius: 14px;
  background: var(--ui-surface);
  border: 1px solid var(--ui-border);
  box-shadow: var(--ui-shadow-card);
  transition: border-color .15s;
}
.task-card:hover { border-color: var(--ui-border-strong); }
.task-top { display: flex; align-items: center; gap: 13px; padding: 14px 16px; cursor: pointer; }
.task-run-box {
  margin: 0 16px 14px 65px;
  padding: 10px 13px;
  border-radius: 10px;
  background: var(--ui-surface-sunken);
  border: 1px solid var(--ui-line);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.task-run-head { display: flex; align-items: center; gap: 7px; flex-wrap: wrap; font-size: 11.5px; color: var(--ui-text-tertiary); }
.run-dot { width: 6px; height: 6px; border-radius: 50%; flex: none; }
.run-dot.is-ok { background: var(--ui-success); }
.run-dot.is-err { background: var(--ui-warning); }
.run-dot.is-running { background: var(--ui-btn); animation: carePulse 1s infinite; }
@keyframes carePulse { 0%, 100% { opacity: 1; } 50% { opacity: .35; } }
.task-run-summary { margin: 0; font-size: 12.5px; line-height: 1.55; color: var(--ui-text-primary); }
.task-run-actions { margin-left: auto; display: flex; align-items: center; gap: 6px; }
.mini-link {
  height: 24px;
  padding: 0 8px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--ui-ink);
  font: inherit;
  font-size: 11.5px;
  font-weight: 500;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
}
.mini-link:hover:not(:disabled) { background: var(--ui-soft); }
.mini-link.is-neutral { color: var(--ui-text-secondary); }
.mini-link.is-neutral:hover:not(:disabled) { background: var(--ui-control); color: var(--ui-text-primary); }
.mini-link:disabled { opacity: .45; cursor: not-allowed; }

/* 已完成折叠 */
.done-bar { display: flex; align-items: center; gap: 8px; padding: 0 4px; }
.done-toggle, .done-clear {
  border: 0;
  background: transparent;
  font: inherit;
  font-size: 12px;
  color: var(--ui-text-tertiary);
  padding: 3px 6px;
  border-radius: 6px;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.done-toggle:hover { background: var(--ui-control); color: var(--ui-text-primary); }
.done-clear { margin-left: auto; }
.done-clear:hover { color: var(--ui-danger); background: var(--ui-control); }
.done-chev { transition: transform .14s; }
.done-chev.is-open { transform: rotate(90deg); }

/* 模板条 & 空状态 3 列卡 */
.tpl-strip { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; padding: 2px; }
.tpl-strip-label { font-size: 12px; color: var(--ui-text-tertiary); }
.tpl-pill {
  height: 29px;
  padding: 0 11px;
  border-radius: 999px;
  background: var(--ui-surface-sunken);
  border: 1px solid var(--ui-border);
  color: var(--ui-text-secondary);
  font: inherit;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
}
.tpl-pill:hover:not(:disabled) { border-color: var(--ui-border-strong); color: var(--ui-text-primary); background: var(--ui-surface); }
.tpl-pill:disabled { opacity: .45; cursor: not-allowed; }

.care-empty {
  padding: 36px 20px 18px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 8px;
}
.empty-icon {
  width: 46px;
  height: 46px;
  border-radius: 14px;
  background: var(--ui-soft);
  color: var(--ui-ink);
  display: grid;
  place-items: center;
  margin-bottom: 4px;
}
.care-empty h2 { margin: 0; font-size: 16px; font-weight: 600; color: var(--ui-text-primary); }
.care-empty p { margin: 0; max-width: 380px; font-size: 12.5px; color: var(--ui-text-secondary); }
.tpl-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 12px; width: 100%; margin-top: 14px; }
.tpl-card {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 6px;
  padding: 14px;
  border-radius: 13px;
  background: var(--ui-surface-sunken);
  border: 1px solid var(--ui-border);
  font: inherit;
  text-align: left;
  cursor: pointer;
  transition: background .15s, border-color .15s, transform .15s;
}
.tpl-card:hover:not(:disabled) { background: var(--ui-surface); border-color: var(--ui-border-strong); transform: translateY(-1px); }
.tpl-card:disabled { opacity: .45; cursor: not-allowed; }
.tpl-card strong { font-size: 13px; font-weight: 600; color: var(--ui-text-primary); }
.tpl-card span { font-size: 12px; color: var(--ui-text-secondary); line-height: 1.45; }
.tpl-card small { margin-top: auto; font-size: 11px; color: var(--ui-text-tertiary); }

.care-footnote {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 2px;
  font-size: 11.5px;
  color: var(--ui-text-tertiary);
}
.care-footnote-tip {
  margin-left: auto;
  cursor: help;
  border-bottom: 1px dashed var(--ui-border-strong);
}

@container remind-stage (max-width: 660px) {
  .care-inner { padding: 0 16px; }
  .care-spotlight { flex-wrap: wrap; gap: 12px; }
  .spot-clock { border-right: 0; padding-right: 0; flex-direction: row; align-items: baseline; gap: 8px; }
  .task-run-box { margin-left: 16px; }
  .tpl-grid { grid-template-columns: minmax(0, 1fr); }
  .row-more-btn { opacity: 1; }
  .filter-hint { display: none; }
}
</style>
