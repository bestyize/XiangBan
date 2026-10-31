<script setup lang="ts">
import { computed } from 'vue'
const props = defineProps<{ kind: string; modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
type Field = { key: string; label: string; type?: 'number' | 'datetime' | 'list' | 'url' | 'color'; required?: boolean; max?: number }
const fields: Record<string, Field[]> = {
  banner: [{key:'tag',label:'标签',max:12},{key:'title',label:'标题',required:true,max:60},{key:'byline',label:'署名',max:40},{key:'link',label:'目标（如 topic:12）'}],
  news: [{key:'title',label:'新闻标题',required:true,max:300},{key:'source',label:'来源',required:true,max:30},{key:'url',label:'官方原文',type:'url',required:true},{key:'cover_url',label:'封面地址',type:'url'},{key:'source_published_at',label:'来源发布时间（本地时区，未知留空）',type:'datetime'}],
  nav: [{key:'group',label:'分组',required:true,max:10},{key:'name',label:'名称',required:true,max:20},{key:'vendor',label:'厂商',required:true,max:20},{key:'url',label:'官方入口',type:'url',required:true},{key:'brand_color',label:'品牌色 #RRGGBB',required:true},{key:'tint',label:'底色 #RRGGBB',required:true}],
  doc: [{key:'product',label:'产品',required:true,max:20},{key:'title',label:'资料标题',required:true,max:60},{key:'subtitle',label:'说明',max:80},{key:'label',label:'类型（guide 导读 / index 索引）',required:true},{key:'url',label:'官方原文',type:'url',required:true}],
  launch: [{key:'title',label:'发布记录标题（历史内容须标注回顾）',required:true,max:80},{key:'summary',label:'已核实摘要',max:500},{key:'source_url',label:'官方发布原文（https）',type:'url'},{key:'tags',label:'标签（每行一个，最多 5 个）',type:'list'}],
  promo: [{key:'text',label:'优惠或试用说明',required:true,max:80},{key:'source_url',label:'官方来源（https；长期权益必填）',type:'url'},{key:'eligibility',label:'地区、身份等适用资格（长期权益必填）',max:500},{key:'started_at',label:'真实开始时间（未知留空；本地时区）',type:'datetime'},{key:'ends_at',label:'真实截止时间（长期提供留空；本地时区）',type:'datetime'}],
  hotword: [{key:'text',label:'热门搜索词',required:true,max:20},{key:'tag',label:'标记',max:4}],
  vendor: [{key:'name',label:'厂商名',required:true,max:20},{key:'brand_color',label:'品牌色 #RRGGBB',required:true},{key:'tint',label:'底色 #RRGGBB',required:true}],
  quota: [{key:'progress_pct',label:'手工确认的已用比例（0–100）',type:'number',required:true},{key:'next_reset_at',label:'手工确认的下次重置（本地时区）',type:'datetime',required:true}],
}
const selected = computed(() => fields[props.kind] ?? [])
const parsed = computed<Record<string, unknown> | null>(() => {
  try { const v: unknown = JSON.parse(props.modelValue); return v && typeof v === 'object' && !Array.isArray(v) ? v as Record<string, unknown> : null } catch { return null }
})
function value(field: Field) {
  const v = parsed.value?.[field.key]
  if (field.type === 'list') return Array.isArray(v) ? v.join('\n') : ''
  if (field.type === 'datetime' && typeof v === 'string') {
    const d = new Date(v)
    return Number.isNaN(d.getTime()) ? '' : new Date(d.getTime() - d.getTimezoneOffset()*60000).toISOString().slice(0,16)
  }
  return v == null ? '' : String(v)
}
function update(field: Field, event: Event) {
  if (!parsed.value) return
  const raw = (event.target as HTMLInputElement).value
  let next: unknown = raw
  if (field.type === 'number') next = raw === '' ? null : Number(raw)
  if (field.type === 'list') next = raw.split('\n').map(x => x.trim()).filter(Boolean)
  if (field.type === 'datetime') next = raw ? new Date(raw).toISOString() : null
  const data = {...parsed.value, [field.key]:next}
  if (props.kind === 'news' && field.key === 'source_published_at') data.time_basis = raw ? 'source_published' : 'collected'
  emit('update:modelValue',JSON.stringify(data,null,2))
}
</script>
<template>
  <div v-if="selected.length" class="wide typed-fields">
    <p v-if="!parsed" class="error" role="alert">JSON 格式有误，修正后可继续使用字段表单。</p>
    <template v-else>
      <p v-if="kind === 'quota'" class="muted">额度仅手工维护；需要可靠依据，不代表所有账号的实时额度。无法确认时请保留草稿。</p>
      <p v-if="kind === 'promo'" class="muted">没有统一截止日期的长期权益可留空；需提供官方来源与适用资格，不能用领取后试用时长推造统一截止日。限时活动须填写真实起止时间。</p>
      <p v-if="kind === 'launch'" class="muted">历史发布须保留原日期并标注“历史回顾”，不要写成今日上新；摘要和出处仅使用已核实的官方资料。</p>
      <div class="adm-form-grid">
        <label v-for="field in selected" :key="field.key" class="field">
          <span>{{ field.label }}</span>
          <textarea v-if="field.type === 'list'" class="input" :value="value(field)" rows="3" @change="update(field,$event)" />
          <input v-else class="input" :type="field.type === 'datetime' ? 'datetime-local' : field.type ?? 'text'" :value="value(field)" :required="field.required" :maxlength="field.max" :min="field.type === 'number' ? 0 : undefined" :max="field.type === 'number' ? 100 : undefined" @change="update(field,$event)" />
        </label>
      </div>
    </template>
  </div>
</template>
<style scoped>.typed-fields{border-block:1px solid var(--line);padding-block:12px;margin-block:4px}.typed-fields p{margin:0 0 10px}</style>
