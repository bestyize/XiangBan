<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { NAlert, NButton, NCollapse, NCollapseItem, NEmpty, NForm, NFormItem, NInput, NInputNumber, NModal, NPopconfirm, NSelect, NSwitch, NTag } from 'naive-ui';
import { invoke } from '../../bridge';
import { careNames, dayNames, describeCareRule, newCareRule, weatherIsFresh, type CareCity, type CareKind, type CareStatus } from '../../care-model';
import type { ControlStore } from '../control-types';
import { clone, createCareState } from '../care/state';
const props = defineProps<{ store: ControlStore }>();
const care = createCareState(props.store.desktop, props.store.notify);
const { status, preferences, preferencesDirty, draft, editing, busy } = care;
const kinds = Object.entries(careNames).map(([value, label]) => ({ value, label }));
const repeats = [{ label: '每天', value: 'daily' }, { label: '指定星期', value: 'weekdays' }, { label: '仅一次', value: 'once' }];
const sources = [{ label: '关闭 · 不访问天气网络', value: 'disabled' }, { label: 'Open-Meteo · 个人非商业使用', value: 'open-meteo' }, { label: '自己的兼容 HTTPS 来源', value: 'custom' }];
const weatherEnabled = computed(() => status.value.config.weatherSource.provider !== 'disabled');
const cityQuery = ref(''); const cities = ref<CareCity[]>([]); const searchDone = ref(false);
function open(kind: CareKind = 'custom') { care.open(kind); cityQuery.value = ''; cities.value = []; searchDone.value = false; }
function changeKind(kind: CareKind) { const defaults = newCareRule(kind); draft.value = { ...draft.value, kind, text: defaults.text, time: defaults.time, city: null }; cities.value = []; }
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
onMounted(() => { void care.initialize(); }); onUnmounted(care.dispose);
</script>

<template>
  <main class="ui-page care-page" data-testid="care-page">

    <NAlert v-if="!store.desktop" type="info" :show-icon="false">浏览器预览：初始没有提醒。可临时添加并检查表单，本次预览不保存、不在后台触发。</NAlert>
    <NAlert v-if="!store.settings.companionEnabled" type="warning" :show-icon="false">小伴已隐藏，关怀提醒暂缓展示。<NButton text type="primary" @click="store.navigate('companion')">前往桌面小伴开启显示</NButton></NAlert>
    <NAlert v-if="status.error" type="error">{{ status.error }}</NAlert>
    <section class="surface list-panel"><div class="section-heading"><h2>我的提醒 <span class="muted">{{ status.rules.length }}</span></h2><div class="quick-types"><NButton type="primary" data-testid="care-add" @click="open()">新增提醒</NButton><NButton v-for="kind in kinds" :key="kind.value" size="small" :disabled="kind.value === 'weather' && !weatherEnabled" @click="open(kind.value as CareKind)">＋ {{ kind.label }}</NButton></div></div>
      <p v-if="!weatherEnabled" class="muted">天气来源已关闭，暂不可搜索城市或添加天气提醒；本地提醒仍可使用。</p>
      <NEmpty v-if="!status.rules.length" description="还没有提醒，从一条喝水或休息提醒开始吧" class="empty" />
      <article v-for="rule in status.rules" :key="rule.id" class="reminder" :data-testid="`care-rule-${rule.id}`"><div class="reminder-copy"><NTag size="small" :bordered="false" type="info">{{ careNames[rule.kind] }}</NTag><strong>{{ rule.text || '根据真实天气生成提示' }}</strong><p>{{ describeCareRule(rule) }}</p><small v-if="rule.notBefore > Date.now()/1000" class="muted">稍后提醒：{{ new Date(rule.notBefore * 1000).toLocaleString() }}</small></div><div class="rule-actions"><NSwitch :value="rule.enabled" :disabled="busy" :aria-label="`启用${careNames[rule.kind]}提醒`" @update:value="care.toggle(rule, $event)"/><NButton size="small" :disabled="busy" @click="care.open(rule.kind, rule)">编辑</NButton><NButton size="small" :disabled="busy" @click="care.preview(rule)">预览</NButton><NPopconfirm @positive-click="care.remove(rule)"><template #trigger><NButton size="small" :disabled="busy">删除</NButton></template>删除这条提醒？待展示的任务也会取消。</NPopconfirm></div></article>
    </section>
    <section class="surface preferences" @change="preferencesDirty = true"><h2>提醒偏好</h2><div class="setting-row"><div><strong>开启关怀提醒</strong><p class="muted">关闭后暂停全部关怀提醒。</p></div><NSwitch v-model:value="preferences.enabled" @update:value="preferencesDirty = true" /></div><div class="setting-row"><strong>提醒声音</strong><NSwitch v-model:value="preferences.sound" @update:value="preferencesDirty = true" /></div><div class="setting-row"><div><strong>免打扰时段</strong><p class="muted">支持跨午夜，期间的有效提醒会延后。</p></div><NSwitch v-model:value="preferences.quietEnabled" @update:value="preferencesDirty = true"/></div><div v-if="preferences.quietEnabled" class="time-pair"><label>开始 <input v-model="preferences.quietStart" aria-label="免打扰开始" type="time"></label><label>结束 <input v-model="preferences.quietEnd" aria-label="免打扰结束" type="time"></label></div>
      <NCollapse><NCollapseItem title="天气来源与网络设置" name="weather"><NForm label-placement="top"><NFormItem label="天气来源"><NSelect v-model:value="preferences.weatherSource.provider" :options="sources" @update:value="preferencesDirty = true" /></NFormItem><template v-if="preferences.weatherSource.provider === 'custom'"><NFormItem label="天气 API · HTTPS"><NInput v-model:value="preferences.weatherSource.forecastUrl" placeholder="https://example.com/v1/forecast" @update:value="preferencesDirty = true"/></NFormItem><NFormItem label="城市搜索 API · HTTPS"><NInput v-model:value="preferences.weatherSource.geocodingUrl" placeholder="https://example.com/v1/search" @update:value="preferencesDirty = true"/></NFormItem><p class="muted">需兼容 Open-Meteo JSON 与参数；不在客户端填写密钥。</p></template><p class="muted">仅发送手选城市坐标。Open-Meteo 免费来源限个人非商业使用；天气数据 CC BY 4.0，城市数据 GeoNames。保存来源后才可搜索城市。</p></NForm></NCollapseItem></NCollapse><NButton type="primary" secondary :loading="busy" data-testid="care-save-preferences" @click="care.savePreferences">保存提醒偏好</NButton>
    </section>
    <section v-if="status.weather || status.weatherError" class="surface weather-status"><template v-if="status.weather"><h2>{{ weatherIsFresh(status.weather) ? '最近天气' : '天气已过期 · 历史记录' }}</h2><p>{{ status.weather.text }}</p><p class="muted">查询 {{ new Date(status.weather.updatedAt * 1000).toLocaleString() }} · 模型 {{ new Date(status.weather.modelAt * 1000).toLocaleString() }} · {{ status.weather.source }}</p></template><NAlert v-if="status.weatherError" type="warning">{{ status.weatherError }}</NAlert></section>
    <p class="muted behavior-note">{{ status.pendingCount ? `有 ${status.pendingCount} 条待展示提醒。` : '' }}提醒按系统本地时间运行。游戏、免打扰、锁屏或小伴隐藏时暂缓，只补发 30 分钟内仍有效的摘要。气泡约显示 10 秒，可稍后 10 分钟提醒；应用完全退出后停止。{{ status.systemNotificationsAvailable ? '系统通知可用。' : '当前系统通知渠道未接入。' }}</p>
    <NModal v-model:show="editing" preset="card" :title="draft.id ? '编辑提醒' : '新建提醒'" class="care-dialog" style="width: min(560px, calc(100vw - 48px))" :mask-closable="false" data-testid="care-dialog"><NForm label-placement="top" @submit.prevent="care.save"><NFormItem label="提醒类型"><NSelect :value="draft.kind" :options="kinds" data-testid="care-kind" @update:value="changeKind"/></NFormItem><NFormItem label="启用这条提醒"><NSwitch v-model:value="draft.enabled" /></NFormItem>
      <NFormItem v-if="draft.kind === 'rest'" label="有效使用电脑的间隔（分钟）" data-testid="care-rest-field"><NInputNumber v-model:value="draft.intervalMinutes" :min="5" :max="480" :precision="0"/><template #feedback>锁屏、睡眠和连续空闲超过一分钟不累计。{{ status.activityAvailable ? '已连接系统空闲检测。' : '系统空闲检测不可用，休息计时暂不累计。' }}</template></NFormItem>
      <template v-else><NFormItem label="重复方式"><NSelect v-model:value="draft.repeat" :options="repeats" data-testid="care-repeat"/></NFormItem><NFormItem v-if="draft.repeat === 'once'" label="日期和时间" data-testid="care-once-field"><input v-model="draft.onceAt" type="datetime-local" aria-label="日期和时间"></NFormItem><NFormItem v-else label="提醒时间" data-testid="care-time-field"><input v-model="draft.time" type="time" aria-label="提醒时间"></NFormItem><NFormItem v-if="draft.repeat === 'weekdays'" label="提醒星期" data-testid="care-weekdays-field"><NSelect v-model:value="draft.weekdays" multiple :options="dayNames.map((label, i) => ({ label, value: i + 1 }))"/></NFormItem></template>
      <div v-if="draft.kind === 'weather'" data-testid="care-weather-fields"><NAlert v-if="!weatherEnabled" type="warning">请先在提醒偏好中启用并保存天气来源；关闭来源时不可搜索城市或添加天气提醒。</NAlert><NFormItem label="手动选择城市"><div class="city-search"><NInput v-model:value="cityQuery" placeholder="例如 北京 / Beijing" :disabled="!weatherEnabled"/><NButton :disabled="!weatherEnabled || busy" @click="searchCities">搜索</NButton></div></NFormItem><div class="city-results"><NButton v-for="(city, index) in cities" :key="index" size="small" @click="draft.city = clone(city); cities = []">{{ [city.name, city.admin1, city.country].filter(Boolean).join(' · ') }}</NButton></div><p v-if="searchDone && !cities.length && !draft.city" class="muted">没有匹配城市，可试英文名或加上省份。</p><p>{{ draft.city ? `已选择：${[draft.city.name, draft.city.admin1, draft.city.country].filter(Boolean).join(' · ')}` : '尚未选择城市' }}</p><NButton size="small" :disabled="!weatherEnabled || !draft.city || busy" @click="checkWeather">查询所选城市天气</NButton></div>
      <NFormItem :label="draft.kind === 'weather' ? '提醒文案（可选开头）' : '提醒文案'"><NInput v-model:value="draft.text" type="textarea" :maxlength="240" show-count data-testid="care-text"/><template #feedback>最多 240 个字，文案仅保存在本机。</template></NFormItem><div class="dialog-actions"><NButton data-testid="care-cancel" @click="editing = false">取消</NButton><NButton :disabled="busy" @click="care.preview(draft)">预览</NButton><NButton type="primary" :loading="busy" data-testid="care-submit" attr-type="submit">{{ draft.id ? '保存修改' : '创建提醒' }}</NButton></div>
    </NForm></NModal>
  </main>
</template>

<style scoped>
.care-page{display:grid;gap:20px}.page-header,.section-heading{display:flex;justify-content:space-between;align-items:center;gap:16px}.care-page h2{font-size:16px;margin:0 0 16px}.list-panel,.preferences,.weather-status{padding:24px}.section-heading h2{margin:0}.quick-types,.rule-actions,.city-results{display:flex;gap:8px;flex-wrap:wrap}.empty{padding:36px 0}.reminder{display:flex;justify-content:space-between;align-items:center;gap:20px;padding:18px 0;border-bottom:1px solid #edf1f6}.reminder:last-child{border-bottom:0}.reminder-copy strong{margin-left:10px;color:#25354d}.reminder-copy p{margin:8px 0;color:#708096;font-size:13px}.rule-actions{align-items:center;flex-shrink:0}.preferences :deep(.n-collapse){margin:20px 0}.time-pair{display:flex;gap:20px;margin:16px 0}.time-pair label{color:#65758a}.behavior-note{font-size:12px;line-height:1.8;margin:0}.city-search{display:flex;width:100%;gap:8px}.dialog-actions{display:flex;justify-content:flex-end;gap:10px;margin-top:20px}input[type=time],input[type=datetime-local]{border:1px solid #dce3ec;border-radius:6px;padding:8px 10px;font:inherit;color:#34465f;background:white}.care-dialog :deep(.n-form-item){margin-bottom:8px}@media(max-width:850px){.reminder,.section-heading{align-items:flex-start;flex-direction:column}.rule-actions{align-self:flex-end}}
</style>
