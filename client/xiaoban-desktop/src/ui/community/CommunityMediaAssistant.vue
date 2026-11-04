<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '../../bridge.ts';

defineProps<{ recognized: boolean }>();

const input = ref('');
const link = ref('');
const reply = ref('');
const output = ref('');
const busy = ref(false);
const status = ref({ installed: false, verified: false, saved: false });
const confirmInstall = ref(false);

async function refresh() {
  status.value = await invoke('community_media_action', { action: 'status' });
}

async function run(task: () => Promise<void>) {
  if (busy.value) return;
  busy.value = true;
  try {
    if (!await invoke<boolean>('community_media_page')) {
      throw new Error('当前已离开 yt-dlp 项目页，请重新打开小伴。');
    }
    await task();
  } catch (error) {
    reply.value = error instanceof Error ? error.message : String(error);
  } finally {
    busy.value = false;
  }
}

function action(name: 'install' | 'download' | 'save') {
  return run(async () => {
    const result = await invoke<{ version?: string; path?: string }>('community_media_action', {
      action: name, ...(name === 'download' ? { url: link.value } : {}),
    });
    await refresh();
    if (name === 'install') {
      confirmInstall.value = false;
      reply.value = `yt-dlp 已安装${result.version ? `（${result.version}）` : ''}。把视频链接发给我试试。`;
    } else if (name === 'download') {
      output.value = result.path ?? '';
      link.value = '';
      reply.value = '下载完成。';
    } else {
      reply.value = '已保存到小伴。以后在普通聊天里说「帮我下载这个视频」，再附上新链接即可。';
    }
  });
}

function ask() {
  const question = input.value.trim();
  if (!question) return;
  return run(async () => {
    input.value = '';
    link.value = question.match(/https?:\/\/[^\s，。！？、）】<>]+/u)?.[0] ?? '';
    output.value = '';
    confirmInstall.value = false;
    await refresh();
    if (/安装|装一下|装上/u.test(question) || (link.value && !status.value.installed)) {
      confirmInstall.value = !status.value.installed;
      reply.value = confirmInstall.value
        ? '可以从 yt-dlp 的 GitHub 官方发布页安装，文件会放在小伴自己的工具目录。'
        : 'yt-dlp 已安装。把视频链接发给我试试。';
    } else if (link.value) {
      reply.value = '可以下载这个视频。文件会保存到系统下载目录下的「小伴」文件夹。';
    } else if (/下载|怎么用|使用/u.test(question)) {
      reply.value = '发一个视频链接给我。我会下载单个视频并告诉你保存位置。';
    } else {
      reply.value = 'yt-dlp 是一个命令行工具，可以把支持的网站上的视频下载到电脑。你可以问我「帮我安装」，也可以直接发视频链接。';
    }
  });
}
</script>

<template>
  <aside class="community-assistant" aria-label="小伴社区助手">
    <strong>小伴</strong>
    <template v-if="recognized">
      <small>当前页面 · yt-dlp</small>
      <form @submit.prevent="ask">
        <input v-model="input" aria-label="问小伴" placeholder="这是什么？" :disabled="busy" />
        <button class="xb-btn is-sm" type="submit" :disabled="busy || !input.trim()">发送</button>
      </form>
      <p v-if="reply" role="status">{{ reply }}</p>
      <p v-if="output" class="community-assistant-path">{{ output }}</p>
      <button v-if="confirmInstall" class="xb-btn is-sm" type="button" :disabled="busy" @click="action('install')">确认安装</button>
      <button v-if="link && status.installed" class="xb-btn is-sm" type="button" :disabled="busy" @click="action('download')">下载视频</button>
      <button v-if="status.verified && !status.saved" class="xb-btn is-sm" type="button" :disabled="busy" @click="action('save')">保存为我的工具</button>
      <small v-if="busy" role="status">正在处理…</small>
    </template>
    <p v-else>打开社区里的 yt-dlp 项目页，即可让小伴帮你安装和下载视频。</p>
  </aside>
</template>

<style scoped>
.community-assistant {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 10px;
  box-sizing: border-box;
  width: min(320px, 100%);
  max-height: min(280px, 40vh);
  min-width: 0;
  padding: 12px;
  overflow: auto;
  border: 1px solid var(--ui-line);
  border-radius: 12px;
  background: var(--ui-surface);
  box-shadow: var(--ui-shadow-panel);
}
.community-assistant form { display: flex; gap: 4px; width: 100%; }
.community-assistant input { flex: 1; min-width: 0; color: inherit; background: var(--ui-surface); }
.community-assistant p { margin: 0; overflow-wrap: anywhere; }
.community-assistant small, .community-assistant-path { color: var(--ui-text-secondary); }
</style>
