<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue';
import { Bell, Info, Pin, Volume2 } from 'lucide-vue-next';
import {
  careTextLimit, dayNames, resolveRuleNotify,
  type CareCity, type CareConfig, type CareKind, type CareRepeat, type CareRule,
} from '../../care-model';
import { formatLocalDateTime } from './schedule';

const props = defineProps<{
  show: boolean;
  draft: CareRule;
  config: CareConfig;
  weatherEnabled: boolean;
  activityAvailable: boolean;
  busy: boolean;
  cityQuery: string;
  cities: CareCity[];
  searchDone: boolean;
  locating: boolean;
  located: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void;
  (e: 'update:cityQuery', value: string): void;
  (e: 'change-kind', kind: CareKind): void;
  (e: 'search-cities'): void;
  (e: 'select-city', city: CareCity): void;
  (e: 'locate-city'): void;
  (e: 'check-weather'): void;
  (e: 'preview'): void;
  (e: 'save'): void;
}>();

const kindOptions: { value: CareKind; label: string }[] = [
  { value: 'custom', label: '常规' },
  { value: 'rest', label: '休息' },
  { value: 'off-work', label: '下班' },
  { value: 'weather', label: '天气' },
  { value: 'agent', label: '定时任务' },
];

const repeatOptions: { value: CareRepeat; label: string }[] = [
  { value: 'once', label: '仅一次' },
  { value: 'daily', label: '每天' },
  { value: 'weekdays', label: '指定星期' },
  { value: 'interval', label: '时段间隔' },
];

const dayShorts = ['一', '二', '三', '四', '五', '六', '日'];

const effectiveNotify = computed(() => resolveRuleNotify(props.draft, props.config));
const textLimit = computed(() => careTextLimit(props.draft.kind));
const textCount = computed(() => Array.from(props.draft.text || '').length);

const titleText = computed(() => {
  if (props.draft.id) return props.draft.kind === 'agent' ? '编辑定时任务' : '编辑提醒';
  return props.draft.kind === 'agent' ? '新建定时任务' : '新建提醒';
});

function setRepeat(repeat: CareRepeat) {
  props.draft.repeat = repeat;
  if (repeat === 'once' && !props.draft.onceAt) {
    props.draft.onceAt = formatLocalDateTime(new Date(Date.now() + 60 * 60_000));
  }
  if ((repeat === 'weekdays' || repeat === 'interval') && !props.draft.weekdays.length) {
    props.draft.weekdays = [1, 2, 3, 4, 5];
  }
  if (repeat === 'interval') {
    props.draft.intervalEveryMinutes ||= 90;
    props.draft.windowStart ||= '09:00';
    props.draft.windowEnd ||= '18:00';
  }
}

function toggleWeekday(day: number) {
  const list = props.draft.weekdays;
  if (list.includes(day)) {
    if (props.draft.repeat === 'weekdays' && list.length <= 1) return;
    props.draft.weekdays = list.filter(item => item !== day);
  } else {
    props.draft.weekdays = [...list, day].sort((a, b) => a - b);
  }
}

function setOnceOffset(minutes: number) {
  props.draft.onceAt = formatLocalDateTime(new Date(Date.now() + minutes * 60_000));
}

function setOncePreset(which: 'tonight' | 'tomorrow') {
  const d = new Date();
  if (which === 'tomorrow') {
    d.setDate(d.getDate() + 1);
    d.setHours(9, 0, 0, 0);
  } else {
    d.setHours(20, 0, 0, 0);
    if (d.getTime() <= Date.now()) d.setDate(d.getDate() + 1);
  }
  props.draft.onceAt = formatLocalDateTime(d);
}

function toggleChannel(channel: 'sound' | 'voice' | 'sticky') {
  const current = effectiveNotify.value[channel];
  props.draft[channel] = !current;
}

function handleKeydown(event: KeyboardEvent) {
  if (props.show && event.key === 'Escape') {
    emit('update:show', false);
  }
}

onMounted(() => window.addEventListener('keydown', handleKeydown));
onUnmounted(() => window.removeEventListener('keydown', handleKeydown));
</script>

<template>
  <section
    v-if="show"
    class="care-editor-win"
    role="dialog"
    aria-modal="false"
    :aria-label="titleText"
    data-testid="care-dialog"
  >
    <header class="ce-chrome">
      <div class="ce-lights">
        <button type="button" class="ce-light is-close" aria-label="关闭窗口" @click="emit('update:show', false)">×</button>
        <span class="ce-light is-min" aria-hidden="true" />
        <span class="ce-light is-max" aria-hidden="true" />
      </div>
      <span class="ce-title">{{ titleText }}</span>
      <span class="ce-badge">{{ draft.id ? '编辑中' : '新建' }}</span>
    </header>

    <form class="ce-body" @submit.prevent="emit('save')">
      <!-- 1. 提醒什么 -->
      <div class="ce-card">
        <div class="ce-field">
          <span class="ce-label">类型</span>
          <div class="ce-kind-pills" data-testid="care-kind">
            <button
              v-for="item in kindOptions"
              :key="item.value"
              type="button"
              class="ce-kind-pill"
              :data-testid="`care-kind-${item.value}`"
              :aria-pressed="draft.kind === item.value"
              @click="emit('change-kind', item.value)"
            >
              {{ item.label }}
            </button>
          </div>
        </div>

        <div class="ce-field">
          <div class="ce-row">
            <label class="ce-label" for="care-editor-text">
              {{ draft.kind === 'agent' ? '要小伴做什么' : draft.kind === 'weather' ? '提醒开头（可选）' : '提醒内容' }}
            </label>
            <span class="ce-hint">{{ textCount }} / {{ textLimit }}</span>
          </div>
          <textarea
            id="care-editor-text"
            v-model="draft.text"
            class="ce-textarea"
            :maxlength="textLimit"
            :placeholder="draft.kind === 'agent' ? '写清要查看什么、希望得到什么结果…' : '写下想被提醒的话…'"
            data-testid="care-text"
          />
          <div v-if="draft.kind === 'agent'" class="ce-notice is-info" data-testid="care-agent-note">
            <Info :size="14" class="ce-notice-ico" />
            <span>到点后台自动执行，结果写入聊天「定时任务」会话并按下方方式通知你。执行时仅可只读查看工作目录与网页，不修改文件或运行命令；每次运行消耗模型额度。</span>
          </div>
        </div>

        <!-- 天气城市 -->
        <div v-if="draft.kind === 'weather'" class="ce-field" data-testid="care-weather-fields">
          <span class="ce-label">目标城市</span>
          <p v-if="!weatherEnabled" class="ce-warn">请先在右上角「偏好设置 → 天气来源」启用天气来源后再添加天气提醒。</p>
          <div class="ce-city-search">
            <input
              :value="cityQuery"
              type="text"
              class="ce-input"
              placeholder="输入城市名，例如 杭州 / Beijing"
              :disabled="!weatherEnabled"
              @input="emit('update:cityQuery', ($event.target as HTMLInputElement).value)"
              @keydown.enter.prevent="emit('search-cities')"
            />
            <button type="button" class="ce-btn-soft" :disabled="!weatherEnabled || busy" @click="emit('search-cities')">搜索</button>
          </div>
          <div v-if="cities.length" class="ce-chips">
            <button
              v-for="(city, index) in cities"
              :key="index"
              type="button"
              class="ce-chip"
              @click="emit('select-city', city)"
            >
              {{ [city.name, city.admin1, city.country].filter(Boolean).join(' · ') }}
            </button>
          </div>
          <p v-if="searchDone && !cities.length && !draft.city" class="ce-hint">没有匹配城市，可试英文名或加上省份。</p>
          <div class="ce-row ce-city-foot">
            <span class="ce-city-state" data-testid="care-city-state">
              {{ draft.city ? `已选择：${[draft.city.name, draft.city.admin1, draft.city.country].filter(Boolean).join(' · ')}` : locating ? '正在根据网络位置判断城市…' : '尚未选择城市' }}
            </span>
            <div class="ce-city-btns">
              <button type="button" class="ce-chip" :disabled="!weatherEnabled || busy || locating" data-testid="care-locate" @click="emit('locate-city')">
                {{ locating ? '定位中…' : '自动定位' }}
              </button>
              <button type="button" class="ce-chip" :disabled="!weatherEnabled || !draft.city || busy" @click="emit('check-weather')">
                查当前天气
              </button>
            </div>
          </div>
          <p v-if="located && draft.city" class="ce-hint" data-testid="care-city-located">已根据网络位置预先填好，不准确可以搜索其他城市。</p>
        </div>
      </div>

      <!-- 2. 什么时候 -->
      <div class="ce-card">
        <div v-if="draft.kind === 'rest'" class="ce-field" data-testid="care-rest-field">
          <span class="ce-label">有效使用电脑间隔</span>
          <div class="ce-inline">
            <span>累计</span>
            <input v-model.number="draft.intervalMinutes" type="number" class="ce-num" min="5" max="480" step="1" aria-label="休息间隔分钟数" />
            <span>分钟后提醒</span>
            <div class="ce-chips ce-inline-end">
              <button type="button" class="ce-chip" @click="draft.intervalMinutes = 20">20 分</button>
              <button type="button" class="ce-chip" @click="draft.intervalMinutes = 45">45 分</button>
              <button type="button" class="ce-chip" @click="draft.intervalMinutes = 60">60 分</button>
            </div>
          </div>
          <span class="ce-hint">
            锁屏、睡眠和连续空闲超过 1 分钟自动暂停累计。{{ activityAvailable ? '已连接系统空闲检测。' : '系统空闲检测不可用，休息计时暂不累计。' }}
          </span>
        </div>

        <template v-else>
          <div class="ce-field">
            <span class="ce-label">重复频率</span>
            <div class="ce-seg" data-testid="care-repeat">
              <button
                v-for="item in repeatOptions"
                v-show="!(draft.kind === 'agent' && item.value === 'interval')"
                :key="item.value"
                type="button"
                class="ce-seg-btn"
                :data-testid="`care-repeat-${item.value}`"
                :aria-pressed="draft.repeat === item.value"
                @click="setRepeat(item.value)"
              >
                {{ item.label }}
              </button>
            </div>
          </div>

          <div v-if="draft.repeat === 'once'" class="ce-field" data-testid="care-once-field">
            <span class="ce-label">日期与时间</span>
            <input v-model="draft.onceAt" type="datetime-local" class="ce-time ce-dt" aria-label="日期和时间" />
            <div class="ce-chips">
              <button type="button" class="ce-chip" @click="setOnceOffset(15)">15 分钟后</button>
              <button type="button" class="ce-chip" @click="setOnceOffset(60)">1 小时后</button>
              <button type="button" class="ce-chip" @click="setOncePreset('tonight')">今晚 20:00</button>
              <button type="button" class="ce-chip" @click="setOncePreset('tomorrow')">明天 09:00</button>
            </div>
          </div>

          <div v-else-if="draft.repeat === 'interval'" class="ce-field" data-testid="care-interval-field">
            <span class="ce-label">周期与生效时段</span>
            <div class="ce-inline">
              <span>每</span>
              <input v-model.number="draft.intervalEveryMinutes" type="number" class="ce-num" min="15" max="360" step="5" aria-label="间隔分钟数" />
              <span>分钟，在</span>
              <input v-model="draft.windowStart" type="time" class="ce-time" aria-label="时段开始时间" />
              <span>至</span>
              <input v-model="draft.windowEnd" type="time" class="ce-time" aria-label="时段结束时间" />
            </div>
          </div>

          <div v-else class="ce-field" data-testid="care-time-field">
            <span class="ce-label">提醒时间</span>
            <div class="ce-inline">
              <input v-model="draft.time" type="time" class="ce-time" aria-label="提醒时间" />
              <div class="ce-chips">
                <button type="button" class="ce-chip" @click="draft.time = '09:00'">09:00</button>
                <button type="button" class="ce-chip" @click="draft.time = '15:00'">15:00</button>
                <button type="button" class="ce-chip" @click="draft.time = '18:30'">18:30</button>
              </div>
            </div>
          </div>

          <div v-if="draft.repeat === 'weekdays' || draft.repeat === 'interval'" class="ce-field" data-testid="care-weekdays-field">
            <div class="ce-row">
              <span class="ce-label">重复星期</span>
              <div class="ce-presets">
                <button type="button" class="ce-link" @click="draft.weekdays = [1, 2, 3, 4, 5]">工作日</button>
                <button type="button" class="ce-link" @click="draft.weekdays = [6, 7]">周末</button>
                <button type="button" class="ce-link" @click="draft.weekdays = [1, 2, 3, 4, 5, 6, 7]">每天</button>
              </div>
            </div>
            <div class="ce-days">
              <button
                v-for="(label, idx) in dayShorts"
                :key="idx"
                type="button"
                class="ce-day"
                :aria-label="dayNames[idx]"
                :aria-pressed="draft.weekdays.includes(idx + 1)"
                @click="toggleWeekday(idx + 1)"
              >
                {{ label }}
              </button>
            </div>
          </div>
        </template>
      </div>

      <!-- 3. 这条提醒的方式（按条绑定） -->
      <div class="ce-card">
        <div class="ce-field">
          <div class="ce-row">
            <span class="ce-label">这条提醒的方式</span>
            <span class="ce-hint">默认弹出桌面气泡，按需叠加</span>
          </div>
          <div class="ce-delivery-grid">
            <button
              type="button"
              class="ce-deliv-card"
              data-testid="care-channel-sound"
              :aria-pressed="effectiveNotify.sound"
              @click="toggleChannel('sound')"
            >
              <div class="ce-deliv-top">
                <Bell :size="15" />
                <span>{{ effectiveNotify.sound ? '开启' : '关闭' }}</span>
              </div>
              <strong>提示音</strong>
              <small>到点轻响一声</small>
            </button>

            <button
              type="button"
              class="ce-deliv-card"
              data-testid="care-channel-voice"
              :aria-pressed="effectiveNotify.voice"
              title="使用「设置 → 声音」的音色朗读；选了在线音色时，朗读会把提醒文字发给在线语音服务，失败时自动回退系统朗读"
              @click="toggleChannel('voice')"
            >
              <div class="ce-deliv-top">
                <Volume2 :size="15" />
                <span>{{ effectiveNotify.voice ? '开启' : '关闭' }}</span>
              </div>
              <strong>语音朗读</strong>
              <small>用小伴音色念出</small>
            </button>

            <button
              type="button"
              class="ce-deliv-card"
              data-testid="care-channel-sticky"
              :aria-pressed="effectiveNotify.sticky"
              @click="toggleChannel('sticky')"
            >
              <div class="ce-deliv-top">
                <Pin :size="15" />
                <span>{{ effectiveNotify.sticky ? '常驻' : '自动收起' }}</span>
              </div>
              <strong>常驻强提醒</strong>
              <small>手动关闭前不消失</small>
            </button>
          </div>
        </div>
      </div>
    </form>

    <footer class="ce-foot">
      <button type="button" class="ce-btn-soft" :disabled="busy" @click="emit('preview')">试看气泡</button>
      <button type="button" class="ce-btn-ghost ce-foot-cancel" data-testid="care-cancel" @click="emit('update:show', false)">取消</button>
      <button type="button" class="xb-btn-primary" :disabled="busy" data-testid="care-submit" @click="emit('save')">
        {{ draft.id ? '保存修改' : '创建' }}
      </button>
    </footer>
  </section>
</template>

<style scoped>
.care-editor-win {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: min(520px, calc(100vw - 32px));
  max-height: min(710px, calc(100dvh - 44px));
  border-radius: 16px;
  background: var(--ui-configuration-canvas);
  box-shadow: var(--ui-shadow-dialog);
  border: 1px solid var(--ui-border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  z-index: 60;
}
.ce-chrome {
  flex: none;
  height: 44px;
  padding: 0 14px;
  display: flex;
  align-items: center;
  gap: 10px;
  background: var(--ui-surface);
  border-bottom: 1px solid var(--ui-line);
  user-select: none;
}
.ce-lights { display: flex; align-items: center; gap: 7px; }
.ce-light {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 0;
  padding: 0;
  display: grid;
  place-items: center;
  color: transparent;
  font-size: 9px;
  font-weight: 700;
}
.ce-light.is-close { background: var(--ui-window-close); cursor: pointer; }
.ce-lights:hover .ce-light.is-close { color: rgba(0, 0, 0, .55); }
.ce-light.is-min { background: var(--ui-window-minimize); }
.ce-light.is-max { background: var(--ui-window-expand); }
.ce-title { font-size: 13.5px; font-weight: 600; color: var(--ui-text-primary); }
.ce-badge {
  margin-left: auto;
  font-size: 11px;
  color: var(--ui-text-tertiary);
  padding: 2px 7px;
  border-radius: 6px;
  background: var(--ui-control);
}
.ce-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.ce-card {
  border-radius: 12px;
  background: var(--ui-surface);
  border: 1px solid var(--ui-border);
  overflow: hidden;
}
.ce-field {
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  border-top: 1px solid var(--ui-line);
}
.ce-field:first-child { border-top: 0; }
.ce-row { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
.ce-label { font-size: 12px; font-weight: 500; color: var(--ui-text-secondary); }
.ce-hint { margin: 0; font-size: 11.5px; color: var(--ui-text-tertiary); line-height: 1.45; }
.ce-warn {
  margin: 0;
  padding: 8px 10px;
  border-radius: 8px;
  background: rgba(224, 154, 43, .12);
  color: var(--ui-warning-text);
  font-size: 12px;
}
.ce-kind-pills { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 5px; }
.ce-kind-pill {
  height: 30px;
  border: 0;
  border-radius: 8px;
  background: var(--ui-control);
  color: var(--ui-text-secondary);
  font: inherit;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: background .14s, color .14s;
}
.ce-kind-pill:hover { background: var(--ui-control-hover); color: var(--ui-text-primary); }
.ce-kind-pill[aria-pressed="true"] { background: var(--ui-soft); color: var(--ui-ink); font-weight: 600; }

.ce-seg {
  display: flex;
  gap: 3px;
  padding: 3px;
  border-radius: 9px;
  background: var(--ui-control);
}
.ce-seg-btn {
  flex: 1;
  height: 27px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: var(--ui-text-secondary);
  font: inherit;
  font-size: 12px;
  cursor: pointer;
  transition: background .14s, color .14s;
}
.ce-seg-btn[aria-pressed="true"] {
  background: var(--ui-surface);
  color: var(--ui-text-primary);
  font-weight: 600;
  box-shadow: 0 1px 2px rgba(0, 0, 0, .1);
}
html[data-theme="dark"] .ce-seg-btn[aria-pressed="true"] { background: #48484C; }

.ce-input, .ce-textarea, .ce-time, .ce-num {
  border: 0;
  border-radius: 8px;
  padding: 7px 10px;
  background: var(--ui-control);
  color: var(--ui-text-primary);
  font: inherit;
  font-size: 13px;
  outline: none;
}
.ce-input, .ce-textarea { width: 100%; box-sizing: border-box; }
.ce-textarea { resize: vertical; min-height: 70px; line-height: 1.5; }
.ce-num { width: 72px; font-variant-numeric: tabular-nums; }
.ce-dt { width: 224px; }
.ce-inline { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; font-size: 13px; color: var(--ui-text-secondary); }
.ce-inline-end { margin-left: auto; }
.ce-chips { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
.ce-chip {
  height: 25px;
  padding: 0 9px;
  border: 0;
  border-radius: 7px;
  background: var(--ui-control);
  color: var(--ui-text-secondary);
  font: inherit;
  font-size: 11.5px;
  cursor: pointer;
  transition: background .13s, color .13s;
}
.ce-chip:hover:not(:disabled) { background: var(--ui-soft); color: var(--ui-ink); }
.ce-chip:disabled { opacity: .45; cursor: not-allowed; }

.ce-city-search { display: flex; gap: 8px; }
.ce-city-foot { flex-wrap: wrap; gap: 8px; }
.ce-city-state { font-size: 12.5px; color: var(--ui-text-primary); }
.ce-city-btns { display: flex; gap: 6px; }

.ce-presets { display: flex; gap: 4px; }
.ce-link {
  border: 0;
  background: transparent;
  color: var(--ui-ink);
  font: inherit;
  font-size: 11.5px;
  padding: 2px 6px;
  border-radius: 5px;
  cursor: pointer;
}
.ce-link:hover { background: var(--ui-soft); }
.ce-days { display: flex; gap: 6px; flex-wrap: wrap; }
.ce-day {
  width: 34px;
  height: 30px;
  border: 0;
  border-radius: 8px;
  background: var(--ui-control);
  color: var(--ui-text-secondary);
  font: inherit;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  display: grid;
  place-items: center;
}
.ce-day[aria-pressed="true"] { background: var(--ui-btn); color: #fff; }

.ce-notice {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 9px 11px;
  border-radius: 9px;
  background: var(--ui-soft);
  color: var(--ui-ink);
  font-size: 11.5px;
  line-height: 1.5;
}
.ce-notice-ico { flex: none; margin-top: 2px; }

.ce-delivery-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; }
.ce-deliv-card {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  padding: 10px 11px;
  border-radius: 10px;
  background: var(--ui-control);
  color: var(--ui-text-secondary);
  border: 1.5px solid transparent;
  font: inherit;
  text-align: left;
  cursor: pointer;
  transition: background .14s, color .14s, border-color .14s;
}
.ce-deliv-card:hover { background: var(--ui-control-hover); color: var(--ui-text-primary); }
.ce-deliv-card[aria-pressed="true"] {
  background: var(--ui-soft);
  color: var(--ui-ink);
  border-color: var(--ui-btn);
}
.ce-deliv-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  font-size: 11px;
}
.ce-deliv-card strong { font-size: 12.5px; font-weight: 600; color: inherit; }
.ce-deliv-card small { font-size: 11px; opacity: .82; line-height: 1.35; }

.ce-foot {
  flex: none;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 11px 15px;
  background: var(--ui-surface);
  border-top: 1px solid var(--ui-line);
}
.ce-foot-cancel { margin-left: auto; }
.ce-btn-soft, .ce-btn-ghost {
  height: 32px;
  padding: 0 12px;
  border: 0;
  border-radius: 8px;
  font: inherit;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
}
.ce-btn-soft { background: var(--ui-control); color: var(--ui-text-primary); }
.ce-btn-soft:hover:not(:disabled) { background: var(--ui-control-hover); }
.ce-btn-ghost { background: transparent; color: var(--ui-text-secondary); }
.ce-btn-ghost:hover { background: var(--ui-control); color: var(--ui-text-primary); }
</style>
