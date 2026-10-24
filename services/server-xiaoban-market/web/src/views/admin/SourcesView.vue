<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import { admin } from '@/lib/api';
import { formatTime, KIND_NAME } from '@/lib/format';
import { AUTO_IMPORT, INTERVALS, intervalLabel } from '@/lib/labels';
import { toast, toastError } from '@/lib/toast';
import type { ProbeResult, Source } from '@/lib/types';

const router = useRouter();
const sources = ref<Source[]>([]);
const loading = ref(true);
const selectedId = ref<number | null>(null);
const busy = ref(false);
const probe = ref<ProbeResult | null>(null);

const MAPPING_FIELDS = ['upstream_id', 'name', 'slug', 'summary', 'description', 'category', 'tags', 'icon_url', 'origin_url', 'license', 'namespace', 'version', 'downloads', 'stars'];

// 表单状态：来源本身 + 配置里的常用字段；其余配置项原样保留。
const form = ref({ name: '', enabled: true, intervalMinutes: 0, autoImport: 'none' });
const cfg = ref<Record<string, any>>({});
const texts = ref({ body: '', query: '', headers: '', raw: '' });
const rows = ref<{ field: string; rule: string }[]>([]);

const selected = computed(() => sources.value.find((s) => s.id === selectedId.value) ?? null);
const isGeneric = computed(() => selected.value?.adapter === 'generic_json');

async function load(keep = true) {
  loading.value = true;
  try {
    sources.value = (await admin.sources()).results;
    if (!keep || !selected.value) select(sources.value[0]?.id ?? null);
    else fill(selected.value);
  } catch (e) {
    toastError(e);
  } finally {
    loading.value = false;
  }
}

function fill(s: Source) {
  form.value = { name: s.name, enabled: s.enabled, intervalMinutes: s.intervalMinutes, autoImport: s.autoImport };
  cfg.value = JSON.parse(JSON.stringify(s.config));
  const pretty = (v: unknown) => (v && Object.keys(v as object).length ? JSON.stringify(v, null, 2) : '');
  texts.value = { body: cfg.value.body ? JSON.stringify(cfg.value.body, null, 2) : '', query: pretty(cfg.value.query), headers: pretty(cfg.value.headers), raw: '' };
  // 服务端返回的 JSON 键是按字母序的；这里按常用顺序展示，未知字段排在最后。
  const order = (f: string) => (MAPPING_FIELDS.includes(f) ? MAPPING_FIELDS.indexOf(f) : MAPPING_FIELDS.length);
  rows.value = Object.entries(cfg.value.mapping ?? {})
    .sort(([a], [b]) => order(a) - order(b))
    .map(([field, rule]) => ({ field, rule: typeof rule === 'string' ? rule : JSON.stringify(rule) }));
}

function select(id: number | null) {
  selectedId.value = id;
  probe.value = null;
  const s = sources.value.find((x) => x.id === id);
  if (s) fill(s);
}

function parseJson(label: string, text: string): unknown {
  try {
    return JSON.parse(text);
  } catch {
    throw new Error(`「${label}」不是有效的 JSON`);
  }
}

function intField(v: unknown): number | undefined {
  const n = Number(v);
  return v === '' || v === undefined || v === null || Number.isNaN(n) ? undefined : Math.max(0, Math.trunc(n));
}

/** 把表单合并回配置对象；未在表单里出现的配置项原样保留。 */
function buildConfig(): Record<string, unknown> {
  const out: Record<string, any> = { ...cfg.value };
  for (const k of ['request_interval_ms', 'page_size', 'page_start', 'max_pages']) {
    const n = intField(out[k]);
    if (n === undefined) delete out[k];
    else out[k] = n;
  }
  for (const k of ['url', 'base_url', 'api_key', 'method', 'items_path', 'total_path', 'origin_url_template', 'config_path', 'namespace']) {
    if (typeof out[k] === 'string') {
      out[k] = out[k].trim();
      if (!out[k]) delete out[k];
    }
  }
  if (isGeneric.value) {
    for (const [k, label] of [['body', '请求体模板'], ['query', '查询参数'], ['headers', '请求头']] as const) {
      const t = texts.value[k].trim();
      if (t) out[k] = parseJson(label, t);
      else delete out[k];
    }
    const mapping: Record<string, unknown> = {};
    for (const r of rows.value) {
      const field = r.field.trim(), rule = r.rule.trim();
      if (!field || !rule) continue;
      mapping[field] = rule.startsWith('[') ? parseJson(`字段映射 ${field}`, rule) : rule;
    }
    out.mapping = mapping;
  }
  return out;
}

async function save(): Promise<boolean> {
  if (!selected.value) return false;
  try {
    await admin.updateSource(selected.value.id, {
      name: form.value.name.trim() || selected.value.name,
      enabled: form.value.enabled,
      intervalMinutes: form.value.intervalMinutes,
      autoImport: form.value.autoImport,
      config: buildConfig(),
    });
    await load();
    return true;
  } catch (e) {
    toastError(e);
    return false;
  }
}

async function onSave() {
  busy.value = true;
  if (await save()) toast('已保存');
  busy.value = false;
}

async function onProbe() {
  if (!selected.value) return;
  busy.value = true;
  probe.value = null;
  try {
    if (await save()) probe.value = await admin.probe(selected.value.id);
  } catch (e) {
    toastError(e);
  } finally {
    busy.value = false;
  }
}

async function startJob(id: number) {
  const job = await admin.startJob(id);
  toast('采集任务已开始');
  router.push({ name: 'admin-jobs', query: { job: job.id } });
}

async function onSaveAndRun() {
  busy.value = true;
  try {
    if (await save()) await startJob(selected.value!.id);
  } catch (e) {
    toastError(e);
  } finally {
    busy.value = false;
  }
}

async function quickRun(s: Source) {
  try {
    await startJob(s.id);
  } catch (e) {
    toastError(e);
  }
}

function applyRaw() {
  try {
    const parsed = parseJson('完整配置', texts.value.raw) as Record<string, unknown>;
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) throw new Error('完整配置必须是 JSON 对象');
    const s = selected.value!;
    fill({ ...s, config: parsed });
    toast('已载入到表单，记得保存');
  } catch (e) {
    toastError(e);
  }
}
function openRaw(e: Event) {
  if ((e.target as HTMLDetailsElement).open) {
    try { texts.value.raw = JSON.stringify(buildConfig(), null, 2); } catch { texts.value.raw = JSON.stringify(cfg.value, null, 2); }
  }
}

function state(s: Source): [string, 'ok' | 'warn' | 'bad' | 'info' | 'idle'] {
  if (!s.enabled) return ['已停用', 'idle'];
  if (!s.configured) return ['待配置', 'warn'];
  if (s.lastJob?.status === 'failed') return ['上次失败', 'bad'];
  if (s.lastJob?.status === 'running') return ['采集中', 'info'];
  return ['正常', 'ok'];
}
const lastText = (s: Source) => {
  if (!s.lastJob) return s.configured ? '尚未采集' : '尚未采集 · 接口未配置';
  const st = s.lastJob.stats;
  return `上次：${formatTime(s.lastJob.createdAt)} · 新增 ${st.new ?? 0} · 更新 ${st.changed ?? 0}`;
};
const first = computed(() => probe.value?.entries?.[0]);

onMounted(() => load(false));
</script>

<template>
  <div class="admin-page">
    <div class="page-h">
      <div><h1>采集来源</h1><div class="sub">每个上游板块一个来源；先「试采一页」确认字段映射，再正式采集</div></div>
    </div>

    <div class="split">
      <section class="list">
        <div v-if="loading && !sources.length" class="empty"><span class="spinner" /> 加载中…</div>
        <article v-for="s in sources" :key="s.id" class="src panel" :class="{ on: s.id === selectedId }">
          <button type="button" class="head" :aria-pressed="s.id === selectedId" @click="select(s.id)">
            <b>{{ s.name }}</b>
            <span class="kind" :class="`k-${s.kind}`">{{ KIND_NAME[s.kind] }}</span>
            <span class="badge" :class="`badge-${state(s)[1]}`">{{ state(s)[0] }}</span>
          </button>
          <div class="meta">{{ s.adapter === 'skillhub' ? 'SkillHub 适配器' : '通用 JSON 适配器' }} · {{ intervalLabel(s.intervalMinutes) }}<template v-if="s.pending"> · 待处理 {{ s.pending }}</template></div>
          <div class="meta">{{ lastText(s) }}</div>
          <div class="btns">
            <button class="btn btn-sm" :class="{ 'btn-primary': s.configured && s.enabled }" type="button" :disabled="!s.configured || !s.enabled" @click="quickRun(s)">立即采集</button>
            <button class="btn btn-sm" type="button" @click="select(s.id)">配置</button>
          </div>
        </article>
      </section>

      <section v-if="selected" class="editor panel">
        <div class="panel-b stack">
          <div class="title">
            <h2>{{ selected.name }}</h2>
            <span class="kind" :class="`k-${selected.kind}`">{{ KIND_NAME[selected.kind] }}</span>
            <span class="chip">{{ selected.adapter === 'skillhub' ? 'SkillHub 适配器' : '通用 JSON 适配器' }}</span>
            <label class="check push"><input v-model="form.enabled" type="checkbox" />启用</label>
          </div>

          <div v-if="!selected.configured" class="notice">
            <b>接口尚未配置。</b>在浏览器打开该板块页面，从开发者工具的网络面板找到加载列表的请求，把地址、请求体和响应结构填在下面，再点「试采一页」，字段映射全部命中后再采集。
          </div>

          <div class="row">
            <div class="field"><label for="nm">来源名称</label><input id="nm" v-model="form.name" class="input" /></div>
            <div class="field"><label for="iv">定时采集</label>
              <select id="iv" v-model.number="form.intervalMinutes" class="select"><option v-for="[m, l] in INTERVALS" :key="m" :value="m">{{ l }}</option></select>
            </div>
            <div class="field"><label for="ai">采集后</label>
              <select id="ai" v-model="form.autoImport" class="select"><option v-for="[v, l] in AUTO_IMPORT" :key="v" :value="v">{{ l }}</option></select>
            </div>
          </div>

          <!-- SkillHub -->
          <template v-if="!isGeneric">
            <h3>上游</h3>
            <div class="row">
              <div class="field"><label for="bu">接口地址</label><input id="bu" v-model="cfg.base_url" class="input mono-input" placeholder="https://api.skillhub.cn" /></div>
              <div class="field"><label for="ak">团队密钥（可选）</label><input id="ak" v-model="cfg.api_key" class="input mono-input" type="password" autocomplete="off" placeholder="不填则使用公开接口" /><span class="hint">只发给上面的接口域名，不会带到下载跳转的对象存储。</span></div>
            </div>
          </template>

          <!-- 通用 JSON -->
          <template v-else>
            <h3>请求</h3>
            <div class="row">
              <div class="field" style="flex: 0 0 110px"><label for="mt">方法</label><select id="mt" v-model="cfg.method" class="select"><option value="">GET</option><option>GET</option><option>POST</option><option>PUT</option></select></div>
              <div class="field" style="flex: 1 1 320px"><label for="ur">列表接口地址（仅 HTTPS）</label><input id="ur" v-model="cfg.url" class="input mono-input" placeholder="https://<上游域名>/<列表接口路径>" /></div>
            </div>
            <div class="field"><label for="bd">请求体模板（可用 {page} {page_size} {keyword} {category}）</label><textarea id="bd" v-model="texts.body" class="textarea mono-input" rows="4" placeholder='{ "page": "{page}", "page_size": "{page_size}" }' /></div>
            <div class="row">
              <div class="field"><label for="qr">查询参数（JSON，可选）</label><textarea id="qr" v-model="texts.query" class="textarea mono-input" rows="2" /></div>
              <div class="field"><label for="hd">请求头（JSON，可选）</label><textarea id="hd" v-model="texts.headers" class="textarea mono-input" rows="2" placeholder='{ "Authorization": "Bearer …" }' /><span class="hint">保存后含 token / key 的值显示为 ***，不会被覆盖。</span></div>
            </div>

            <h3>分页与节奏</h3>
            <div class="row">
              <div class="field"><label for="ip">条目数组路径</label><input id="ip" v-model="cfg.items_path" class="input mono-input" placeholder="/data/items" /></div>
              <div class="field"><label for="tp">总数路径</label><input id="tp" v-model="cfg.total_path" class="input mono-input" placeholder="/data/total" /></div>
            </div>
            <div class="row">
              <div class="field"><label for="ps">每页条数</label><input id="ps" v-model="cfg.page_size" class="input" inputmode="numeric" placeholder="20" /></div>
              <div class="field"><label for="pt">起始页码</label><input id="pt" v-model="cfg.page_start" class="input" inputmode="numeric" placeholder="1" /></div>
              <div class="field"><label for="ri">页间隔（毫秒）</label><input id="ri" v-model="cfg.request_interval_ms" class="input" inputmode="numeric" placeholder="800" /></div>
              <div class="field"><label for="mp">单次最多页数</label><input id="mp" v-model="cfg.max_pages" class="input" inputmode="numeric" placeholder="5" /></div>
            </div>

            <div class="map-h">
              <h3>字段映射</h3>
              <span class="hint">JSON 指针相对每个条目；用 [&quot;/a&quot;,&quot;/b&quot;] 表示取第一个非空值；=字面量</span>
            </div>
            <div class="tbl-wrap map">
              <table class="tbl">
                <thead><tr><th>目标字段</th><th>JSON 指针</th><th>试采命中</th><th /></tr></thead>
                <tbody>
                  <tr v-for="(r, i) in rows" :key="i">
                    <td><input v-model="r.field" class="input mono-input" list="fields" :aria-label="`目标字段 ${i + 1}`" /></td>
                    <td><input v-model="r.rule" class="input mono-input" :aria-label="`JSON 指针 ${i + 1}`" /></td>
                    <td>
                      <template v-for="m in (probe?.mapping ?? []).filter((m) => m.field === r.field)" :key="m.field">
                        <span class="badge" :class="m.hit ? 'badge-ok' : 'badge-bad'">{{ m.hit ? '命中' : '未命中' }}</span>
                        <div v-if="m.sample" class="sample">{{ m.sample }}</div>
                      </template>
                    </td>
                    <td><button class="btn btn-sm" type="button" :aria-label="`删除映射 ${r.field}`" @click="rows.splice(i, 1)">删除</button></td>
                  </tr>
                </tbody>
              </table>
              <datalist id="fields"><option v-for="f in MAPPING_FIELDS" :key="f" :value="f" /></datalist>
            </div>
            <div><button class="btn btn-sm" type="button" @click="rows.push({ field: '', rule: '' })">添加字段</button></div>

            <div class="row">
              <div class="field"><label for="ou">来源页链接模板</label><input id="ou" v-model="cfg.origin_url_template" class="input mono-input" placeholder="https://…/{upstream_id}" /></div>
              <div class="field"><label for="cp">连接器配置路径（可选）</label><input id="cp" v-model="cfg.config_path" class="input mono-input" placeholder="/config" /></div>
            </div>
          </template>

          <details class="raw" @toggle="openRaw">
            <summary>高级：直接编辑完整配置 JSON</summary>
            <textarea v-model="texts.raw" class="textarea mono-input" rows="10" aria-label="完整配置 JSON" />
            <button class="btn btn-sm" type="button" @click="applyRaw">载入到表单</button>
          </details>

          <div class="actions">
            <button class="btn" type="button" :disabled="busy" @click="onProbe">{{ busy ? '处理中…' : '保存并试采一页' }}</button>
            <button class="btn" type="button" :disabled="busy" @click="onSave">保存配置</button>
            <button class="btn btn-primary" type="button" :disabled="busy || !form.enabled" @click="onSaveAndRun">保存并采集</button>
          </div>

          <section v-if="probe" class="probe" aria-live="polite">
            <h3>试采预览</h3>
            <div v-if="!probe.ok" class="error-box">{{ probe.error }}</div>
            <template v-else>
              <p class="summary">本页 {{ probe.count }} 条<template v-if="probe.total != null"> · 共 {{ probe.total }} 条</template><template v-if="probe.hasMore"> · 还有下一页</template><template v-if="probe.skipped"> · 跳过 {{ probe.skipped }} 条</template></p>
              <div v-if="!probe.count" class="notice">接口返回了数据，但没有解析出任何条目：检查「条目数组路径」和「name」映射。</div>
              <div v-if="first" class="two">
                <div><div class="label">上游原文（第 1 条）</div><pre class="pre">{{ JSON.stringify(first.raw, null, 2) }}</pre></div>
                <div>
                  <div class="label">规范化结果</div>
                  <div class="norm">
                    <div class="kv">
                      <span>名称</span><span>{{ first.normalized.name }}</span>
                      <span>标识</span><span class="mono">{{ first.normalized.namespace }} / {{ first.normalized.slug }}</span>
                      <span>摘要</span><span>{{ first.normalized.summary || '—' }}</span>
                      <span>分类</span><span>{{ first.normalized.category ?? '—' }}</span>
                      <span>标签</span><span>{{ first.normalized.tags.join(' · ') || '—' }}</span>
                      <span>版本</span><span>{{ first.normalized.version ?? '—' }}</span>
                      <span>连接配置</span><span>{{ (first.normalized.metadata as any).configs?.length ?? 0 }} 份</span>
                    </div>
                  </div>
                </div>
              </div>
              <details v-if="probe.response" class="raw"><summary>完整响应</summary><pre class="pre">{{ JSON.stringify(probe.response, null, 2) }}</pre></details>
            </template>
          </section>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.list { flex: 1 1 380px; max-width: 460px; min-width: 0; display: flex; flex-direction: column; gap: 10px; }
.editor { flex: 999 1 560px; min-width: 0; }
.src { padding: 14px 16px; display: flex; flex-direction: column; gap: 6px; border-width: 2px; }
.src.on { border-color: var(--ink); }
.head { display: flex; flex-wrap: wrap; align-items: center; gap: 6px 10px; border: 0; background: transparent; padding: 0; text-align: left; cursor: pointer; min-height: 32px; }
.head b { font-size: 15px; }
.head .badge { margin-left: auto; }
.meta { font-size: 13px; color: var(--ink-2); }
.btns { display: flex; gap: 8px; margin-top: 4px; }
.kind { padding: 1px 8px; border-radius: 5px; font-size: 12px; font-weight: 500; }
.k-skill { background: #e0e9fb; color: #1f3f8f; }
.k-expert { background: #ebe5fa; color: #43308a; }
.k-connector { background: #fcefd3; color: #7a4b00; }
.title { display: flex; flex-wrap: wrap; align-items: center; gap: 8px 12px; }
.title h2 { font-size: 20px; font-weight: 700; }
.push { margin-left: auto; }
h3 { font-size: 15px; font-weight: 700; }
.map-h { display: flex; flex-wrap: wrap; align-items: baseline; justify-content: space-between; gap: 6px; }
.map { border: 1px solid var(--line); border-radius: 12px; }
.map .input { min-height: 38px; padding: 6px 10px; }
.sample { font-size: 12px; color: var(--ink-2); max-width: 180px; overflow-wrap: anywhere; }
.raw summary { cursor: pointer; min-height: 32px; color: var(--ink-2); font-size: 14px; }
.raw .textarea { margin: 8px 0; }
.actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 8px; padding-top: 16px; border-top: 1px solid #f0f0ee; }
.probe { display: flex; flex-direction: column; gap: 12px; }
.summary { font-size: 14px; color: var(--ink-2); }
.two { display: flex; flex-wrap: wrap; gap: 12px; }
.two > div { flex: 1 1 260px; min-width: 0; }
.two .pre { max-height: 300px; }
.label { font-size: 13px; color: var(--ink-2); margin-bottom: 6px; }
.norm { padding: 14px 16px; border: 1px solid var(--line); border-radius: 12px; background: #fbfcfb; }
</style>
