<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage, formatRelativeTime } from '../api';
import type { NavKey } from '../components/CommandPalette.vue';
import Icon from '../components/Icon.vue';
import { copyWithToast } from '../toast';
import type { AuditRow, ClientRow, SenderConfig, SignupMethods, Stats } from '../types';

const emit = defineEmits<{
  navigate: [page: NavKey];
  openUser: [uid: string];
}>();

const stats = ref<Stats | null>(null);
const clients = ref<ClientRow[]>([]);
const sender = ref<SenderConfig | null>(null);
const signup = ref<SignupMethods | null>(null);
const recentAudit = ref<AuditRow[]>([]);
const loading = ref(false);
const error = ref('');

const KIND_LABEL: Record<string, string> = { email: '邮箱', phone: '手机号' };
const ACTION_SHORT: Record<string, string> = {
  'user.create': '创建用户',
  'user.disabled': '停用用户',
  'user.active': '启用用户',
  'user.rename': '修改用户名',
  'user.revoke_sessions': '下线全部设备',
  'client.create': '新增应用',
  'client.enable': '启用应用',
  'client.disable': '停用应用',
  'client.rename': '重命名应用',
  'sender.update': '更新发信通道',
  'signup_methods.update': '调整注册方式',
  'admin.password_change': '修改管理员密码',
};

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

async function load() {
  loading.value = true;
  error.value = '';
  try {
    const [s, c, sd, sm, au] = await Promise.all([
      api.stats(),
      api.clients(),
      api.getSender(),
      api.getSignupMethods(),
      api.audit(6),
    ]);
    stats.value = s;
    clients.value = c.items;
    sender.value = sd;
    signup.value = sm;
    recentAudit.value = au.items;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
}

onMounted(load);

const activeRatio = computed(() => {
  const s = stats.value;
  if (!s || s.users.total === 0) return 100;
  return Math.round((s.users.active / s.users.total) * 100);
});

const loginRatio = computed(() => {
  const s = stats.value;
  if (!s || s.users.active === 0) return 0;
  return Math.min(100, Math.round((s.users.logged_in_24h / s.users.active) * 100));
});

const maxIpCount = computed(() => {
  const ips = stats.value?.codes_24h.top_ips ?? [];
  return Math.max(1, ...ips.map((r) => r.count));
});

const totalClientSessions = computed(() =>
  Math.max(1, clients.value.reduce((acc, c) => acc + c.active_sessions, 0)),
);

const topClients = computed(() =>
  [...clients.value].sort((a, b) => b.active_sessions - a.active_sessions).slice(0, 5),
);

const senderLabel = computed(() => {
  if (!sender.value) return '—';
  if (sender.value.provider === 'smtp') return 'SMTP 邮件服务';
  if (sender.value.provider === 'tencent') return '腾讯云 SMS / SES';
  return '控制台日志（开发测试）';
});

const signupSummary = computed(() => {
  if (!signup.value) return '—';
  const list: string[] = [];
  if (signup.value.email) list.push('邮箱');
  if (signup.value.phone) list.push('手机号');
  return list.length ? list.join(' · ') : '已关闭';
});
</script>

<template>
  <div class="page-head">
    <div>
      <h1>概览</h1>
      <p>实时掌握账号增长、设备会话、验证码发信健康度与近期安全操作。</p>
    </div>
    <div class="page-actions">
      <button type="button" class="btn" :disabled="loading" @click="load">
        <Icon name="refresh" :size="14" />
        <span>{{ loading ? '刷新中…' : '刷新数据' }}</span>
      </button>
      <button type="button" class="btn primary" @click="emit('navigate', 'users')">
        <Icon name="users" :size="14" />
        <span>管理用户</span>
      </button>
    </div>
  </div>

  <div v-if="error" class="banner danger" style="margin-bottom: 16px" role="alert">
    <Icon name="alert" :size="15" class="banner-ico" />
    <span>{{ error }}</span>
  </div>

  <!-- 控制台日志模式警示条 -->
  <div
    v-if="sender && sender.provider === 'console'"
    class="banner warn"
    style="margin-bottom: 16px"
  >
    <Icon name="alert" :size="16" class="banner-ico" />
    <div>
      <strong>当前发信通道处于「控制台日志」模式</strong>，验证码不会真实发送给用户。生产环境请配置 SMTP 或腾讯云通道。
    </div>
    <button type="button" class="btn sm banner-act" @click="emit('navigate', 'sender')">前往配置</button>
  </div>

  <!-- 通道与注册策略快览条 -->
  <section v-if="sender && signup" class="card status-strip">
    <div class="strip-item">
      <span class="strip-k">主发信通道</span>
      <span class="strip-v">
        <span class="tag" :class="sender.provider === 'console' ? 'warn' : 'ok'">
          <i class="dot" />{{ senderLabel }}
        </span>
      </span>
    </div>
    <div class="strip-sep" />
    <div class="strip-item">
      <span class="strip-k">阿里云短信认证 (PNVS)</span>
      <span class="strip-v">
        <span class="tag" :class="sender.pnvs_configured ? 'ok' : ''">
          <i class="dot" />{{ sender.pnvs_configured ? '已接入' : '未配置' }}
        </span>
      </span>
    </div>
    <div class="strip-sep" />
    <div class="strip-item">
      <span class="strip-k">开放注册方式</span>
      <span class="strip-v">{{ signupSummary }}</span>
    </div>
    <div class="strip-sep" />
    <div class="strip-item">
      <span class="strip-k">已登记应用</span>
      <span class="strip-v num">
        {{ clients.filter((c) => c.enabled).length }} / {{ clients.length }} 启用
      </span>
    </div>
    <button type="button" class="btn ghost sm strip-act" @click="emit('navigate', 'sender')">
      <span>调整设置</span>
      <Icon name="chevron-right" :size="14" />
    </button>
  </section>

  <!-- 骨架屏 -->
  <div v-if="!stats && loading" class="kpi-grid">
    <div v-for="i in 4" :key="i" class="card kpi-card">
      <div class="skel" style="height: 14px; width: 45%" />
      <div class="skel" style="height: 30px; width: 60%; margin: 6px 0" />
      <div class="skel" style="height: 12px; width: 80%" />
    </div>
  </div>

  <!-- KPI 指标卡 -->
  <div v-if="stats" class="kpi-grid">
    <div class="card kpi-card clickable-card" @click="emit('navigate', 'users')">
      <div class="kpi-head">
        <span>注册用户总数</span>
        <span class="tag" :class="stats.users.disabled > 0 ? 'warn' : 'ok'">正常 {{ activeRatio }}%</span>
      </div>
      <div class="kpi-val num">{{ stats.users.total.toLocaleString() }}</div>
      <div class="kpi-bar"><i :style="{ width: `${activeRatio}%` }" /></div>
      <div class="kpi-sub">
        <span>正常 <strong class="num">{{ stats.users.active }}</strong></span>
        <span>·</span>
        <span>已停用 <strong class="num">{{ stats.users.disabled }}</strong></span>
      </div>
    </div>

    <div class="card kpi-card">
      <div class="kpi-head">
        <span>24 小时新增用户</span>
        <span class="tag coral">7 天 +{{ stats.users.new_7d }}</span>
      </div>
      <div class="kpi-val num">{{ stats.users.new_24h.toLocaleString() }}</div>
      <div class="kpi-sub">
        <span>24 小时登录活跃 <strong class="num">{{ stats.users.logged_in_24h }}</strong> 人（占正常用户 {{ loginRatio }}%）</span>
      </div>
    </div>

    <div class="card kpi-card clickable-card" @click="emit('navigate', 'clients')">
      <div class="kpi-head">
        <span>当前有效会话</span>
        <Icon name="laptop" :size="15" class="faint" />
      </div>
      <div class="kpi-val num">{{ stats.active_sessions.toLocaleString() }}</div>
      <div class="kpi-sub">
        <span>未过期且未吊销的设备刷新令牌总量</span>
      </div>
    </div>

    <div class="card kpi-card clickable-card" @click="emit('navigate', 'sender')">
      <div class="kpi-head">
        <span>24 小时验证码发送</span>
        <Icon name="mail" :size="15" class="faint" />
      </div>
      <div class="kpi-val num">{{ stats.codes_24h.total.toLocaleString() }}</div>
      <div class="kpi-sub">
        <template v-if="stats.codes_24h.by_kind.length">
          <span v-for="k in stats.codes_24h.by_kind" :key="k.kind" class="tag">
            {{ KIND_LABEL[k.kind] ?? k.kind }} <strong class="num">{{ k.count }}</strong>
          </span>
        </template>
        <span v-else>最近 24 小时暂无发信记录</span>
      </div>
    </div>
  </div>

  <!-- 下方双栏详情 -->
  <div v-if="stats" class="lower-grid">
    <!-- 左栏：24 小时验证码来源 IP -->
    <section class="card panel-card">
      <div class="panel-head">
        <div>
          <h2>24 小时验证码发送来源 IP（前 5）</h2>
          <p class="hint">单个 IP 发送量异常偏高时，可能存在短信轰炸风险，建议调低 <code>ACCOUNT_DAILY_PER_IP</code> 或在网关层封禁。</p>
        </div>
      </div>

      <div v-if="stats.codes_24h.top_ips.length" class="ip-list">
        <div v-for="(row, idx) in stats.codes_24h.top_ips" :key="row.ip" class="ip-row">
          <span class="ip-rank num">{{ idx + 1 }}</span>
          <div class="ip-main">
            <div class="ip-top">
              <span class="mono ip-addr">{{ row.ip }}</span>
              <button
                type="button"
                class="copy-mini"
                title="复制 IP"
                aria-label="复制 IP"
                @click="copyWithToast(row.ip, `已复制 IP ${row.ip}`)"
              >
                <Icon name="copy" :size="12" />
              </button>
              <span v-if="row.count >= 20" class="tag warn">偏高</span>
              <strong class="num ip-cnt">{{ row.count }} 次</strong>
            </div>
            <div class="bar-track">
              <i
                :class="{ hot: row.count >= 20 }"
                :style="{ width: `${Math.max(6, Math.round((row.count / maxIpCount) * 100))}%` }"
              />
            </div>
          </div>
        </div>
      </div>

      <div v-else class="empty-state compact">
        <Icon name="shield" :size="22" class="faint" />
        <span>最近 24 小时没有验证码发送记录</span>
      </div>
    </section>

    <!-- 右栏：接入应用活跃分布 + 最近审计 -->
    <div class="right-stack">
      <section class="card panel-card">
        <div class="panel-head">
          <h2>应用会话分布</h2>
          <button type="button" class="btn ghost sm" @click="emit('navigate', 'clients')">
            <span>全部应用</span>
            <Icon name="chevron-right" :size="13" />
          </button>
        </div>

        <div v-if="topClients.length" class="app-dist">
          <div v-for="c in topClients" :key="c.client_id" class="app-row">
            <div class="app-row-top">
              <div class="app-name">
                <strong>{{ c.name }}</strong>
                <span class="mono faint">{{ c.client_id }}</span>
              </div>
              <span class="num app-cnt">{{ c.active_sessions }}</span>
            </div>
            <div class="bar-track thin">
              <i :style="{ width: `${Math.round((c.active_sessions / totalClientSessions) * 100)}%` }" />
            </div>
          </div>
        </div>
        <div v-else class="empty-state compact">暂无已登记应用</div>
      </section>

      <section class="card panel-card">
        <div class="panel-head">
          <h2>最近管理员操作</h2>
          <button type="button" class="btn ghost sm" @click="emit('navigate', 'audit')">
            <span>完整日志</span>
            <Icon name="chevron-right" :size="13" />
          </button>
        </div>

        <div v-if="recentAudit.length" class="mini-audit">
          <div v-for="(row, i) in recentAudit" :key="i" class="ma-row">
            <div class="ma-left">
              <span class="tag">{{ ACTION_SHORT[row.action] ?? row.action }}</span>
              <button
                v-if="row.target && UUID_RE.test(row.target)"
                type="button"
                class="mono target-link"
                title="查看该用户详情"
                @click="emit('openUser', row.target)"
              >
                {{ row.target.slice(0, 8) }}…
              </button>
              <span v-else-if="row.target" class="mono faint truncate">{{ row.target }}</span>
            </div>
            <div class="ma-right">
              <span class="faint">{{ row.admin }}</span>
              <span class="faint">·</span>
              <span class="faint">{{ formatRelativeTime(row.created_at) }}</span>
            </div>
          </div>
        </div>
        <div v-else class="empty-state compact">暂无操作记录</div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.status-strip {
  padding: 12px 18px;
  margin-bottom: 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
}
.strip-item {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.strip-k {
  font-size: 11.5px;
  color: var(--ink-3);
  font-weight: 600;
}
.strip-v {
  font-size: 13px;
  font-weight: 620;
  color: var(--ink);
}
.strip-sep {
  width: 1px;
  height: 28px;
  background: var(--line);
}
.strip-act {
  margin-left: auto;
}

.kpi-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
}
.kpi-card {
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  transition: border-color 0.14s, transform 0.14s;
}
.clickable-card {
  cursor: pointer;
}
.clickable-card:hover {
  border-color: var(--line-strong);
  transform: translateY(-1px);
}
.kpi-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-size: 12.5px;
  font-weight: 600;
  color: var(--ink-2);
}
.kpi-val {
  font-size: 28px;
  font-weight: 750;
  letter-spacing: -0.025em;
  line-height: 1.15;
}
.kpi-bar {
  height: 4px;
  border-radius: 99px;
  background: var(--canvas-soft);
  overflow: hidden;
}
.kpi-bar i {
  display: block;
  height: 100%;
  border-radius: 99px;
  background: var(--ink);
}
.kpi-sub {
  font-size: 12px;
  color: var(--ink-3);
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.kpi-sub strong {
  color: var(--ink-2);
}

.lower-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.15fr) minmax(0, 1fr);
  gap: 14px;
  margin-top: 16px;
}
.right-stack {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.panel-card {
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.panel-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.panel-head h2 {
  font-size: 14.5px;
  font-weight: 680;
}
.panel-head .hint {
  margin-top: 4px;
}
.panel-head code {
  font-family: var(--mono);
  font-size: 11.5px;
  padding: 1px 5px;
  border-radius: 4px;
  background: var(--canvas-soft);
  color: var(--ink-2);
}

.ip-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.ip-row {
  display: flex;
  align-items: center;
  gap: 12px;
}
.ip-rank {
  width: 22px;
  height: 22px;
  border-radius: 6px;
  background: var(--canvas-soft);
  color: var(--ink-2);
  font-size: 11.5px;
  font-weight: 700;
  display: grid;
  place-items: center;
  flex: none;
}
.ip-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.ip-top {
  display: flex;
  align-items: center;
  gap: 6px;
}
.ip-addr {
  font-weight: 600;
  color: var(--ink);
}
.ip-cnt {
  margin-left: auto;
  font-size: 12.5px;
}

.bar-track {
  height: 6px;
  border-radius: 99px;
  background: var(--canvas-soft);
  overflow: hidden;
}
.bar-track.thin {
  height: 5px;
}
.bar-track i {
  display: block;
  height: 100%;
  border-radius: 99px;
  background: var(--ink);
  transition: width 0.3s var(--ease);
}
.bar-track i.hot {
  background: var(--coral-grad);
}

.app-dist {
  display: flex;
  flex-direction: column;
  gap: 11px;
}
.app-row {
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.app-row-top {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
}
.app-name {
  display: flex;
  align-items: baseline;
  gap: 7px;
  min-width: 0;
  flex-wrap: wrap;
}
.app-name strong {
  font-size: 13px;
  font-weight: 620;
}
.app-cnt {
  font-size: 12.5px;
  font-weight: 650;
  color: var(--ink-2);
}

.mini-audit {
  display: flex;
  flex-direction: column;
}
.ma-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 0;
  border-bottom: 1px solid var(--line);
}
.ma-row:last-child {
  border-bottom: 0;
  padding-bottom: 0;
}
.ma-row:first-child {
  padding-top: 0;
}
.ma-left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}
.ma-right {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  flex: none;
}
.target-link {
  color: var(--info);
  text-decoration: underline;
  text-underline-offset: 2px;
}
.target-link:hover {
  opacity: 0.8;
}

.empty-state.compact {
  padding: 26px 12px;
}

@media (max-width: 1060px) {
  .kpi-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .lower-grid {
    grid-template-columns: 1fr;
  }
}
@media (max-width: 600px) {
  .kpi-grid {
    grid-template-columns: 1fr;
  }
  .strip-sep {
    display: none;
  }
}
</style>
