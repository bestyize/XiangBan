<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { formatTime, relativeTime } from '../format';
import { notify } from '../toast';
import type { Anomaly } from '../types';
import EmptyState from '../components/EmptyState.vue';
import Icon from '../components/Icon.vue';
import PageHead from '../components/PageHead.vue';

const emit = defineEmits<{ changed: [] }>();

const items = ref<Anomaly[]>([]);
const error = ref('');
const loading = ref(true);

const open = computed(() => items.value.filter((i) => !i.resolved_at));
const done = computed(() => items.value.filter((i) => i.resolved_at));

async function load() {
  try {
    items.value = (await api.anomalies()).items;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
}
onMounted(load);

async function resolve(item: Anomaly) {
  error.value = '';
  try {
    await api.resolveAnomaly(item.id);
    notify('已标记为处理');
    await load();
    emit('changed');
  } catch (e) {
    error.value = errorMessage(e);
  }
}

const kindLabel: Record<string, string> = { external_credit: '网关里的额度比预期多' };
</script>

<template>
  <div>
    <PageHead title="对账异常" desc="系统每分钟会核对 AI 网关里的用户额度。核对不上时记在这里。系统已经按我们的账本把网关里的额度改回了正确的数，处理只是确认你知道了。" />

    <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="17" /><span>{{ error }}</span></div>

    <div v-if="loading" class="list"><i class="skel" style="height: 74px; border-radius: 12px"></i></div>

    <template v-else>
      <div v-if="items.length === 0" class="panel"><EmptyState icon="shield" title="账目是对的" desc="系统没有发现网关里的额度和我们的账本对不上。" /></div>

      <section v-if="open.length" aria-label="待处理">
        <h2 class="gh">待处理 <span class="tag amber num">{{ open.length }}</span></h2>
        <ul class="list">
          <li v-for="i in open" :key="i.id" class="item open">
            <span class="ic"><Icon name="alert" :size="18" /></span>
            <div class="mid">
              <strong>{{ kindLabel[i.kind] ?? i.kind }}</strong>
              <p>{{ i.detail }}</p>
              <span class="meta faint num" :title="formatTime(i.created_at)">绑定 #{{ i.binding_id }}，{{ relativeTime(i.created_at) }}发现</span>
            </div>
            <button type="button" class="btn small" @click="resolve(i)"><Icon name="check" :size="15" />标记为已处理</button>
          </li>
        </ul>
      </section>

      <section v-if="done.length" class="past" aria-label="已处理">
        <h2 class="gh">已处理 <span class="tag num">{{ done.length }}</span></h2>
        <ul class="list">
          <li v-for="i in done" :key="i.id" class="item">
            <span class="ic ok"><Icon name="circle-check" :size="18" /></span>
            <div class="mid">
              <strong>{{ kindLabel[i.kind] ?? i.kind }}</strong>
              <p>{{ i.detail }}</p>
              <span class="meta faint num">绑定 #{{ i.binding_id }}，{{ formatTime(i.resolved_at) }} 处理</span>
            </div>
          </li>
        </ul>
      </section>
    </template>
  </div>
</template>

<style scoped>
.notice { margin-bottom: 14px; }
.gh { display: flex; align-items: center; gap: 9px; font-size: 14px; margin: 0 0 12px; }
.past { margin-top: 30px; }
.list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 10px; }
.item { display: flex; align-items: flex-start; gap: 14px; padding: 16px 18px; border: 1px solid var(--line); border-radius: 14px; background: var(--surface); }
.item.open { border-color: var(--amber-line); background: linear-gradient(90deg, var(--amber-soft), var(--surface) 38%); }
.ic { flex: none; display: grid; place-items: center; width: 36px; height: 36px; border-radius: 11px; background: var(--amber-soft); color: var(--amber); }
.ic.ok { background: var(--tide-soft); color: var(--tide); }
.item:not(.open) { background: transparent; }
.item:not(.open) strong { color: var(--muted); }
.mid { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 3px; }
.mid p { color: var(--muted); max-width: 72ch; }
.meta { font-size: 12.5px; margin-top: 2px; }

@media (max-width: 640px) {
  .item { flex-wrap: wrap; }
  .item .btn { margin-left: 50px; }
}
</style>
