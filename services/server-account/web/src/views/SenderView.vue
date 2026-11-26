<script setup lang="ts">
import { onMounted, onUnmounted, reactive, ref } from 'vue';
import { api, errorMessage } from '../api';
import Icon from '../components/Icon.vue';
import { showToast } from '../toast';
import type { SignupMethods, SmtpConfig } from '../types';

const emit = defineEmits<{ updated: [] }>();

const loading = ref(true);
const saving = ref(false);
const testing = ref(false);
const showSmtpPwd = ref(false);
const message = ref<{ type: 'ok' | 'danger' | 'info'; text: string } | null>(null);

const provider = ref<'console' | 'smtp' | 'tencent'>('console');
const pnvsConfigured = ref(false);
const smtp = reactive<SmtpConfig>({
  host: '',
  port: 465,
  secure: true,
  username: '',
  password: '',
  has_password: false,
  from_name: '相伴',
  from_email: '',
});

const signup = reactive<SignupMethods>({ email: true, phone: false });
const savingSignup = ref(false);
const signupMessage = ref<{ type: 'ok' | 'danger'; text: string } | null>(null);

const testRecipient = ref('');
const showTestModal = ref(false);
const testError = ref('');

const presets = [
  { label: '网易 163', host: 'smtp.163.com', port: 465, secure: true },
  { label: 'QQ 邮箱', host: 'smtp.qq.com', port: 465, secure: true },
  { label: '腾讯企业邮', host: 'smtp.exmail.qq.com', port: 465, secure: true },
  { label: 'Gmail', host: 'smtp.gmail.com', port: 587, secure: false },
  { label: 'Resend', host: 'smtp.resend.com', port: 465, secure: true },
];

const providers = [
  {
    value: 'console' as const,
    title: '控制台日志（开发测试）',
    badge: '本地调试',
    desc: '验证码只打印在服务器终端或日志中，不通过真实网络发信，严禁用于生产环境。',
  },
  {
    value: 'smtp' as const,
    title: 'SMTP 邮件服务',
    badge: '推荐',
    desc: '使用任意标准 SMTP 邮箱（163、QQ、Gmail、企业邮、Resend 等）直接发送验证码邮件，保存后实时热切换。',
  },
  {
    value: 'tencent' as const,
    title: '腾讯云服务（SMS / SES）',
    badge: '环境变量',
    desc: '通过服务器环境变量配置 TENCENT_SECRET_ID 与短信 / 邮件模板参数，适合同时使用腾讯云短信与邮件。',
  },
];

function applyPreset(p: (typeof presets)[number]) {
  smtp.host = p.host;
  smtp.port = p.port;
  smtp.secure = p.secure;
  showToast(`已套用「${p.label}」服务器配置，请填写账号与授权码`, 'info');
}

async function load() {
  loading.value = true;
  message.value = null;
  try {
    const [data, methods] = await Promise.all([api.getSender(), api.getSignupMethods()]);
    Object.assign(signup, methods);
    provider.value = data.provider;
    pnvsConfigured.value = data.pnvs_configured;
    if (data.smtp) {
      smtp.host = data.smtp.host || '';
      smtp.port = data.smtp.port || 465;
      smtp.secure = data.smtp.secure ?? smtp.port === 465;
      smtp.username = data.smtp.username || '';
      smtp.has_password = data.smtp.has_password ?? false;
      smtp.from_name = data.smtp.from_name || '相伴';
      smtp.from_email = data.smtp.from_email || '';
      smtp.password = '';
    }
  } catch (err) {
    message.value = { type: 'danger', text: errorMessage(err) };
  } finally {
    loading.value = false;
  }
}

async function save() {
  saving.value = true;
  message.value = null;
  try {
    await api.updateSender({
      provider: provider.value,
      smtp: provider.value === 'smtp' ? smtp : undefined,
    });
    if (smtp.password) {
      smtp.has_password = true;
      smtp.password = '';
    }
    message.value = { type: 'ok', text: '发信通道设置已保存，并已实时热重载生效。' };
    showToast('发信设置已保存');
    emit('updated');
  } catch (err) {
    message.value = { type: 'danger', text: errorMessage(err) };
  } finally {
    saving.value = false;
  }
}

async function saveSignup() {
  savingSignup.value = true;
  signupMessage.value = null;
  try {
    Object.assign(signup, await api.updateSignupMethods({ email: signup.email, phone: signup.phone }));
    signupMessage.value = { type: 'ok', text: '注册方式已保存，各端登录页刷新后即生效。' };
    showToast('注册方式已更新');
  } catch (err) {
    signupMessage.value = { type: 'danger', text: errorMessage(err) };
  } finally {
    savingSignup.value = false;
  }
}

function openTestModal() {
  testError.value = '';
  showTestModal.value = true;
}

async function sendTest() {
  testError.value = '';
  if (!testRecipient.value.trim() || !testRecipient.value.includes('@')) {
    testError.value = '请输入有效的测试收件人邮箱地址';
    return;
  }
  testing.value = true;
  try {
    const res = await api.testSender({
      recipient: testRecipient.value.trim(),
      smtp: provider.value === 'smtp' ? smtp : undefined,
    });
    showTestModal.value = false;
    showToast(res.message || '测试邮件已发送，请查收收件箱或垃圾箱', 'ok', 3400);
  } catch (err) {
    testError.value = errorMessage(err);
  } finally {
    testing.value = false;
  }
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape' && showTestModal.value && !testing.value) showTestModal.value = false;
}
onMounted(() => {
  load();
  window.addEventListener('keydown', onKey);
});
onUnmounted(() => window.removeEventListener('keydown', onKey));
</script>

<template>
  <div class="sender-wrap">
    <div class="page-head">
      <div>
        <h1>发信与注册设置</h1>
        <p>管理注册 / 登录验证码的邮件与短信发信通道，以及对新用户开放的注册方式。</p>
      </div>
      <div class="page-actions">
        <button v-if="provider === 'smtp'" type="button" class="btn" :disabled="loading" @click="openTestModal">
          <Icon name="send" :size="14" />
          <span>测试发信</span>
        </button>
        <button type="button" class="btn primary" :disabled="saving || loading" @click="save">
          <Icon name="check" :size="14" />
          <span>{{ saving ? '保存中…' : '保存发信设置' }}</span>
        </button>
      </div>
    </div>

    <div v-if="message" class="banner" :class="message.type" style="margin-bottom: 16px" role="status">
      <Icon :name="message.type === 'danger' ? 'alert' : 'check'" :size="15" class="banner-ico" />
      <span>{{ message.text }}</span>
    </div>

    <div v-if="loading" class="card form-card">
      <div class="skel" style="height: 20px; width: 32%; margin-bottom: 14px" />
      <div class="skel" style="height: 64px; margin-bottom: 10px" />
      <div class="skel" style="height: 64px" />
    </div>

    <template v-else>
      <!-- 主发信通道卡片 -->
      <section class="card form-card">
        <div class="card-head">
          <div>
            <h2>主发信通道</h2>
            <p class="hint">配置以主密钥加密存于数据库，重启后优先于 <code>ACCOUNT_SENDER</code> 环境变量。</p>
          </div>
          <span class="tag" :class="provider === 'console' ? 'warn' : 'ok'">
            <i class="dot" />{{ provider === 'smtp' ? 'SMTP 已选' : provider === 'tencent' ? '腾讯云已选' : '仅日志输出' }}
          </span>
        </div>

        <div class="provider-grid">
          <button
            v-for="item in providers"
            :key="item.value"
            type="button"
            class="provider-card"
            :class="{ selected: provider === item.value }"
            @click="provider = item.value"
          >
            <div class="pc-top">
              <span class="pc-radio" aria-hidden="true" />
              <strong>{{ item.title }}</strong>
              <span class="tag" :class="item.value === 'smtp' ? 'coral' : ''">{{ item.badge }}</span>
            </div>
            <p>{{ item.desc }}</p>
          </button>
        </div>

        <!-- SMTP 配置表单 -->
        <div v-if="provider === 'smtp'" class="smtp-box">
          <div class="smtp-head">
            <h3>SMTP 服务器参数</h3>
            <div class="preset-bar">
              <span class="faint" style="font-size: 12px">快捷预设：</span>
              <button
                v-for="p in presets"
                :key="p.label"
                type="button"
                class="preset-chip"
                @click="applyPreset(p)"
              >
                {{ p.label }}
              </button>
            </div>
          </div>

          <div class="form-grid">
            <label class="field">
              <span class="field-label">SMTP 服务器地址 *</span>
              <input v-model="smtp.host" class="input mono" placeholder="例如 smtp.163.com" />
            </label>

            <label class="field">
              <span class="field-label">端口 *</span>
              <input v-model.number="smtp.port" type="number" class="input num" placeholder="465 或 587" />
            </label>

            <div class="field">
              <span class="field-label">传输加密方式</span>
              <div class="seg-pills full-seg">
                <button
                  type="button"
                  :class="{ active: smtp.secure === true }"
                  @click="smtp.secure = true"
                >
                  SSL / TLS（推荐 465）
                </button>
                <button
                  type="button"
                  :class="{ active: smtp.secure === false }"
                  @click="smtp.secure = false"
                >
                  STARTTLS（587 / 25）
                </button>
              </div>
            </div>

            <label class="field">
              <span class="field-label">发信人显示名称</span>
              <input v-model="smtp.from_name" class="input" placeholder="例如 相伴 或 小伴壁纸" />
            </label>

            <label class="field">
              <span class="field-label">发信账号 (Username) *</span>
              <input v-model="smtp.username" class="input mono" placeholder="例如 yourname@163.com" />
            </label>

            <label class="field">
              <span class="field-label">发信人邮箱 (From Email) *</span>
              <input v-model="smtp.from_email" class="input mono" placeholder="例如 yourname@163.com" />
            </label>

            <label class="field span-2">
              <span class="field-label">
                <span>邮箱密码 / 客户端授权码 *</span>
                <span v-if="smtp.has_password" class="tag ok"><i class="dot" />已加密保存密码</span>
              </span>
              <div class="input-action-wrap">
                <input
                  v-model="smtp.password"
                  :type="showSmtpPwd ? 'text' : 'password'"
                  class="input"
                  :placeholder="smtp.has_password ? '留空表示沿用当前已保存密码' : '输入邮箱客户端专用授权码或密码'"
                />
                <button
                  type="button"
                  class="input-action-btn"
                  :aria-label="showSmtpPwd ? '隐藏密码' : '显示密码'"
                  @click="showSmtpPwd = !showSmtpPwd"
                >
                  <Icon :name="showSmtpPwd ? 'eye-off' : 'eye'" :size="15" />
                </button>
              </div>
              <span class="hint">网易 163、QQ 邮箱等需填写网页端生成的「客户端专用授权码」，而非登录密码。</span>
            </label>
          </div>
        </div>
      </section>

      <!-- 手机号短信通道 & 新用户注册方式 -->
      <section class="card form-card" style="margin-top: 16px">
        <div class="card-head">
          <div>
            <h2>手机号短信通道（阿里云 PNVS）</h2>
            <p class="hint">
              {{
                pnvsConfigured
                  ? '已检测到阿里云号码认证服务（PNVS）配置，所有手机号验证码优先走此通道，可与上方 SMTP 邮件并行使用。'
                  : '当前未配置阿里云 PNVS 环境变量，手机号验证码将回退到上方主发信通道。'
              }}
            </p>
          </div>
          <span class="tag" :class="pnvsConfigured ? 'ok' : ''">
            <i class="dot" />{{ pnvsConfigured ? 'PNVS 已接入' : '未配置 PNVS' }}
          </span>
        </div>

        <div class="divider" />

        <div class="card-head" style="margin-bottom: 12px">
          <div>
            <h2>新用户注册方式</h2>
            <p class="hint">
              控制新用户允许使用的注册方式及已有账号新绑定方式（已注册用户的密码登录与重置密码不受影响）。至少保留一种，且须已配置对应发信通道。
            </p>
          </div>
          <button type="button" class="btn primary sm" :disabled="savingSignup" @click="saveSignup">
            {{ savingSignup ? '保存中…' : '保存注册方式' }}
          </button>
        </div>

        <div v-if="signupMessage" class="banner" :class="signupMessage.type" style="margin-bottom: 12px">
          <Icon :name="signupMessage.type === 'danger' ? 'alert' : 'check'" :size="15" class="banner-ico" />
          <span>{{ signupMessage.text }}</span>
        </div>

        <div class="toggle-list">
          <div class="toggle-row" @click="signup.email = !signup.email">
            <div class="toggle-ico">
              <Icon name="mail" :size="16" />
            </div>
            <div class="toggle-txt">
              <strong>邮箱验证码注册</strong>
              <span>开放后，桌面、官网、社区与手机登录页展示邮箱注册与绑定入口</span>
            </div>
            <button
              type="button"
              role="switch"
              :aria-checked="signup.email"
              class="switch"
              :class="{ on: signup.email }"
              @click.stop="signup.email = !signup.email"
            />
          </div>

          <div class="toggle-row" @click="signup.phone = !signup.phone">
            <div class="toggle-ico">
              <Icon name="phone" :size="16" />
            </div>
            <div class="toggle-txt">
              <strong>手机号验证码注册</strong>
              <span>仅支持中国大陆手机号，开启前需配置阿里云 PNVS 或腾讯云短信通道</span>
            </div>
            <button
              type="button"
              role="switch"
              :aria-checked="signup.phone"
              class="switch"
              :class="{ on: signup.phone }"
              @click.stop="signup.phone = !signup.phone"
            />
          </div>
        </div>
      </section>
    </template>

    <!-- 测试发信弹窗 -->
    <template v-if="showTestModal">
      <div class="scrim" @click="!testing && (showTestModal = false)" />
      <form class="dialog" role="dialog" aria-modal="true" aria-label="发送测试邮件" @submit.prevent="sendTest">
        <div class="dialog-head">
          <div>
            <h3>发送测试验证码邮件</h3>
            <p>将使用当前表单中的 SMTP 参数（无需先保存）向指定邮箱发送测试邮件。</p>
          </div>
          <button
            type="button"
            class="btn ghost icon sm"
            :disabled="testing"
            aria-label="关闭"
            @click="showTestModal = false"
          >
            <Icon name="x" :size="16" />
          </button>
        </div>

        <div class="dialog-body">
          <label class="field">
            <span class="field-label">接收测试邮件的邮箱地址</span>
            <input
              v-model="testRecipient"
              class="input"
              type="email"
              placeholder="例如 your-personal@example.com"
              required
              autofocus
            />
          </label>
          <div v-if="testError" class="banner danger" role="alert">
            <Icon name="alert" :size="15" class="banner-ico" />
            <span>{{ testError }}</span>
          </div>
        </div>

        <div class="dialog-foot">
          <button type="button" class="btn" :disabled="testing" @click="showTestModal = false">取消</button>
          <button type="submit" class="btn primary" :disabled="testing">
            <Icon name="send" :size="14" />
            <span>{{ testing ? '正在发送…' : '立即发送测试' }}</span>
          </button>
        </div>
      </form>
    </template>
  </div>
</template>

<style scoped>
.sender-wrap {
  max-width: 900px;
}

.form-card {
  padding: 22px 24px;
}

.card-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 14px;
  flex-wrap: wrap;
}
.card-head h2 {
  font-size: 15.5px;
  font-weight: 680;
}
.card-head .hint {
  margin-top: 4px;
  max-width: 64ch;
}
.card-head code {
  font-family: var(--mono);
  font-size: 11.5px;
  padding: 1px 5px;
  border-radius: 4px;
  background: var(--canvas-soft);
  color: var(--ink);
}

.provider-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  margin-top: 16px;
}
.provider-card {
  padding: 14px 15px;
  border-radius: var(--r-md);
  border: 1.5px solid var(--line-strong);
  background: var(--surface);
  text-align: left;
  display: flex;
  flex-direction: column;
  gap: 8px;
  transition: border-color 0.14s, background 0.14s, box-shadow 0.14s;
}
.provider-card:hover {
  background: var(--surface-hover);
  border-color: var(--line-focus);
}
.provider-card.selected {
  border-color: var(--ink);
  background: var(--surface-muted);
  box-shadow: var(--shadow-xs);
}
.pc-top {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.pc-top strong {
  font-size: 13.5px;
  font-weight: 650;
}
.pc-radio {
  width: 15px;
  height: 15px;
  border-radius: 50%;
  border: 1.5px solid var(--line-strong);
  position: relative;
  flex: none;
}
.provider-card.selected .pc-radio {
  border-color: var(--ink);
}
.provider-card.selected .pc-radio::after {
  content: '';
  position: absolute;
  inset: 3px;
  border-radius: 50%;
  background: var(--ink);
}
.provider-card p {
  font-size: 12px;
  color: var(--ink-2);
  line-height: 1.5;
}

.smtp-box {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid var(--line);
}
.smtp-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}
.smtp-head h3 {
  font-size: 14px;
  font-weight: 680;
}
.preset-bar {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.preset-chip {
  height: 26px;
  padding: 0 9px;
  border-radius: 6px;
  border: 1px solid var(--line-strong);
  background: var(--surface);
  font-size: 12px;
  font-weight: 550;
  color: var(--ink-2);
  transition: border-color 0.14s, color 0.14s, background 0.14s;
}
.preset-chip:hover {
  border-color: var(--ink);
  color: var(--ink);
  background: var(--surface-hover);
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 15px 18px;
}
.span-2 {
  grid-column: span 2;
}
.full-seg {
  display: grid;
  grid-template-columns: 1fr 1fr;
  width: 100%;
}
.full-seg button {
  justify-content: center;
}

.divider {
  height: 1px;
  background: var(--line);
  margin: 20px 0;
}

.toggle-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.toggle-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 13px 15px;
  border-radius: var(--r-md);
  border: 1px solid var(--line);
  background: var(--surface-muted);
  cursor: pointer;
  transition: border-color 0.14s, background 0.14s;
}
.toggle-row:hover {
  border-color: var(--line-strong);
  background: var(--surface-hover);
}
.toggle-ico {
  width: 34px;
  height: 34px;
  border-radius: 9px;
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--ink-2);
  display: grid;
  place-items: center;
  flex: none;
}
.toggle-txt {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.toggle-txt strong {
  font-size: 13.5px;
  font-weight: 630;
}
.toggle-txt span {
  font-size: 12px;
  color: var(--ink-3);
}

@media (max-width: 780px) {
  .provider-grid {
    grid-template-columns: 1fr;
  }
  .form-grid {
    grid-template-columns: 1fr;
  }
  .span-2 {
    grid-column: span 1;
  }
}
</style>
