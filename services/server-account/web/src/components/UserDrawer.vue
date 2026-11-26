<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue';
import { api, errorMessage, formatTime } from '../api';
import ConfirmDialog from './ConfirmDialog.vue';
import Icon from './Icon.vue';
import { copyWithToast, showToast } from '../toast';
import type { UserDetail } from '../types';

const props = defineProps<{ uid: string }>();
const emit = defineEmits<{ close: []; changed: [] }>();

const detail = ref<UserDetail | null>(null);
const loading = ref(false);
const error = ref('');

type ModalMode = 'rename' | 'revoke' | 'toggle' | null;
const modal = ref<ModalMode>(null);
const modalBusy = ref(false);
const modalError = ref('');

const KIND_LABEL: Record<string, string> = { email: '邮箱', phone: '手机号' };

async function load() {
  loading.value = true;
  error.value = '';
  try {
    detail.value = await api.user(props.uid);
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    loading.value = false;
  }
}

watch(() => props.uid, load);
onMounted(load);

function openModal(mode: ModalMode) {
  modalError.value = '';
  modal.value = mode;
}

async function handleConfirm(inputValue: string) {
  if (!detail.value || !modal.value) return;
  modalBusy.value = true;
  modalError.value = '';
  try {
    if (modal.value === 'rename') {
      if (inputValue === detail.value.user.username) {
        modal.value = null;
        return;
      }
      await api.renameUser(props.uid, inputValue);
      showToast(`已将用户名修改为「${inputValue}」`);
    } else if (modal.value === 'revoke') {
      await api.revokeSessions(props.uid);
      showToast('已吊销该用户的全部设备会话');
    } else if (modal.value === 'toggle') {
      const next = detail.value.status === 'active' ? 'disabled' : 'active';
      await api.setUserStatus(props.uid, next);
      showToast(next === 'disabled' ? '用户已停用并强制下线' : '用户已重新启用');
    }
    modal.value = null;
    await load();
    emit('changed');
  } catch (e) {
    modalError.value = errorMessage(e);
  } finally {
    modalBusy.value = false;
  }
}

function onKey(event: KeyboardEvent) {
  if (event.key === 'Escape' && !modal.value) emit('close');
}
onMounted(() => window.addEventListener('keydown', onKey));
onUnmounted(() => window.removeEventListener('keydown', onKey));
</script>

<template>
  <div class="scrim" @click="emit('close')" />
  <aside class="drawer" role="dialog" aria-modal="true" aria-label="用户详情">
    <header class="drawer-head">
      <div class="head-left">
        <h3>用户详情</h3>
        <span v-if="detail" class="tag" :class="detail.status === 'active' ? 'ok' : 'danger'">
          <i class="dot" />{{ detail.status === 'active' ? '正常' : '已停用' }}
        </span>
      </div>
      <button type="button" class="btn ghost icon sm" aria-label="关闭抽屉" @click="emit('close')">
        <Icon name="x" :size="16" />
      </button>
    </header>

    <div class="drawer-body">
      <div v-if="error" class="banner danger" role="alert">
        <Icon name="alert" :size="15" class="banner-ico" />
        <span>{{ error }}</span>
      </div>

      <template v-if="detail">
        <!-- 用户名片卡 -->
        <section class="hero-box">
          <img
            v-if="detail.user.avatar_url"
            :src="detail.user.avatar_url"
            :alt="detail.user.username"
            class="hero-av-img"
          />
          <span v-else class="hero-av-txt">
            {{ detail.user.username.slice(0, 1).toUpperCase() }}
          </span>
          <div class="hero-info">
            <div class="hero-name-row">
              <strong>{{ detail.user.username }}</strong>
              <span class="tag">{{ detail.user.has_password ? '已设密码' : '未设密码' }}</span>
            </div>
            <div class="hero-ids">
              <span class="id-chip">
                <span class="faint">账号 ID</span>
                <span class="mono num">{{ detail.user.account_id }}</span>
                <button
                  type="button"
                  class="copy-mini"
                  title="复制账号 ID"
                  @click="copyWithToast(detail.user.account_id, '已复制账号 ID')"
                >
                  <Icon name="copy" :size="11" />
                </button>
              </span>
            </div>
          </div>
        </section>

        <!-- 基础信息 -->
        <section class="sec">
          <h4>账号资料</h4>
          <dl class="kv-grid">
            <dt>全局 UID</dt>
            <dd class="mono uid-val">
              <span>{{ detail.user.uid }}</span>
              <button
                type="button"
                class="copy-mini"
                title="复制 UID"
                @click="copyWithToast(detail.user.uid, '已复制 UID')"
              >
                <Icon name="copy" :size="12" />
              </button>
            </dd>
            <dt>注册时间</dt>
            <dd class="num">{{ formatTime(detail.user.created_at) }}</dd>
            <dt>最近登录</dt>
            <dd class="num">{{ formatTime(detail.last_login_at) }}</dd>
            <dt>头像地址</dt>
            <dd class="mono wrap">{{ detail.user.avatar_url ?? '—' }}</dd>
          </dl>
        </section>

        <!-- 已绑定标识 -->
        <section class="sec">
          <h4>已绑定登录标识</h4>
          <div v-if="detail.user.identities.length" class="ident-cards">
            <div v-for="(identity, index) in detail.user.identities" :key="index" class="ident-card">
              <Icon :name="identity.kind === 'phone' ? 'phone' : 'mail'" :size="15" class="muted" />
              <span class="tag">{{ KIND_LABEL[identity.kind] ?? identity.kind }}</span>
              <span class="mono">{{ identity.masked }}</span>
            </div>
          </div>
          <p v-else class="hint">尚未绑定邮箱或手机号（当前仅支持用户名加密码登录）。</p>
        </section>

        <!-- 有效会话 -->
        <section class="sec">
          <div class="sec-head">
            <h4>有效设备会话</h4>
            <span class="tag num">{{ detail.sessions.length }} 台</span>
          </div>
          <div v-if="detail.sessions.length" class="session-list">
            <div v-for="(s, index) in detail.sessions" :key="index" class="session-card">
              <div class="session-top">
                <span class="mono client-pill">{{ s.client_id }}</span>
                <span class="session-dev truncate">{{ s.device_name || '未命名设备' }}</span>
              </div>
              <div class="session-meta">
                <span class="mono">IP {{ s.ip ?? '—' }}</span>
                <span>·</span>
                <span class="num">登录于 {{ formatTime(s.created_at) }}</span>
              </div>
            </div>
          </div>
          <p v-else class="hint">当前没有活跃的刷新令牌会话。</p>
        </section>
      </template>

      <div v-else-if="loading" class="empty-state">正在加载用户资料…</div>
    </div>

    <footer v-if="detail" class="drawer-foot">
      <button type="button" class="btn" @click="openModal('rename')">
        <Icon name="edit" :size="14" />
        <span>改用户名</span>
      </button>
      <button type="button" class="btn" :disabled="!detail.sessions.length" @click="openModal('revoke')">
        <Icon name="log-out" :size="14" />
        <span>下线全部设备</span>
      </button>
      <button
        type="button"
        class="btn"
        :class="detail.status === 'active' ? 'danger' : 'primary'"
        @click="openModal('toggle')"
      >
        <Icon name="power" :size="14" />
        <span>{{ detail.status === 'active' ? '停用用户' : '启用用户' }}</span>
      </button>
    </footer>
  </aside>

  <!-- 改用户名弹窗 -->
  <ConfirmDialog
    v-if="modal === 'rename' && detail"
    over
    title="修改用户名"
    description="用户名对用户本人不可修改，仅用于处理违规或冒充场景。旧用户名将被释放，此操作会写入审计日志。"
    input-label="新用户名"
    input-placeholder="输入新用户名"
    input-hint="至少 2 个汉字或 3 个字母，最长 20（汉字按 2 计），不区分大小写唯一。"
    :initial-value="detail.user.username"
    :max-length="20"
    confirm-text="保存新用户名"
    :busy="modalBusy"
    :error="modalError"
    @close="modal = null"
    @confirm="handleConfirm"
  />

  <!-- 下线全部设备确认弹窗 -->
  <ConfirmDialog
    v-if="modal === 'revoke' && detail"
    over
    title="下线全部设备"
    :description="`确定吊销「${detail.user.username}」的全部 ${detail.sessions.length} 个有效会话？所有设备将需要重新登录（已发出的访问令牌最长再存活约 30 分钟）。`"
    confirm-text="确认全部下线"
    :busy="modalBusy"
    :error="modalError"
    @close="modal = null"
    @confirm="handleConfirm"
  />

  <!-- 停用/启用确认弹窗 -->
  <ConfirmDialog
    v-if="modal === 'toggle' && detail"
    over
    :title="detail.status === 'active' ? '停用该用户' : '重新启用用户'"
    :description="
      detail.status === 'active'
        ? `停用「${detail.user.username}」后，该账号将无法登录，且全部设备会话立即下线。`
        : `确定重新启用「${detail.user.username}」并恢复正常登录权限？`
    "
    :confirm-text="detail.status === 'active' ? '确认停用' : '确认启用'"
    :danger="detail.status === 'active'"
    :busy="modalBusy"
    :error="modalError"
    @close="modal = null"
    @confirm="handleConfirm"
  />
</template>

<style scoped>
.head-left {
  display: flex;
  align-items: center;
  gap: 8px;
}
.head-left h3 {
  font-size: 16px;
  font-weight: 700;
}

.hero-box {
  padding: 15px 16px;
  border-radius: var(--r-md);
  background: var(--surface-muted);
  border: 1px solid var(--line);
  display: flex;
  align-items: center;
  gap: 13px;
}
.hero-av-img,
.hero-av-txt {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  flex: none;
}
.hero-av-img {
  object-fit: cover;
  border: 1px solid var(--line);
}
.hero-av-txt {
  display: grid;
  place-items: center;
  background: var(--btn-primary-bg);
  color: var(--btn-primary-ink);
  font-size: 17px;
  font-weight: 700;
}
.hero-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.hero-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.hero-name-row strong {
  font-size: 16px;
  font-weight: 700;
}
.hero-ids {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.id-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 6px;
  background: var(--surface);
  border: 1px solid var(--line);
}

.sec {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.sec h4 {
  font-size: 13px;
  font-weight: 680;
  color: var(--ink-2);
}
.sec-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.kv-grid {
  display: grid;
  grid-template-columns: 78px minmax(0, 1fr);
  gap: 9px 12px;
  padding: 12px 14px;
  border-radius: var(--r-md);
  border: 1px solid var(--line);
  background: var(--surface);
}
.kv-grid dt {
  color: var(--ink-3);
  font-size: 12.5px;
}
.kv-grid dd {
  font-size: 13px;
  min-width: 0;
}
.uid-val {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  word-break: break-all;
}
.wrap {
  word-break: break-all;
}

.ident-cards {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.ident-card {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: var(--r-sm);
  border: 1px solid var(--line);
  background: var(--surface);
}

.session-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.session-card {
  padding: 11px 13px;
  border-radius: var(--r-sm);
  border: 1px solid var(--line);
  background: var(--surface);
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.session-top {
  display: flex;
  align-items: center;
  gap: 8px;
}
.client-pill {
  padding: 1px 7px;
  border-radius: 5px;
  background: var(--canvas-soft);
  color: var(--ink);
  font-weight: 600;
}
.session-dev {
  font-size: 13px;
  font-weight: 550;
}
.session-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--ink-3);
  flex-wrap: wrap;
}
</style>
