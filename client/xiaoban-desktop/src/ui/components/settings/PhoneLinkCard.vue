<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { NAlert, NSwitch } from 'naive-ui';
import { desktop, invoke, listen } from '../../../bridge.ts';

export interface LinkedPeer {
  id: string;
  name: string;
  online: boolean;
}

export interface PhoneLinkStatus {
  enabled: boolean;
  connected: boolean;
  deviceId: string | null;
  deviceName: string;
  relayUrl: string;
  loggedIn: boolean;
  peers: LinkedPeer[];
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
  lastError: null,
});
const loading = ref(true);
const busy = ref(false);
const error = ref('');

let unlisten: (() => void) | null = null;

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

async function run(command: string, args?: Record<string, unknown>) {
  if (!desktop || busy.value) return;
  busy.value = true;
  error.value = '';
  try {
    status.value = await invoke<PhoneLinkStatus>(command, args);
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

const toggleEnabled = (next: boolean) => run('phone_link_set_enabled', { enabled: next });
const removePeer = (peerId: string) => run('phone_link_remove_peer', { peerId });

onMounted(async () => {
  await refresh();
  if (desktop) {
    unlisten = await listen<PhoneLinkStatus>('phone-link-changed', next => {
      status.value = next;
    });
  }
});

onUnmounted(() => {
  unlisten?.();
});
</script>

<template>
  <div class="phone-link-card" data-testid="phone-link-card">
    <div class="xb-group-title">手机遥控</div>
    <div class="xb-group">
      <div class="xb-row">
        <div class="xb-row-text">
          <b>允许手机遥控这台电脑</b>
          <span>
            开启后，用同一个小伴账号登录的手机小伴能看到这台电脑，查看会话、发消息和处理确认；手机发来的指令按最严格权限逐项确认。
            {{ status.enabled ? (status.connected ? '已连上，手机上登录同一账号即可看到这台电脑。' : '正在连接…') : '' }}
          </span>
        </div>
        <NSwitch
          :value="status.enabled"
          aria-label="允许手机遥控"
          data-testid="phone-link-toggle"
          :disabled="!desktop || loading || busy"
          @update:value="toggleEnabled"
        />
      </div>

      <div v-for="peer in status.peers" :key="peer.id" class="xb-row" data-testid="phone-link-peer">
        <div class="xb-row-text">
          <b>
            {{ peer.name }}
            <em class="peer-status" :class="{ online: peer.online }">{{ peer.online ? '在线' : '离线' }}</em>
          </b>
        </div>
        <button
          type="button"
          class="xb-btn is-ghost is-sm peer-remove"
          data-testid="phone-link-remove"
          :disabled="busy"
          @click="removePeer(peer.id)"
        >
          移除
        </button>
      </div>

      <div v-if="!status.peers.length" class="xb-row" data-testid="phone-link-empty">
        <div class="xb-row-text">
          <span>{{ status.enabled ? '还没有手机登录这个账号。在手机小伴里登录同一个账号后会自动出现在这里。' : '开启后，同账号下的手机会出现在这里。' }}</span>
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
.peer-status{display:inline-block;margin-left:8px;padding:1px 7px;border-radius:999px;font-style:normal;font-size:11px;font-weight:500;background:var(--ui-surface-sunken);color:var(--ui-text-tertiary)}
.peer-status.online{background:var(--ui-soft);color:var(--ui-btn)}
.peer-remove{color:var(--ui-danger)}
</style>
