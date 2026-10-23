<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { auditLabels, clock, dayKey, dayTitle } from '../format';
import type { AuditRow } from '../types';
import EmptyState from '../components/EmptyState.vue';
import Icon, { type IconName } from '../components/Icon.vue';
import PageHead from '../components/PageHead.vue';

const items = ref<AuditRow[]>([]);
const error = ref('');
const loading = ref(true);

const icons: Record<string, IconName> = {
  'admin.password_change': 'key',
  'product.save': 'layers',
  'tier.save': 'layers',
  'tier.delete': 'trash',
  'media.save': 'layers',
  'media.delete': 'trash',
  'binding.grant': 'plus',
  'bucket.void': 'undo',
  'anomaly.resolve': 'circle-check',
  'batch.create': 'ticket',
  'batch.void': 'x',
  'code.void': 'x',
  'settings.newapi_token': 'plug',
};

interface Group {
  key: string;
  title: string;
  rows: AuditRow[];
}
const groups = computed<Group[]>(() => {
  const out: Group[] = [];
  for (const row of items.value) {
    const key = dayKey(row.created_at);
    let group = out[out.length - 1];
    if (!group || group.key !== key) {
      group = { key, title: dayTitle(key), rows: [] };
      out.push(group);
    }
    group.rows.push(row);
  }
  return out;
});

onMounted(async () => {
  try {
    items.value = (await api.audit()).items;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div>
    <PageHead title="操作日志" desc="管理员做过的改动，最近 100 条。改档位、补发和扣回额度、生成兑换码都会记在这里。" />

    <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="17" /><span>{{ error }}</span></div>

    <div v-if="loading" class="skels"><i v-for="n in 5" :key="n" class="skel" style="height: 46px"></i></div>
    <div v-else-if="items.length === 0" class="panel"><EmptyState icon="scroll" title="还没有操作记录" desc="你在后台做的每一次改动，都会按时间记在这里。" /></div>

    <div v-for="g in groups" :key="g.key" class="day">
      <h2 class="dh">{{ g.title }}</h2>
      <ul class="tl">
        <li v-for="(r, i) in g.rows" :key="`${g.key}-${i}`">
          <span class="node" aria-hidden="true"><Icon :name="icons[r.action] ?? 'circle-check'" :size="15" /></span>
          <div class="body">
            <div class="head"><strong>{{ auditLabels[r.action] ?? r.action }}</strong><code v-if="r.target" class="mono target">{{ r.target }}</code></div>
            <p v-if="r.detail" class="detail mono">{{ r.detail }}</p>
          </div>
          <div class="when num"><span>{{ clock(r.created_at) }}</span><span class="faint who">{{ r.admin }}</span></div>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.notice { margin-bottom: 14px; }
.skels { display: flex; flex-direction: column; gap: 10px; }
.day + .day { margin-top: 28px; }
.dh { font-size: 13px; color: var(--muted); margin-bottom: 12px; }
.tl { list-style: none; margin: 0; padding: 0; position: relative; }
.tl::before { content: ''; position: absolute; left: 17px; top: 18px; bottom: 18px; width: 2px; background: var(--line); border-radius: 1px; }
.tl li { position: relative; display: flex; align-items: flex-start; gap: 14px; padding: 9px 0; }
.node { position: relative; z-index: 1; flex: none; display: grid; place-items: center; width: 36px; height: 36px; border-radius: 50%; background: var(--surface); color: var(--tide); box-shadow: 0 0 0 1.5px var(--line-strong), 0 0 0 5px var(--paper); }
.body { flex: 1; min-width: 0; padding-top: 5px; }
.head { display: flex; align-items: center; flex-wrap: wrap; gap: 4px 10px; }
.target { padding: 1px 7px; border-radius: 6px; background: #e8eeef; color: var(--muted); font-size: 12px; }
.detail { color: var(--muted); margin-top: 3px; overflow-wrap: anywhere; max-width: 78ch; font-size: 12px; }
.when { flex: none; display: flex; flex-direction: column; align-items: flex-end; padding-top: 5px; font-weight: 600; font-size: 13px; }
.who { font-weight: 500; font-size: 12px; }
</style>
