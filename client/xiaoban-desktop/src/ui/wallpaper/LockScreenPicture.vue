<script setup lang="ts">
defineProps<{ frame: string; busy: boolean; manual: boolean; desktop?: boolean }>();
const emit = defineEmits<{ apply: []; save: []; close: [] }>();
</script>

<template>
  <section v-if="frame" class="lock-screen-preview" :aria-label="desktop ? '系统桌面底图预览' : '锁屏图片预览'">
    <img :src="frame" :alt="desktop ? '系统桌面底图预览' : '锁屏图片预览'" />
    <p v-if="manual" role="status">系统未能直接设置锁屏图片。</p>
    <div class="lock-screen-actions">
      <button type="button" class="xb-btn" :disabled="busy" @click="emit('close')">取消</button>
      <button v-if="manual" type="button" class="xb-btn-primary" :disabled="busy" data-testid="lock-screen-manual" @click="emit('save')">保存图片并打开锁屏设置</button>
      <button v-else type="button" class="xb-btn-primary" :disabled="busy" data-testid="lock-screen-confirm" @click="emit('apply')">{{ desktop ? '应用系统桌面底图' : '应用锁屏图片' }}</button>
    </div>
  </section>
</template>

<style scoped>
.lock-screen-preview { display: grid; gap: 8px; width: 100%; }
.lock-screen-preview img { display: block; width: 100%; max-height: 180px; object-fit: contain; border-radius: 8px; background: var(--ui-soft); }
.lock-screen-preview p { margin: 0; color: var(--ui-text-secondary); }
.lock-screen-actions { display: flex; flex-wrap: wrap; gap: 8px; justify-content: flex-end; }
</style>
