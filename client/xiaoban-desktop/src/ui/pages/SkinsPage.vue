<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { Download, Sparkles } from 'lucide-vue-next';
import type { ControlStore } from '../control-types.ts';
import StudioActions from '../components/StudioActions.vue';
import { activeSkinId, applySkin, isBuiltinSkin, deleteSkin, exportSkin, importSkin, refreshSkins, skins } from '../skins/store.ts';

const props = defineProps<{ store: ControlStore }>();
const busy = ref(false);
const removing = ref<string | null>(null);
async function run(action: () => Promise<void>) {
  if (busy.value) return;
  busy.value = true;
  try { await action(); } catch (error) { props.store.notify(error, 'error'); } finally { busy.value = false; }
}
onMounted(() => { if (props.store.desktop) void run(refreshSkins); });
function install() { void run(async () => { const item = await importSkin(); if (item) props.store.notify('皮肤已导入'); }); }
function apply(id: string | null) { void run(async () => { await applySkin(id); props.store.notify(id ? '已应用皮肤' : '已恢复默认外观'); }); }
function remove(id: string) { void run(async () => { await deleteSkin(id); removing.value = null; props.store.notify('皮肤已删除'); }); }
function share(id: string) { void run(async () => { if (await exportSkin(id)) props.store.notify('皮肤已导出'); }); }
</script>
<template>
  <div class="skins-page" data-testid="skins-page">
    <StudioActions><button class="xb-btn" type="button" data-testid="skin-import" :disabled="busy || !store.desktop" @click="install"><Download :size="14" />导入</button><button class="xb-btn-primary" type="button" data-testid="skin-create" :disabled="!store.desktop" @click="store.startCreation('skin')"><Sparkles :size="14" />让小伴创作</button></StudioActions>
    <div class="skins-grid">
      <article class="skin-card" data-testid="skin-default">
        <span class="preview default-preview"><span></span></span>
        <div class="caption"><b>默认外观</b><small v-if="!activeSkinId">使用中</small></div>
        <button class="xb-btn" type="button" data-testid="skin-apply-default" :disabled="busy || !activeSkinId || !store.desktop" @click="apply(null)">{{ !activeSkinId ? '使用中' : '恢复默认' }}</button>
      </article>
      <article v-for="item in skins" :key="item.id" class="skin-card" :data-testid="`skin-${item.id}`">
        <span class="preview"><img :src="item.preview" alt="" /></span>
        <div class="caption"><b :title="item.name">{{ item.name }}</b><small v-if="activeSkinId === item.id">使用中</small></div>
        <div class="actions">
          <button class="xb-btn" type="button" :data-testid="`skin-apply-${item.id}`" :disabled="busy || activeSkinId === item.id || !store.desktop" @click="apply(item.id)">{{ activeSkinId === item.id ? '使用中' : '应用' }}</button>
          <template v-if="!isBuiltinSkin(item.id)"><button class="xb-btn" type="button" :disabled="busy" @click="share(item.id)">导出</button><button class="xb-btn" type="button" :disabled="busy" @click="removing = item.id">删除</button></template>
        </div>
        <div v-if="removing === item.id" class="confirm" role="alert"><span>删除此皮肤？使用中的皮肤将恢复默认。</span><button class="xb-btn" type="button" @click="removing = null">取消</button><button class="xb-btn is-danger" type="button" :disabled="busy" @click="remove(item.id)">删除</button></div>
      </article>
    </div>
  </div>
</template>
<style scoped>
.skins-page{min-width:0;overflow:auto;padding:24px 28px;color:var(--ui-text-primary)}
.skins-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(180px,1fr));gap:20px;align-content:start}
.skin-card{min-width:0;color:inherit}
.preview{display:block;aspect-ratio:16/10;overflow:hidden;border-radius:10px;background:var(--ui-control)}
.preview img{width:100%;height:100%;object-fit:cover;display:block}
.caption{display:flex;align-items:center;justify-content:space-between;gap:8px;padding:10px 2px;font-size:13px}
.caption b{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-weight:500}.caption small{flex:none;color:var(--ui-ink)}
.default-preview{padding:16px;background:var(--ui-tint)}.default-preview span{display:block;width:100%;height:100%;border-radius:6px;background:var(--ui-surface)}
.actions,.confirm{display:flex;gap:8px;align-items:center;flex-wrap:wrap}.confirm{margin-top:8px;font-size:12px;color:var(--ui-text-secondary)}
</style>
