<script setup lang="ts">
import { ref } from 'vue';
import { NButton } from 'naive-ui';
import { FileDown, Share2 } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
// `icons` drops the wording and keeps only the two marks, for the narrow strip at a card's lower right.
const props = defineProps<{ store: ControlStore; kind: 'wallpaper' | 'effect' | 'widget' | 'game'; id?: string; disabled?: boolean; importLabel?: string; icons?: boolean }>();
const emit = defineEmits<{ imported: [result: { kind: string; id: string; name: string }] }>();
const busy = ref(false);
// Where each kind of pack lands, for the note shown when a pack is picked on another kind's page.
const PLACES: Record<string, string> = { wallpaper: '壁纸页的“我的壁纸”', effect: '桌面特效页', widget: '桌面组件页', game: '游戏空间页' };
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
      const result = await invoke<{ kind: string; id: string; name: string } | null>('import_resource_pack');
      if (!result) return;
      if (result.kind === props.kind) {
        emit('imported', result);
        props.store.notify(`已导入「${result.name}」，可预览后再使用。`, 'success');
      } else {
        props.store.notify(`已导入「${result.name}」，在${PLACES[result.kind] ?? '对应页面'}查看。`, 'success');
      }
    }
  } catch (error) { props.store.notify(error, 'error'); }
  finally { busy.value = false; }
}
</script>
<template>
  <div v-if="store.desktop" class="resource-share-actions" :class="{ icons }" @click.stop @keydown.stop>
    <NButton :size="importLabel && !icons ? 'medium' : 'small'" :round="!!importLabel && !icons" :circle="!!icons" secondary :loading="busy" :disabled="busy || disabled" :title="id ? '导出资源包' : importLabel || '导入资源包'" :aria-label="id ? '导出资源包' : importLabel || '导入资源包'" :data-testid="id ? 'export-resource' : 'import-resource'" @click="run()">
      <template v-if="icons" #icon><FileDown :size="15" /></template>
      <template v-if="!icons">{{ id ? '导出资源包' : importLabel || '导入资源包' }}</template>
    </NButton>
    <NButton v-if="id" size="small" :circle="!!icons" :secondary="!!icons" :quaternary="!icons" :disabled="busy || disabled" :title="icons ? '分享到论坛' : undefined" :aria-label="icons ? '分享到论坛' : undefined" data-testid="share-resource" @click="run(true)">
      <template v-if="icons" #icon><Share2 :size="15" /></template>
      <template v-if="!icons">分享到论坛</template>
    </NButton>
  </div>
</template>
<style scoped>
.resource-share-actions{display:flex;gap:6px;align-items:center;flex-wrap:wrap}
.resource-share-actions.icons{gap:4px;flex-wrap:nowrap}
</style>
