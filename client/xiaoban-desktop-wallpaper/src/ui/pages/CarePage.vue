<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { NAlert, NCollapse, NCollapseItem, NForm, NFormItem, NInput, NInputNumber, NModal, NPopconfirm, NSelect, NSwitch } from 'naive-ui';
import { Bot, Coffee, CloudSun, Droplet, Moon, Plus } from 'lucide-vue-next';
import { invoke } from '../../bridge';
import { careNames, careTextLimit, dayNames, describeAgentRun, describeCareRule, newCareRule, weatherIsFresh, type CareCity, type CareKind, type CareRule, type CareStatus } from '../../care-model';
import type { ControlStore } from '../control-types';
import { clone, createCareState } from '../care/state';
import { nextRunLabel } from '../care/schedule';
const props = defineProps<{ store: ControlStore }>();
const care = createCareState(props.store.desktop, props.store.notify);
const { status, preferences, preferencesDirty, draft, editing, busy } = care;
const kinds = Object.entries(careNames).map(([value, label]) => ({ value, label }));
const repeats = [{ label: '每天', value: 'daily' }, { label: '指定星期', value: 'weekdays' }, { label: '仅一次', value: 'once' }];
const sources = [{ label: 'Open-Meteo · 默认，个人非商业使用', value: 'open-meteo' }, { label: '关闭 · 不访问天气网络', value: 'disabled' }, { label: '自己的兼容 HTTPS 来源', value: 'custom' }];
const weatherEnabled = computed(() => status.value.config.weatherSource.provider !== 'disabled');
const cityQuery = ref(''); const cities = ref<CareCity[]>([]); const searchDone = ref(false);
const locating = ref(false); const located = ref(false);

// Two top-level tabs. 关怀提醒 holds the reminder kinds; 定时任务 holds the agent kind (rules that run a task and report back).
const tab = ref<'reminders' | 'tasks'>('reminders');
const reminders = computed(() => status.value.rules.filter(rule => rule.kind !== 'agent'));
const tasks = computed(() => status.value.rules.filter(rule => rule.kind === 'agent'));
const now = ref(new Date());
let clockTimer: ReturnType<typeof setInterval> | undefined;
const kindIcon = { custom: Droplet, 'off-work': Moon, rest: Coffee, weather: CloudSun, agent: Bot } as const;
const templates: { kind: CareKind; hint: string }[] = [
  { kind: 'custom', hint: '每天固定时间，写下你想被提醒的话' },
  { kind: 'off-work', hint: '工作日下班时间，提醒收工休息' },
  { kind: 'rest', hint: '累计使用电脑一段时间后，提醒起来活动' },
  { kind: 'weather', hint: '每天按所选城市查询天气并提醒' },
  { kind: 'agent', hint: '到点让小伴查看文件并总结结果' },
];

function open(kind: CareKind = 'custom') { care.open(kind); cityQuery.value = ''; cities.value = []; searchDone.value = false; located.value = false; if (kind === 'weather') void prefillCity(); }
function changeKind(kind: CareKind) { const defaults = newCareRule(kind); draft.value = { ...draft.value, kind, text: defaults.text, time: defaults.time, city: null }; cities.value = []; located.value = false; if (kind === 'weather') void prefillCity(); }
// Fills the city from the network location. A city the user already chose is never replaced by the automatic one.
async function prefillCity(manual = false) {
  if (!weatherEnabled.value || locating.value || (!manual && draft.value.city)) return;
  if (!props.store.desktop) { if (manual) props.store.notify('自动定位需要桌面程序；浏览器预览不访问网络', 'info'); return; }
  locating.value = true;
  try {
    const city = await invoke<CareCity>('detect_care_city');
    if (editing.value && draft.value.kind === 'weather' && (manual || !draft.value.city)) { draft.value.city = clone(city); located.value = true; cities.value = []; }
  } catch (error) { if (manual) props.store.notify(error instanceof Error ? error.message : String(error), 'error'); }
  finally { locating.value = false; }
}
async function searchCities() {
  if (!weatherEnabled.value) return;
  if (!props.store.desktop) { props.store.notify('城市搜索需要桌面程序；浏览器预览不访问天气网络', 'info'); return; }
  if (!cityQuery.value.trim()) { props.store.notify('请输入城市名', 'error'); return; }
  await care.run(async () => { cities.value = await invoke<CareCity[]>('search_care_cities', { query: cityQuery.value.trim() }); searchDone.value = true; });
}
async function checkWeather() {
  if (!weatherEnabled.value || !draft.value.city) { props.store.notify('请先启用天气来源并选择城市', 'error'); return; }
  if (!props.store.desktop) { props.store.notify('天气查询需要桌面程序', 'info'); return; }
  await care.run(async () => { care.receive(await invoke<CareStatus>('check_care_weather', { city: clone(draft.value.city) })); props.store.notify('天气查询已完成', 'success'); });
}
onMounted(() => { void care.initialize(); clockTimer = setInterval(() => { now.value = new Date(); }, 30_000); });
onUnmounted(() => { if (clockTimer) clearInterval(clockTimer); care.dispose(); });
</script>

<template>
  <main class="xb-page care-page" data-testid="care-page">
    <header class="xb-page-head care-head" data-tauri-drag-region>
      <div class="xb-tabs-lg" role="tablist" aria-label="提醒类型">
        <button type="button" role="tab" data-testid="care-tab-reminders" :aria-selected="tab === 'reminders'" @click="tab = 'reminders'">关怀提醒 <span class="care-count">{{ reminders.length }}</span></button>
        <button type="button" role="tab" data-testid="care-tab-tasks" :aria-selected="tab === 'tasks'" @click="tab = 'tasks'">定时任务 <span class="care-count">{{ tasks.length }}</span></button>
      </div>
      <div class="xb-page-actions">
        <button v-if="tab === 'reminders'" type="button" class="xb-btn-primary" data-testid="care-add" @click="open()"><Plus :size="14" />新建</button>
        <button v-else type="button" class="xb-btn-primary" data-testid="care-add-task" @click="open('agent')"><Plus :size="14" />新建</button>
      </div>
    </header>

    <NAlert v-if="!store.desktop" type="info" :show-icon="false">浏览器预览：初始没有提醒。可临时添加并检查表单，本次预览不保存、不在后台触发。</NAlert>
    <NAlert v-if="status.error" type="error">{{ status.error }}</NAlert>

    <div class="care-layout">
      <div class="care-main">
        <template v-if="tab === 'reminders'">
          <p v-if="!weatherEnabled" class="xb-sub care-note">天气来源已关闭，暂不可搜索城市或添加天气提醒；本地提醒仍可使用。可在右侧“提醒方式 → 天气来源”重新开启。</p>
          <section v-if="reminders.length" class="xb-group" aria-label="我的提醒">
            <article v-for="rule in reminders" :key="rule.id" class="xb-row care-rule" :data-testid="`care-rule-${rule.id}`">
              <span class="care-ico" :class="`is-${rule.kind}`" aria-hidden="true"><component :is="kindIcon[rule.kind]" :size="18" /></span>
              <div class="xb-row-text">
                <b>{{ rule.text || '根据真实天气生成提示' }}</b>
                <span>{{ careNames[rule.kind] }} · {{ describeCareRule(rule) }}</span>
                <small v-if="status.agentRuns?.[rule.id]" class="xb-sub" :data-testid="`care-agent-run-${rule.id}`" :title="status.agentRuns[rule.id].summary">{{ describeAgentRun(status.agentRuns[rule.id]) }}，完整结果在聊天里</small>
                <small v-if="rule.notBefore > Date.now() / 1000" class="xb-sub">稍后提醒：{{ new Date(rule.notBefore * 1000).toLocaleString() }}</small>
              </div>
              <span class="care-next" :data-testid="`care-next-${rule.id}`">{{ nextRunLabel(rule, now) }}</span>
              <NSwitch class="care-switch" :value="rule.enabled" :disabled="busy" :aria-label="`启用${careNames[rule.kind]}提醒`" @update:value="care.toggle(rule, $event)" />
              <div class="care-rule-actions">
                <button type="button" class="xb-btn-ghost" :disabled="busy" @click="care.open(rule.kind, rule)">编辑</button>
                <button type="button" class="xb-btn-ghost" :disabled="busy" @click="care.preview(rule)">预览</button>
                <NPopconfirm @positive-click="care.remove(rule)">
                  <template #trigger><button type="button" class="xb-btn-ghost is-danger" :disabled="busy">删除</button></template>
                  删除这条提醒？待展示的任务也会取消。
                </NPopconfirm>
              </div>
            </article>
          </section>
          <div v-else class="xb-group xb-empty care-empty">
            <span class="xb-empty-icon"><Droplet :size="24" /></span>
            <h2>还没有提醒</h2>
            <p>从一条喝水或休息提醒开始吧，下面的模板可以直接用。</p>
          </div>

          <section class="care-templates" aria-label="试试这些">
            <h3 class="care-section-title">试试这些</h3>
            <div class="care-tpl-grid">
              <button v-for="item in templates" :key="item.kind" type="button" class="care-tpl" :data-testid="`care-template-${item.kind}`" :disabled="item.kind === 'weather' && !weatherEnabled" @click="open(item.kind)">
                <span class="care-ico" :class="`is-${item.kind}`" aria-hidden="true"><component :is="kindIcon[item.kind]" :size="16" /></span>
                <b>{{ careNames[item.kind] }}</b>
                <span>{{ item.hint }}</span>
              </button>
            </div>
          </section>

          <p class="xb-sub care-behavior">{{ status.pendingCount ? `有 ${status.pendingCount} 条待展示提醒。` : '' }}提醒按系统本地时间运行，显示在当前应用上方；游戏、免打扰或锁屏时暂缓，只补发 30 分钟内仍有效的摘要。普通提醒约显示 10 秒，语音提醒约显示 30 秒，可稍后 10 分钟提醒；应用完全退出后停止。{{ status.systemNotificationsAvailable ? '系统通知可用。' : '当前系统通知渠道未接入。' }}</p>
        </template>

        <template v-else>
          <section v-if="tasks.length" class="xb-group" aria-label="定时任务">
            <article v-for="rule in tasks" :key="rule.id" class="xb-row care-rule" :data-testid="`care-rule-${rule.id}`">
              <span class="care-ico is-agent" aria-hidden="true"><Bot :size="18" /></span>
              <div class="xb-row-text">
                <b>{{ rule.text || '定时任务' }}</b>
                <span>{{ careNames[rule.kind] }} · {{ describeCareRule(rule) }}</span>
                <small v-if="status.agentRuns?.[rule.id]" class="xb-sub" :data-testid="`care-agent-run-${rule.id}`" :title="status.agentRuns[rule.id].summary">{{ describeAgentRun(status.agentRuns[rule.id]) }}，完整结果在聊天里</small>
              </div>
              <span class="care-next" :data-testid="`care-next-${rule.id}`">{{ nextRunLabel(rule, now) }}</span>
              <NSwitch class="care-switch" :value="rule.enabled" :disabled="busy" :aria-label="`启用${careNames[rule.kind]}提醒`" @update:value="care.toggle(rule, $event)" />
              <div class="care-rule-actions">
                <button type="button" class="xb-btn-ghost" :disabled="busy" @click="care.open(rule.kind, rule)">编辑</button>
                <button type="button" class="xb-btn-ghost" :disabled="busy" @click="care.preview(rule)">预览</button>
                <NPopconfirm @positive-click="care.remove(rule)">
                  <template #trigger><button type="button" class="xb-btn-ghost is-danger" :disabled="busy">删除</button></template>
                  删除这条定时任务？待展示的任务也会取消。
                </NPopconfirm>
              </div>
            </article>
          </section>
          <div v-else class="xb-group xb-empty care-empty">
            <span class="xb-empty-icon"><Bot :size="24" /></span>
            <h2>还没有定时任务</h2>
            <p>到点让小伴按你的说明自动去做，结果显示在桌面提醒里，完整内容在聊天“定时任务”对话中。</p>
            <button type="button" class="xb-btn-primary" @click="open('agent')"><Plus :size="14" />新建定时任务</button>
          </div>
        </template>
      </div>

      <aside class="care-side">
        <section class="xb-group care-prefs" aria-label="提醒方式" @change="preferencesDirty = true">
          <h2 class="care-prefs-title">提醒方式</h2>
          <div class="xb-row"><div class="xb-row-text"><b>开启关怀提醒</b><span>关闭后暂停全部关怀提醒。</span></div><NSwitch v-model:value="preferences.enabled" @update:value="preferencesDirty = true" /></div>
          <div class="xb-row"><div class="xb-row-text"><b>提示音</b><span>所有提醒到点都响一声。</span></div><NSwitch v-model:value="preferences.sound" @update:value="preferencesDirty = true" /></div>
          <div class="xb-row"><div class="xb-row-text"><b>一次性提醒带提示音</b><span>只提醒一次的事不想错过，所以默认响一声，即使上面的提示音关着。</span></div><NSwitch v-model:value="preferences.onceSound" data-testid="care-once-sound" @update:value="preferencesDirty = true" /></div>
          <div class="xb-row"><div class="xb-row-text"><b>语音播报</b><span>使用“设置 → 声音”的音色；播报时会把提醒文案发送给在线语音服务。默认关闭。</span></div><NSwitch v-model:value="preferences.voice" data-testid="care-voice-switch" @update:value="preferencesDirty = true" /></div>
          <div class="xb-row"><div class="xb-row-text"><b>免打扰时段</b><span>支持跨午夜，期间的有效提醒会延后。</span></div><NSwitch v-model:value="preferences.quietEnabled" @update:value="preferencesDirty = true" /></div>
          <div v-if="preferences.quietEnabled" class="care-time-pair">
            <label>开始 <input v-model="preferences.quietStart" aria-label="免打扰开始" type="time"></label>
            <label>结束 <input v-model="preferences.quietEnd" aria-label="免打扰结束" type="time"></label>
          </div>
          <div class="care-weather-source">
            <NCollapse><NCollapseItem title="天气来源与网络设置" name="weather"><NForm label-placement="top"><NFormItem label="天气来源"><NSelect v-model:value="preferences.weatherSource.provider" :options="sources" @update:value="preferencesDirty = true" /></NFormItem><template v-if="preferences.weatherSource.provider === 'custom'"><NFormItem label="天气 API · HTTPS"><NInput v-model:value="preferences.weatherSource.forecastUrl" placeholder="https://example.com/v1/forecast" @update:value="preferencesDirty = true"/></NFormItem><NFormItem label="城市搜索 API · HTTPS"><NInput v-model:value="preferences.weatherSource.geocodingUrl" placeholder="https://example.com/v1/search" @update:value="preferencesDirty = true"/></NFormItem><p class="xb-sub">需兼容 Open-Meteo JSON 与参数；不在客户端填写密钥。</p></template><p class="xb-sub">天气查询只发送所选城市的坐标。Open-Meteo 免费来源限个人非商业使用；天气数据 CC BY 4.0，城市数据 GeoNames。新建天气提醒时会按网络位置预先填好城市，这一步会把你的公网 IP 发送给 ipwho.is（失败时用 ip.sb）来判断所在城市，不会发送其他信息；关闭天气来源后不再定位。</p></NForm></NCollapseItem></NCollapse>
          </div>
          <div class="care-save"><button type="button" class="xb-btn-primary" :disabled="busy" data-testid="care-save-preferences" @click="care.savePreferences">保存提醒偏好</button></div>
        </section>
        <section v-if="status.weather || status.weatherError" class="xb-group care-weather">
          <template v-if="status.weather"><h2>{{ weatherIsFresh(status.weather) ? '最近天气' : '天气已过期 · 历史记录' }}</h2><p>{{ status.weather.text }}</p><p class="xb-sub">查询 {{ new Date(status.weather.updatedAt * 1000).toLocaleString() }} · 模型 {{ new Date(status.weather.modelAt * 1000).toLocaleString() }} · {{ status.weather.source }}</p></template>
          <NAlert v-if="status.weatherError" type="warning">{{ status.weatherError }}</NAlert>
        </section>
      </aside>
    </div>

    <NModal v-model:show="editing" preset="card" :title="draft.id ? '编辑提醒' : '新建提醒'" class="care-dialog" style="width: min(560px, calc(100vw - 48px))" :mask-closable="false" data-testid="care-dialog"><NForm class="care-form" label-placement="top" @submit.prevent="care.save"><div class="xb-group care-form-group">
      <NFormItem label="提醒类型"><NSelect :value="draft.kind" :options="kinds" data-testid="care-kind" @update:value="changeKind"/></NFormItem><NFormItem label="启用这条提醒"><NSwitch v-model:value="draft.enabled" /></NFormItem>
      <NFormItem v-if="draft.kind === 'rest'" label="有效使用电脑的间隔（分钟）" data-testid="care-rest-field"><NInputNumber v-model:value="draft.intervalMinutes" :min="5" :max="480" :precision="0"/><template #feedback>锁屏、睡眠和连续空闲超过一分钟不累计。{{ status.activityAvailable ? '已连接系统空闲检测。' : '系统空闲检测不可用，休息计时暂不累计。' }}</template></NFormItem>
      <template v-else><NFormItem label="重复方式"><NSelect v-model:value="draft.repeat" :options="repeats" data-testid="care-repeat"/></NFormItem><NFormItem v-if="draft.repeat === 'once'" label="日期和时间" data-testid="care-once-field"><input v-model="draft.onceAt" type="datetime-local" aria-label="日期和时间"></NFormItem><NFormItem v-else label="提醒时间" data-testid="care-time-field"><input v-model="draft.time" type="time" aria-label="提醒时间"></NFormItem><NFormItem v-if="draft.repeat === 'weekdays'" label="提醒星期" data-testid="care-weekdays-field"><NSelect v-model:value="draft.weekdays" multiple :options="dayNames.map((label, i) => ({ label, value: i + 1 }))"/></NFormItem></template>
      <div v-if="draft.kind === 'weather'" data-testid="care-weather-fields"><NAlert v-if="!weatherEnabled" type="warning">请先在提醒偏好中启用并保存天气来源；关闭来源时不可搜索城市或添加天气提醒。</NAlert><NFormItem label="手动选择城市"><div class="city-search"><NInput v-model:value="cityQuery" placeholder="例如 北京 / Beijing" :disabled="!weatherEnabled"/><button type="button" class="xb-btn" :disabled="!weatherEnabled || busy" @click="searchCities">搜索</button></div></NFormItem><div class="city-results"><button v-for="(city, index) in cities" :key="index" type="button" class="xb-btn is-sm" @click="draft.city = clone(city); cities = []">{{ [city.name, city.admin1, city.country].filter(Boolean).join(' · ') }}</button></div><p v-if="searchDone && !cities.length && !draft.city" class="xb-sub">没有匹配城市，可试英文名或加上省份。</p><p class="care-city-state" data-testid="care-city-state">{{ draft.city ? `已选择：${[draft.city.name, draft.city.admin1, draft.city.country].filter(Boolean).join(' · ')}` : locating ? '正在根据网络位置判断城市…' : '尚未选择城市' }}</p><p v-if="located && draft.city" class="xb-sub" data-testid="care-city-located">已根据网络位置预先填好，不准确可以搜索其他城市。</p><div class="city-actions"><button type="button" class="xb-btn is-sm" :disabled="!weatherEnabled || busy || locating" data-testid="care-locate" @click="prefillCity(true)">{{ locating ? '定位中…' : '按网络位置填写' }}</button><button type="button" class="xb-btn is-sm" :disabled="!weatherEnabled || !draft.city || busy" @click="checkWeather">查询所选城市天气</button></div></div>
      <NAlert v-if="draft.kind === 'agent'" type="info" :show-icon="false" data-testid="care-agent-note">到时小伴会按说明自动去做，结果显示在桌面提醒里，完整内容在聊天“定时任务”对话中。你不在电脑前也会执行，所以小伴这时只能查看工作目录里的文件和打开网页，不会修改文件、运行命令或改动桌面；每次运行都会使用 AI 模型额度。</NAlert>
      <NFormItem :label="draft.kind === 'weather' ? '提醒文案（可选开头）' : draft.kind === 'agent' ? '要小伴做什么' : '提醒文案'"><NInput v-model:value="draft.text" type="textarea" :maxlength="careTextLimit(draft.kind)" show-count data-testid="care-text"/><template #feedback>{{ draft.kind === 'agent' ? '最多 1000 个字，写清要查看什么、希望得到什么结果。' : '最多 240 个字，文案仅保存在本机。' }}</template></NFormItem>
    </div><div class="dialog-actions"><button type="button" class="xb-btn" data-testid="care-cancel" @click="editing = false">取消</button><button type="button" class="xb-btn" :disabled="busy" @click="care.preview(draft)">预览</button><button type="submit" class="xb-btn-primary" :disabled="busy" data-testid="care-submit">{{ draft.id ? '保存修改' : '创建提醒' }}</button></div>
    </NForm></NModal>
  </main>
</template>

<style scoped>
/* Kind colours are local tokens so the icon squares follow light and dark mode. */
.care-page{--care-blue-bg:#E8F2FB;--care-blue-fg:#1F6AA8;--care-green-bg:#EDF5E8;--care-green-fg:#3C7A2C;--care-purple-bg:#F1EEFB;--care-purple-fg:#5E43B8;--care-orange-bg:#FFF2E2;--care-orange-fg:#A65A0A;display:flex;flex-direction:column;gap:18px;padding:22px 28px;min-width:0;container:care-page / inline-size}
html[data-theme=dark] .care-page{--care-blue-bg:#1E2F44;--care-blue-fg:#8FC1F2;--care-green-bg:#213A29;--care-green-fg:#9ED49A;--care-purple-bg:#2C2745;--care-purple-fg:#B7A8F0;--care-orange-bg:#3B2C17;--care-orange-fg:#F0BE7D}
.care-head{align-items:center}
.care-count{margin-left:4px;font-size:13px;font-weight:500;color:var(--ui-text-tertiary)}
.care-layout{display:grid;grid-template-columns:minmax(0,1fr) 280px;gap:20px;align-items:start}
.care-main{container:care-main / inline-size}
.care-main,.care-side{display:flex;flex-direction:column;gap:18px;min-width:0}
.care-note{margin:0}
.care-behavior{margin:0;font-size:12px;line-height:1.8}
.care-section-title{margin:0 4px 10px;font-size:13px;font-weight:600;color:var(--ui-text-primary)}
.care-rule{display:flex;align-items:center;gap:14px}
.care-rule .xb-row-text{gap:3px;overflow-wrap:anywhere}
.care-rule .xb-row-text>b{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.care-rule .xb-row-text>small{font-size:11.5px;line-height:1.5}
.care-next{flex-shrink:0;padding:3px 8px;border-radius:6px;background:var(--ui-soft);color:var(--ui-ink);font-size:12px;font-weight:500;white-space:nowrap}
.care-switch{flex-shrink:0}
.care-rule-actions{display:flex;align-items:center;gap:2px;flex-shrink:0}
.care-rule-actions .is-danger{color:var(--ui-danger)}
.care-ico{width:36px;height:36px;flex-shrink:0;border-radius:10px;display:grid;place-items:center;background:var(--ui-soft);color:var(--ui-ink)}
.care-ico.is-custom{background:var(--care-blue-bg);color:var(--care-blue-fg)}
.care-ico.is-rest{background:var(--care-green-bg);color:var(--care-green-fg)}
.care-ico.is-off-work{background:var(--care-purple-bg);color:var(--care-purple-fg)}
.care-ico.is-weather{background:var(--care-orange-bg);color:var(--care-orange-fg)}
.care-empty{padding:40px 24px}
.care-tpl-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:12px}
.care-tpl{display:flex;flex-direction:column;align-items:flex-start;gap:8px;padding:14px;border:0;border-radius:12px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-border),0 1px 2px rgba(0,0,0,.03);color:var(--ui-text-primary);text-align:left;font:inherit;cursor:pointer;transition:box-shadow 160ms}
.care-tpl:hover:not(:disabled){box-shadow:0 0 0 1px var(--ui-border-strong),0 6px 16px var(--ui-shadow-hover)}
.care-tpl:disabled{cursor:not-allowed;opacity:.5}
.care-tpl b{font-size:13.5px;font-weight:600}
.care-tpl span{font-size:12px;line-height:1.5;color:var(--ui-text-secondary)}
.care-prefs{padding-bottom:16px}
.care-prefs-title{margin:0;padding:14px 16px 6px;font-size:13px;font-weight:600;color:var(--ui-text-primary)}
.care-prefs .xb-row{min-height:0;padding:10px 16px}
.care-prefs .xb-row-text>b{font-size:13px}
.care-time-pair{display:flex;flex-wrap:wrap;gap:12px;padding:0 16px 10px;color:var(--ui-text-secondary);font-size:12.5px}
.care-weather-source{padding:0 16px;border-top:1px solid var(--ui-line)}
.care-weather-source :deep(.n-collapse){margin:10px 0}
.care-save{padding:12px 16px 0;display:flex;justify-content:flex-end}
.care-weather{padding:16px 18px}
.care-weather h2{margin:0 0 8px;font-size:14px;font-weight:600;color:var(--ui-text-primary)}
.care-weather p{margin:0 0 6px;font-size:13px;line-height:1.6;color:var(--ui-text-primary)}
.care-dialog :deep(.n-form-item){margin-bottom:0}
.care-form-group{padding:0;overflow:hidden}
.care-form-group :deep(.n-form-item){padding:12px 16px;margin:0}
.care-form-group :deep(.n-form-item+.n-form-item),.care-form-group>*+*{border-top:1px solid var(--ui-line)}
.care-form-group :deep(.n-form-item-feedback-wrapper){min-height:0}
.care-form-group>div[data-testid]{border-top:1px solid var(--ui-line)}
input[type=time],input[type=datetime-local]{border:0;border-radius:8px;padding:6px 10px;font:inherit;font-size:13px;color:var(--ui-text-primary);background:var(--ui-control);outline:none}
.care-form :deep(.n-form-item-label){font-size:13px;color:var(--ui-text-secondary)}
.city-search{display:flex;width:100%;gap:8px}
.city-results{display:flex;gap:8px;flex-wrap:wrap;padding:0 16px}
.city-actions{display:flex;gap:8px;flex-wrap:wrap;margin-top:6px}
.care-city-state{margin:6px 16px;font-size:13px;color:var(--ui-text-primary)}
.care-form-group>p{margin:6px 16px}
.dialog-actions{display:flex;justify-content:flex-end;gap:10px;margin-top:16px}
.dialog-actions .xb-btn-primary,.dialog-actions .xb-btn{height:32px}
/* Use the available page/column width: the expanded sidebar consumes 264px. */
@container care-page (max-width:780px){
  .care-layout{grid-template-columns:minmax(0,1fr)}
}
@container care-main (max-width:620px){
  .care-rule{display:grid;grid-template-columns:36px minmax(0,1fr) auto;gap:8px 12px}
  .care-rule>.care-ico{grid-column:1;grid-row:1}
  .care-rule>.xb-row-text{grid-column:2;grid-row:1}
  .care-rule .xb-row-text>b{white-space:normal}
  .care-rule>.care-switch{grid-column:3;grid-row:1}
  .care-rule>.care-next{grid-column:2 / -1;grid-row:2;justify-self:start;max-width:100%;white-space:normal;overflow-wrap:anywhere}
  .care-rule-actions{grid-column:2 / -1;grid-row:3;flex-wrap:wrap}
}
@container care-main (max-width:460px){
  .care-tpl-grid{grid-template-columns:repeat(2,minmax(0,1fr))}
}
@container care-main (max-width:280px){
  .care-tpl-grid{grid-template-columns:minmax(0,1fr)}
}
</style>

