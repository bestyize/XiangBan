<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import { api, errorMessage } from '../api';
import type { SmtpConfig } from '../types';

const loading = ref(true);
const saving = ref(false);
const testing = ref(false);
const message = ref<{ type: 'ok' | 'error'; text: string } | null>(null);

const provider = ref<'console' | 'smtp' | 'tencent'>('console');
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

const testRecipient = ref('');
const showTestModal = ref(false);

const presets = [
  { label: '网易 163 邮箱', host: 'smtp.163.com', port: 465, secure: true },
  { label: 'QQ 邮箱', host: 'smtp.qq.com', port: 465, secure: true },
  { label: '腾讯企业邮', host: 'smtp.exmail.qq.com', port: 465, secure: true },
  { label: 'Gmail', host: 'smtp.gmail.com', port: 587, secure: false },
  { label: 'Resend', host: 'smtp.resend.com', port: 465, secure: true },
];

function applyPreset(p: (typeof presets)[number]) {
  smtp.host = p.host;
  smtp.port = p.port;
  smtp.secure = p.secure;
  message.value = { type: 'ok', text: `已套用【${p.label}】服务器配置，请填入账号和授权码` };
}

async function load() {
  loading.value = true;
  message.value = null;
  try {
    const data = await api.getSender();
    provider.value = data.provider;
    if (data.smtp) {
      smtp.host = data.smtp.host || '';
      smtp.port = data.smtp.port || 465;
      smtp.secure = data.smtp.secure ?? (smtp.port === 465);
      smtp.username = data.smtp.username || '';
      smtp.has_password = data.smtp.has_password ?? false;
      smtp.from_name = data.smtp.from_name || '相伴';
      smtp.from_email = data.smtp.from_email || '';
      smtp.password = '';
    }
  } catch (err) {
    message.value = { type: 'error', text: errorMessage(err) };
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
    message.value = { type: 'ok', text: '发信设置保存成功，已实时生效' };
    if (smtp.password) {
      smtp.has_password = true;
      smtp.password = '';
    }
  } catch (err) {
    message.value = { type: 'error', text: errorMessage(err) };
  } finally {
    saving.value = false;
  }
}

async function sendTest() {
  if (!testRecipient.value.trim() || !testRecipient.value.includes('@')) {
    alert('请输入有效的测试收件人邮箱');
    return;
  }
  testing.value = true;
  try {
    const res = await api.testSender({
      recipient: testRecipient.value.trim(),
      smtp: provider.value === 'smtp' ? smtp : undefined,
    });
    alert(res.message || '测试邮件已发送，请查收');
    showTestModal.value = false;
  } catch (err) {
    alert('发信失败：' + errorMessage(err));
  } finally {
    testing.value = false;
  }
}

onMounted(load);
</script>

<template>
  <div class="sender-page">
    <div class="page-head">
      <div>
        <h2>发信通道设置</h2>
        <div class="muted">配置注册与登录验证码的发送渠道（支持控制台日志、SMTP 邮件服务与腾讯云）</div>
      </div>
      <div class="head-actions">
        <button v-if="provider === 'smtp'" type="button" class="btn" @click="showTestModal = true">
          测试发信
        </button>
        <button type="button" class="btn primary" :disabled="saving || loading" @click="save">
          {{ saving ? '保存中…' : '保存设置' }}
        </button>
      </div>
    </div>

    <div v-if="message" class="banner" :class="message.type">
      {{ message.text }}
    </div>

    <div v-if="loading" class="empty">加载中…</div>

    <div v-else class="card form-card">
      <div class="section-title">选择发信通道</div>
      <div class="provider-options">
        <label class="provider-option" :class="{ selected: provider === 'console' }">
          <input v-model="provider" type="radio" value="console" />
          <div class="option-content">
            <div class="option-title">控制台输出（本地 / 开发测试）</div>
            <div class="muted">验证码只打印在服务器终端或日志中，不通过真实网络发信。</div>
          </div>
        </label>

        <label class="provider-option" :class="{ selected: provider === 'smtp' }">
          <input v-model="provider" type="radio" value="smtp" />
          <div class="option-content">
            <div class="option-title">SMTP 邮件服务（推荐）</div>
            <div class="muted">使用任意标准 SMTP 邮箱（163、QQ、Gmail、企业邮等）直接发送验证码邮件。</div>
          </div>
        </label>

        <label class="provider-option" :class="{ selected: provider === 'tencent' }">
          <input v-model="provider" type="radio" value="tencent" />
          <div class="option-content">
            <div class="option-title">腾讯云服务（SMS / SES）</div>
            <div class="muted">需在服务器环境变量配置 `TENCENT_SECRET_ID` 及短信/模板参数。</div>
          </div>
        </label>
      </div>

      <!-- SMTP 配置区域 -->
      <div v-if="provider === 'smtp'" class="smtp-section">
        <div class="section-divider"></div>
        <div class="section-head">
          <div class="section-title">SMTP 服务配置</div>
          <div class="presets">
            <span class="muted" style="font-size: 13px;">快速预设：</span>
            <button
              v-for="p in presets"
              :key="p.label"
              type="button"
              class="btn small"
              @click="applyPreset(p)"
            >
              {{ p.label }}
            </button>
          </div>
        </div>

        <div class="form-grid">
          <label class="field">
            <span>SMTP 服务器地址 *</span>
            <input v-model="smtp.host" class="input" placeholder="例如 smtp.163.com 或 smtp.qq.com" />
          </label>

          <label class="field">
            <span>端口 *</span>
            <input v-model.number="smtp.port" type="number" class="input" placeholder="465 或 587" />
          </label>

          <label class="field">
            <span>加密连接 (SSL/TLS)</span>
            <select v-model="smtp.secure" class="input">
              <option :value="true">SSL / TLS (推荐端口 465)</option>
              <option :value="false">STARTTLS / 明文 (常用端口 587 / 25)</option>
            </select>
          </label>

          <label class="field">
            <span>发信人显示名称</span>
            <input v-model="smtp.from_name" class="input" placeholder="例如 小伴壁纸 或 相伴" />
          </label>

          <label class="field">
            <span>发信账号 (Username) *</span>
            <input v-model="smtp.username" class="input" placeholder="例如 yourname@163.com" />
          </label>

          <label class="field">
            <span>发信人邮箱 (From Email) *</span>
            <input v-model="smtp.from_email" class="input" placeholder="例如 yourname@163.com" />
          </label>

          <label class="field full-width">
            <span>
              邮箱密码 / 客户端授权码 *
              <span v-if="smtp.has_password" class="badge ok" style="margin-left: 6px;">已保存密码</span>
            </span>
            <input
              v-model="smtp.password"
              type="password"
              class="input"
              :placeholder="smtp.has_password ? '留空表示保持当前已配置密码不变' : '输入发信邮箱的授权码或密码'"
            />
            <span class="muted" style="font-size: 12px; margin-top: 4px;">
              注：网易邮箱、QQ 邮箱等通常需要填在网页端生成的“专用授权码”，而非邮箱登录密码。
            </span>
          </label>
        </div>
      </div>
    </div>

    <!-- 测试发信弹窗 -->
    <div v-if="showTestModal" class="modal-mask">
      <div class="card modal-box">
        <div class="modal-title">发送测试邮件</div>
        <p class="muted" style="margin: 0 0 16px;">
          系统将使用当前表单填写的 SMTP 配置向指定邮箱发送一封包含测试验证码的邮件。
        </p>
        <label class="field">
          <span>接收测试邮件的邮箱地址：</span>
          <input
            v-model="testRecipient"
            class="input"
            type="email"
            placeholder="例如 your-personal@example.com"
            @keyup.enter="sendTest"
          />
        </label>
        <div class="modal-actions">
          <button type="button" class="btn" :disabled="testing" @click="showTestModal = false">取消</button>
          <button type="button" class="btn primary" :disabled="testing" @click="sendTest">
            {{ testing ? '发送中…' : '立刻发送' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.sender-page { max-width: 860px; }
.head-actions { display: flex; gap: 8px; }
.banner { padding: 10px 14px; border-radius: var(--radius); margin-bottom: 16px; font-size: 13.5px; }
.banner.ok { background: var(--ok-bg); color: var(--ok); }
.banner.error { background: var(--danger-bg); color: var(--danger); }
.form-card { padding: 24px 28px; }
.section-title { font-weight: 600; font-size: 15px; margin-bottom: 12px; }
.provider-options { display: flex; flex-direction: column; gap: 10px; }
.provider-option {
  display: flex; align-items: flex-start; gap: 12px; padding: 12px 16px;
  border: 1px solid var(--border); border-radius: 8px; cursor: pointer;
  transition: all 0.15s;
}
.provider-option:hover { background: #fafbff; }
.provider-option.selected { border-color: var(--primary); background: #f5f4fe; }
.provider-option input { margin-top: 3px; }
.option-title { font-weight: 500; margin-bottom: 2px; }
.section-divider { height: 1px; background: var(--border); margin: 24px 0; }
.section-head { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 10px; margin-bottom: 16px; }
.presets { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
.form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px 20px; }
.full-width { grid-column: span 2; }

.modal-mask {
  position: fixed; inset: 0; background: rgba(0, 0, 0, 0.45);
  display: flex; align-items: center; justify-content: center; z-index: 1000;
}
.modal-box { padding: 24px; width: 100%; max-width: 440px; box-shadow: 0 10px 25px rgba(0,0,0,0.15); }
.modal-title { font-size: 17px; font-weight: 600; margin-bottom: 10px; }
.modal-actions { display: flex; justify-content: flex-end; gap: 10px; margin-top: 20px; }

@media (max-width: 640px) {
  .form-grid { grid-template-columns: 1fr; }
  .full-width { grid-column: span 1; }
}
</style>
