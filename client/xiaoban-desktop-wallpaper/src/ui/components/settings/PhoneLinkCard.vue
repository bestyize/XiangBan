<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { NAlert, NSwitch } from 'naive-ui';
import { desktop, invoke, listen } from '../../../bridge.ts';
import qrcode from '../../vendor/qrcode.js';

export interface LinkedPeer {
  id: string;
  name: string;
  online: boolean;
}

export interface PairingView {
  qrText: string;
  code: string;
  expiresAtMs: number;
}

export interface PhoneLinkStatus {
  enabled: boolean;
  connected: boolean;
  deviceId: string | null;
  deviceName: string;
  relayUrl: string;
  loggedIn: boolean;
  peers: LinkedPeer[];
  pairing: PairingView | null;
  lastError: string | null;
}

const status = ref<PhoneLinkStatus>({
  enabled: false,
  connected: false,
  deviceId: null,
  deviceName: '小伴电脑',
  relayUrl: 'https://link.xiangban.xyz/api/link',
  loggedIn: false,
  peers: [],
  pairing: null,
  lastError: null,
});
const loading = ref(true);
const busy = ref(false);
const error = ref('');
const copied = ref(false);
const nowMs = ref(Date.now());

let unlisten: (() => void) | null = null;
let timer: ReturnType<typeof setInterval> | null = null;

const remainingSeconds = computed(() => {
  const exp = status.value.pairing?.expiresAtMs ?? 0;
  return Math.max(0, Math.ceil((exp - nowMs.value) / 1000));
});

const activePairing = computed(() => {
  if (!status.value.pairing || remainingSeconds.value <= 0) return null;
  return status.value.pairing;
});

const qrSvg = computed(() => {
  const text = activePairing.value?.qrText;
  if (!text) return '';
  try {
    const qr = qrcode(0, 'L');
    qr.addData(text, 'Byte');
    qr.make();
    return qr.createSvgTag({ cellSize: 4, margin: 2, scalable: true });
  } catch {
    return '';
  }
});

function formatRemaining(sec: number): string {
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return `${m}:${String(s).padStart(2, '0')}`;
}

async function refresh() {
  if (!desktop) {
    loading.value = false;
    return;
  }
  try {
    status.value = await invoke<PhoneLinkStatus>('phone_link_status');
    error.value = '';
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function toggleEnabled(next: boolean) {
  if (!desktop || busy.value) return;
  busy.value = true;
  error.value = '';
  try {
    status.value = await invoke<PhoneLinkStatus>('phone_link_set_enabled', { enabled: next });
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function beginPairing() {
  if (!desktop || busy.value) return;
  busy.value = true;
  error.value = '';
  copied.value = false;
  try {
    status.value = await invoke<PhoneLinkStatus>('phone_link_begin_pairing');
    nowMs.value = Date.now();
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function cancelPairing() {
  if (!desktop || busy.value) return;
  busy.value = true;
  error.value = '';
  try {
    status.value = await invoke<PhoneLinkStatus>('phone_link_cancel_pairing');
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function unpair(peerId: string) {
  if (!desktop || busy.value) return;
  busy.value = true;
  error.value = '';
  try {
    status.value = await invoke<PhoneLinkStatus>('phone_link_unpair', { peerId });
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function copyQrText() {
  const text = activePairing.value?.qrText;
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    copied.value = true;
  } catch {
    copied.value = false;
  }
}

onMounted(async () => {
  await refresh();
  if (desktop) {
    unlisten = await listen<PhoneLinkStatus>('phone-link-changed', next => {
      status.value = next;
      nowMs.value = Date.now();
    });
  }
  timer = setInterval(() => {
    nowMs.value = Date.now();
  }, 1000);
});

onUnmounted(() => {
  unlisten?.();
  if (timer) clearInterval(timer);
});
</script>

<template>
  <div class="phone-link-card" data-testid="phone-link-card">
    <div class="xb-group-title">手机遥控与互联</div>
    <div class="xb-group">
      <div class="xb-row">
        <div class="xb-row-text">
          <b>允许手机遥控</b>
          <span>开启后，同一账号下的手机小伴可经端到端加密通道查看会话、发送消息与处理确认；远程指令默认按最严格权限逐项确认</span>
        </div>
        <NSwitch
          :value="status.enabled"
          aria-label="允许手机遥控"
          data-testid="phone-link-toggle"
          :disabled="!desktop || loading || busy"
          @update:value="toggleEnabled"
        />
      </div>

      <div class="xb-row">
        <div class="xb-row-text">
          <b>连接新手机</b>
          <span>
            {{ status.enabled ? (status.connected ? '已连接中转服务 · 端到端加密就绪' : '正在连接中转服务…') : '首次点击配对将自动开启手机遥控' }}
          </span>
        </div>
        <button
          v-if="!activePairing"
          type="button"
          class="xb-btn is-sm"
          data-testid="phone-link-pair"
          :disabled="!desktop || loading || busy"
          @click="beginPairing"
        >
          {{ busy ? '准备中…' : '连接手机' }}
        </button>
        <button
          v-else
          type="button"
          class="xb-btn is-ghost is-sm"
          data-testid="phone-link-cancel"
          :disabled="busy"
          @click="cancelPairing"
        >
          关闭二维码
        </button>
      </div>

      <div v-if="activePairing" class="xb-row qr-row" data-testid="phone-link-qr">
        <div class="qr-box" v-html="qrSvg" />
        <div class="qr-meta">
          <b>请用手机小伴扫码配对</b>
          <span>在手机端登录同一小伴账号后扫码，中转服务看不到你的对话内容。</span>
          <span class="qr-timer" data-testid="phone-link-countdown">二维码有效剩余 {{ formatRemaining(remainingSeconds) }}</span>
          <div class="qr-actions">
            <button type="button" class="xb-btn is-ghost is-sm" data-testid="phone-link-copy" @click="copyQrText">
              {{ copied ? '已复制配对串' : '复制配对串' }}
            </button>
          </div>
        </div>
      </div>

      <div v-for="peer in status.peers" :key="peer.id" class="xb-row" data-testid="phone-link-peer">
        <div class="xb-row-text">
          <b>
            {{ peer.name }}
            <em class="peer-status" :class="{ online: peer.online }">{{ peer.online ? '在线' : '离线' }}</em>
          </b>
          <span>设备编号：{{ peer.id }}</span>
        </div>
        <button
          type="button"
          class="xb-btn is-ghost is-sm peer-remove"
          data-testid="phone-link-unpair"
          :disabled="busy"
          @click="unpair(peer.id)"
        >
          移除设备
        </button>
      </div>

      <div v-if="!status.peers.length && !activePairing" class="xb-row">
        <div class="xb-row-text">
          <span>暂无已配对的手机设备</span>
        </div>
      </div>
    </div>

    <NAlert v-if="error || status.lastError" type="error" :show-icon="false" style="margin-top:10px;margin-bottom:14px">
      {{ error || status.lastError }}
    </NAlert>
  </div>
</template>

<style scoped>
.phone-link-card{margin-bottom:18px}
.qr-row{align-items:center;gap:16px;padding-top:12px;padding-bottom:12px}
.qr-box{width:148px;height:148px;flex-shrink:0;padding:8px;border-radius:10px;background:#fff;box-shadow:0 0 0 1px var(--ui-line);display:grid;place-items:center}
.qr-box :deep(svg){width:100%;height:100%;display:block}
.qr-meta{display:flex;flex-direction:column;gap:6px;min-width:0;flex:1}
.qr-meta b{font-size:13.5px;font-weight:600}
.qr-meta span{font-size:12px;color:var(--ui-text-secondary);line-height:1.5}
.qr-timer{color:var(--ui-btn)!important;font-weight:500}
.qr-actions{display:flex;gap:8px;margin-top:4px}
.peer-status{display:inline-block;margin-left:8px;padding:1px 7px;border-radius:999px;font-style:normal;font-size:11px;font-weight:500;background:var(--ui-surface-sunken);color:var(--ui-text-tertiary)}
.peer-status.online{background:var(--ui-soft);color:var(--ui-btn)}
.peer-remove{color:var(--ui-danger)}
</style>
