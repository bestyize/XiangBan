<script setup lang="ts">
import { ref } from 'vue';
import { NButton } from 'naive-ui';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
const props = defineProps<{ store: ControlStore; kind: 'wallpaper' | 'effect' | 'widget' | 'game'; id?: string; disabled?: boolean }>();
const emit = defineEmits<{ imported: [result: { kind: string; id: string; name: string }] }>();
const busy = ref(false);
async function run(share = false) {
  if (busy.value || props.disabled || !props.store.desktop) return;
  busy.value = true;
  try {
    if (props.id) {
      const saved = await invoke<boolean>('export_resource_pack', { kind: props.kind, id: props.id });
      if (!saved) return;
      props.store.notify(share ? '资源包已导出，请在论坛发帖，添加预览图和资源包附件。' : '资源包已导出，其他用户可在对应页面导入使用。', 'success');
      if (share) await invoke('open_download_url', { url: 'https://chat.xiangban.xyz' });
    } else {
      const result = await invoke<{ kind: string; id: string; name: string } | null>('import_resource_pack', { kind: props.kind });
      if (!result) return;
      emit('imported', result);
      props.store.notify(`已导入「${result.name}」，可预览后再使用。`, 'success');
    }
  } catch (error) { props.store.notify(error, 'error'); }
  finally { busy.value = false; }
}
</script>
<template>
  <div v-if="store.desktop" class="resource-share-actions" @click.stop @keydown.stop>
    <NButton size="small" secondary :loading="busy" :disabled="busy || disabled" :data-testid="id ? 'export-resource' : 'import-resource'" @click="run()">{{ id ? '导出资源包' : '导入资源包' }}</NButton>
    <NButton v-if="id" size="small" quaternary :disabled="busy || disabled" data-testid="share-resource" @click="run(true)">分享到论坛</NButton>
  </div>
</template>
<style scoped>
.resource-share-actions{display:flex;gap:6px;align-items:center;flex-wrap:wrap}
</style>
