<script setup lang="ts">
import { computed } from 'vue';
import { NModal, NSelect, NSwitch } from 'naive-ui';
import { X } from 'lucide-vue-next';
import WindowChrome from '../configuration/WindowChrome.vue';
import { dragConfigurationWindow, zoomConfigurationWindow } from '../configuration/windows.ts';

export interface GreetingSettings {
  enabled: boolean;
  frequency: string;
  contextEnabled?: boolean;
  calendarRegion?: 'CN' | null;
}
export interface EnvironmentContext {
  temporal?: { localDay: number; hour: number; minute: number };
  calendar?: { dayKind: string };
  weather?: { condition: string };
  activity?: { mode: string };
  task?: { state: string };
}

const props = defineProps<{
  show?: boolean;
  detached?: boolean;
  platform?: string;
  settings: GreetingSettings;
  ready: boolean;
  busy: boolean;
  environment: EnvironmentContext | null;
}>();
const emit = defineEmits<{
  'update:show': [boolean];
  change: [Partial<GreetingSettings>];
  inspect: [];
  close: [];
}>();

const calendarRegions = [
  { label: '未指定 · 不推断节假日', value: 'none' },
  { label: '中国大陆（含法定节假日与调休）', value: 'CN' },
];
const greetingModes = [
  { label: '安静 · 最多每天 1 次', value: 'quiet' },
  { label: '自然 · 最多每天 2 次', value: 'natural' },
  { label: '活泼 · 最多每天 4 次', value: 'lively' },
];
const freqNames: Record<string, string> = { quiet: '安静', natural: '自然', lively: '活泼' };
const dayNames: Record<string, string> = {
  workday: '工作日', rest_day: '休息日', holiday: '节假日', makeup_workday: '调休工作日', unknown: '地区日历未知',
};
const weatherNames: Record<string, string> = {
  clear: '晴', cloudy: '多云', fog: '雾', rain: '雨', snow: '雪', thunderstorm: '雷雨',
};
const activityNames: Record<string, string> = {
  available: '当前空闲', active: '使用中', busy: '专注忙碌', idle: '离开暂歇', away: '已离开', unknown: '活动未知',
};
const taskNames: Record<string, string> = {
  idle: '对话空闲', running: '任务执行中', waiting: '等待回应', paused: '任务已暂停', completed: '任务已完成',
};

const statusLabel = computed(() =>
  props.settings.enabled ? `已开启 · ${freqNames[props.settings.frequency] ?? '自然'}` : '默认关闭',
);

function formatDay(localDay: number) {
  const s = String(localDay);
  return s.length === 8 ? `${s.slice(0, 4)}-${s.slice(4, 6)}-${s.slice(6, 8)}` : s;
}

const environmentChips = computed<string[]>(() => {
  const value = props.environment;
  if (!value) return ['尚未读取'];
  const chips: string[] = [];
  if (value.temporal) {
    chips.push(`${formatDay(value.temporal.localDay)} ${String(value.temporal.hour).padStart(2, '0')}:${String(value.temporal.minute).padStart(2, '0')}`);
  }
  if (value.calendar) chips.push(dayNames[value.calendar.dayKind] || '地区日历未知');
  if (value.weather) chips.push(weatherNames[value.weather.condition] || '天气未知');
  else chips.push('天气无有效缓存');
  if (value.activity?.mode) chips.push(activityNames[value.activity.mode] || value.activity.mode);
  if (value.task?.state) chips.push(taskNames[value.task.state] || `任务：${value.task.state}`);
  return chips;
});

function closePanel() {
  emit('update:show', false);
  emit('close');
}
</script>

<template>
  <template v-if="detached">
    <section class="proactivity-shell is-detached" role="region" aria-label="主动陪伴" data-testid="pet-proactivity-settings">
      <WindowChrome :platform="platform" />
      <header class="pw-head" @mousedown="dragConfigurationWindow" @dblclick="zoomConfigurationWindow">
        <h2>主动陪伴</h2>
        <span class="pw-status" :class="{ off: !settings.enabled }"><i />{{ statusLabel }}</span>
      </header>
      <div class="pw-body">
        <div class="cfg-card">
          <div class="cfg-row" title="鼠标停在宠物身上时，偶尔轻声打个招呼；不自动朗读、不读取屏幕、不主动调用 AI" aria-description="鼠标停在宠物身上时，偶尔轻声打个招呼；不自动朗读、不读取屏幕、不主动调用 AI">
            <span class="cfg-lbl">悬停问候</span>
            <NSwitch :value="settings.enabled" :disabled="!ready || busy" data-testid="pet-proactivity-switch" @update:value="enabled => emit('change', { enabled })" />
          </div>
          <div class="cfg-row" title="限制每天主动打招呼的最大次数" aria-description="限制每天主动打招呼的最大次数">
            <span class="cfg-lbl">陪伴频率</span>
            <NSelect :value="settings.frequency" :options="greetingModes" :disabled="!ready || busy || !settings.enabled" size="small" class="pw-select" data-testid="pet-proactivity-frequency" @update:value="frequency => emit('change', { frequency })" />
          </div>
        </div>

        <div class="cfg-card">
          <div class="cfg-row" title="独立授权：让问候参考本地时间、日历、已有天气缓存与当前任务状态" aria-description="独立授权：让问候参考本地时间、日历、已有天气缓存与当前任务状态">
            <span class="cfg-lbl">环境上下文</span>
            <NSwitch :value="settings.contextEnabled ?? false" :disabled="!ready || busy" data-testid="pet-context-switch" @update:value="enabled => emit('change', { contextEnabled: enabled })" />
          </div>
          <div class="cfg-row" title="用于区分工作日、休息日、法定节假日与调休" aria-description="用于区分工作日、休息日、法定节假日与调休">
            <span class="cfg-lbl">节假日地区</span>
            <NSelect :value="settings.calendarRegion ?? 'none'" :options="calendarRegions" :disabled="!ready || busy || !settings.contextEnabled" size="small" class="pw-select" data-testid="pet-proactivity-calendar" @update:value="region => emit('change', { calendarRegion: region === 'CN' ? 'CN' : null })" />
          </div>
        </div>

        <div v-if="settings.contextEnabled" class="cfg-card context-card">
          <div class="context-head">
            <span>最近感知状态</span>
            <button type="button" class="xb-link" data-testid="pet-inspect-context" @click="emit('inspect')">刷新</button>
          </div>
          <div class="context-chips" data-testid="pet-context-summary">
            <span v-for="(chip, idx) in environmentChips" :key="idx" class="ctx-chip">{{ chip }}</span>
          </div>
        </div>
      </div>
      <footer class="pw-foot">
        <button type="button" class="xb-btn-primary is-sm" @click="closePanel">完成</button>
      </footer>
    </section>
  </template>

  <NModal v-else :show="!!show" :auto-focus="false" @update:show="v => emit('update:show', v)">
    <section class="proactivity-shell is-modal" role="dialog" aria-modal="true" aria-label="主动陪伴" data-testid="pet-proactivity-settings">
      <header class="pw-head">
        <h2>主动陪伴</h2>
        <span class="pw-status" :class="{ off: !settings.enabled }"><i />{{ statusLabel }}</span>
        <button type="button" class="xb-icon-btn pw-close" aria-label="关闭" @click="closePanel"><X :size="16" /></button>
      </header>
      <div class="pw-body">
        <div class="cfg-card">
          <div class="cfg-row" title="鼠标停在宠物身上时，偶尔轻声打个招呼；不自动朗读、不读取屏幕、不主动调用 AI" aria-description="鼠标停在宠物身上时，偶尔轻声打个招呼；不自动朗读、不读取屏幕、不主动调用 AI">
            <span class="cfg-lbl">悬停问候</span>
            <NSwitch :value="settings.enabled" :disabled="!ready || busy" data-testid="pet-proactivity-switch" @update:value="enabled => emit('change', { enabled })" />
          </div>
          <div class="cfg-row" title="限制每天主动打招呼的最大次数" aria-description="限制每天主动打招呼的最大次数">
            <span class="cfg-lbl">陪伴频率</span>
            <NSelect :value="settings.frequency" :options="greetingModes" :disabled="!ready || busy || !settings.enabled" size="small" class="pw-select" data-testid="pet-proactivity-frequency" @update:value="frequency => emit('change', { frequency })" />
          </div>
        </div>

        <div class="cfg-card">
          <div class="cfg-row" title="独立授权：让问候参考本地时间、日历、已有天气缓存与当前任务状态" aria-description="独立授权：让问候参考本地时间、日历、已有天气缓存与当前任务状态">
            <span class="cfg-lbl">环境上下文</span>
            <NSwitch :value="settings.contextEnabled ?? false" :disabled="!ready || busy" data-testid="pet-context-switch" @update:value="enabled => emit('change', { contextEnabled: enabled })" />
          </div>
          <div class="cfg-row" title="用于区分工作日、休息日、法定节假日与调休" aria-description="用于区分工作日、休息日、法定节假日与调休">
            <span class="cfg-lbl">节假日地区</span>
            <NSelect :value="settings.calendarRegion ?? 'none'" :options="calendarRegions" :disabled="!ready || busy || !settings.contextEnabled" size="small" class="pw-select" data-testid="pet-proactivity-calendar" @update:value="region => emit('change', { calendarRegion: region === 'CN' ? 'CN' : null })" />
          </div>
        </div>

        <div v-if="settings.contextEnabled" class="cfg-card context-card">
          <div class="context-head">
            <span>最近感知状态</span>
            <button type="button" class="xb-link" data-testid="pet-inspect-context" @click="emit('inspect')">刷新</button>
          </div>
          <div class="context-chips" data-testid="pet-context-summary">
            <span v-for="(chip, idx) in environmentChips" :key="idx" class="ctx-chip">{{ chip }}</span>
          </div>
        </div>
      </div>
      <footer class="pw-foot">
        <button type="button" class="xb-btn-primary is-sm" @click="closePanel">完成</button>
      </footer>
    </section>
  </NModal>
</template>

<style scoped>
.proactivity-shell{display:flex;flex-direction:column;background:var(--ui-configuration-canvas);color:var(--ui-text-primary);font-family:var(--ui-font-family)}
.proactivity-shell.is-detached{height:100%;width:100%;overflow:hidden}
.proactivity-shell.is-modal{width:492px;max-width:calc(100vw - 32px);max-height:calc(100vh - 48px);border-radius:var(--ui-radius-dialog);box-shadow:0 0 0 1px var(--ui-border),var(--ui-shadow-dialog);overflow:hidden}
.pw-head{flex:none;display:flex;align-items:center;gap:10px;padding:14px 20px 10px}
.is-detached .pw-head{padding-top:4px}
.pw-head h2{margin:0;font-size:17px;font-weight:600}
.pw-status{height:21px;padding:0 8px;border-radius:999px;background:var(--ui-configuration-control);color:var(--ui-text-secondary);font-size:11px;font-weight:500;display:inline-flex;align-items:center;gap:5px}
.pw-status i{width:5.5px;height:5.5px;border-radius:50%;background:var(--ui-success)}
.pw-status.off i{background:var(--ui-text-disabled)}
.pw-close{margin-left:auto}
.pw-body{flex:1;min-height:0;overflow-y:auto;padding:2px 20px 16px;display:flex;flex-direction:column;gap:12px}
.cfg-card{background:var(--ui-configuration-group);border-radius:12px;padding:2px 14px}
.cfg-row{display:flex;align-items:center;justify-content:space-between;gap:12px;min-height:46px;padding:8px 0;border-top:1px solid var(--ui-line)}
.cfg-row:first-child{border-top:0}
.cfg-lbl{font-size:13px;font-weight:500;color:var(--ui-text-primary)}
.pw-select{width:228px;flex:none}
.pw-select :deep(.n-base-selection){background:var(--ui-configuration-control);border-radius:8px}
.pw-select :deep(.n-base-selection__border),.pw-select :deep(.n-base-selection__state-border){border:0;box-shadow:none}
.context-card{padding:12px 14px;display:flex;flex-direction:column;gap:9px}
.context-head{display:flex;align-items:center;justify-content:space-between;font-size:12px;color:var(--ui-text-secondary)}
.context-chips{display:flex;flex-wrap:wrap;gap:6px}
.ctx-chip{height:24px;padding:0 9px;border-radius:7px;background:var(--ui-configuration-control);color:var(--ui-text-primary);font-size:11.5px;display:inline-flex;align-items:center}
.pw-foot{flex:none;display:flex;justify-content:flex-end;padding:10px 20px 14px;border-top:1px solid var(--ui-line)}
.pw-foot .xb-btn-primary{border:0;background:var(--ui-btn);box-shadow:none}
</style>
