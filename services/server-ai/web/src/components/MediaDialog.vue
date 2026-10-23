<script setup lang="ts">
import { useModal } from '../composables/useModal';
import { computed, ref } from 'vue';
import { api, errorMessage } from '../api';
import type { CapabilityInfo } from '../media';
import type { MediaSetting, ModelOption, Product } from '../types';

const props = defineProps<{ product: Product; capability: CapabilityInfo; setting: MediaSetting | null; models: ModelOption[] }>();
const emit = defineEmits<{ close: []; saved: [warnings: string[]] }>();

const root = ref<HTMLElement | null>(null);
useModal(root, () => emit('close'));

const model = ref(props.setting?.model ?? '');
const enabled = ref(props.setting?.enabled ?? true);
const requiresPaid = ref(props.setting?.requires_paid ?? false);
const voices = ref((props.setting?.voices ?? []).join('\n'));
const sizes = ref((props.setting?.sizes.length ? props.setting.sizes : ['1024x1024']).join('\n'));
const minutes = ref(Math.round((props.setting?.max_secs ?? 1800) / 60));
const busy = ref(false);
const error = ref('');

const picked = computed(() => props.models.find((m) => m.model_name === model.value) ?? null);
// 已保存的模型如果从网关消失了，仍要显示在下拉里，否则用户会误以为没选。
const options = computed(() => {
  const list = props.models.map((m) => m.model_name);
  return model.value && !list.includes(model.value) ? [model.value, ...list] : list;
});

/** 一行一个，也接受逗号分隔，方便从文档里整段粘贴。 */
function lines(text: string): string[] {
  return text.split(/[\n,，]/).map((s) => s.trim()).filter(Boolean);
}

async function submit() {
  busy.value = true;
  error.value = '';
  try {
    const result = await api.saveMedia(props.product.product_id, props.capability.id, {
      model: model.value,
      enabled: enabled.value,
      requires_paid: requiresPaid.value,
      voices: props.capability.id === 'speech' ? lines(voices.value) : [],
      sizes: props.capability.id === 'image' ? lines(sizes.value) : [],
      max_secs: Math.round(Number(minutes.value) * 60) || 1800,
    });
    emit('saved', result.warnings);
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="scrim" @click="emit('close')"></div>
  <form ref="root" class="dialog" role="dialog" aria-modal="true" aria-labelledby="media-title" @submit.prevent="submit">
    <div class="dialog-head">
      <h2 id="media-title">{{ setting ? '修改' : '开放' }}{{ capability.label }} · {{ product.name }}</h2>
    </div>
    <div class="dialog-body">
      <div v-if="error" class="notice error" role="alert">{{ error }}</div>
      <p class="hint">用于{{ capability.usedBy }}。客户端不传也看不到模型名，换模型立即生效，不用发版。</p>

      <fieldset class="back">
        <legend>幕后调用（用户看不到）</legend>
        <div class="field">
          <label for="m-model">实际模型</label>
          <select id="m-model" v-model="model" class="select mono" required>
            <option value="" disabled>选择 AI 网关里已启用的模型</option>
            <option v-for="name in options" :key="name" :value="name">{{ name }}</option>
          </select>
          <span v-if="picked" class="hint num">模型倍率 {{ picked.model_ratio }}，补全倍率 {{ picked.completion_ratio }}</span>
        </div>
        <div v-if="picked?.ratio_warning" class="notice warn">
          这个模型的倍率和 New API 未配置时的默认值相同，很可能还没设置真实定价。可以在 New API 的「同步价格」里选 LiteLLM 预设补上。
        </div>
        <div v-else-if="model && !picked" class="notice error">AI 网关里已经找不到这个模型，请换一个。</div>

        <div v-if="capability.id === 'speech'" class="field">
          <label for="m-voices">可选音色</label>
          <textarea id="m-voices" v-model="voices" class="textarea mono" rows="4" placeholder="alloy&#10;nova" required></textarea>
          <span class="hint">一行一个，第一个是默认音色。客户端只能从这里选。</span>
        </div>
        <div v-if="capability.id === 'image'" class="field">
          <label for="m-sizes">可选尺寸</label>
          <textarea id="m-sizes" v-model="sizes" class="textarea mono" rows="3" placeholder="1024x1024" required></textarea>
          <span class="hint">一行一个，如 1024x1024，或 auto；第一个是默认尺寸。</span>
        </div>
        <div v-if="capability.id === 'realtime'" class="field minutes">
          <label for="m-minutes">单次对话最长（分钟）</label>
          <input id="m-minutes" v-model.number="minutes" class="input num" type="number" min="1" max="240" step="1" required />
          <span class="hint">1–240 分钟，到时自动结束，用户可以重新开始。</span>
        </div>
        <p class="hint">注意：{{ capability.caution }}</p>
      </fieldset>

      <div class="rules">
        <label class="check"><input v-model="requiresPaid" type="checkbox" /> 需要先兑换额度才能使用（试用额度不能用）</label>
        <label class="check"><input v-model="enabled" type="checkbox" /> 启用</label>
      </div>
    </div>
    <div class="dialog-foot">
      <button type="button" class="btn" @click="emit('close')">取消</button>
      <button type="submit" class="btn primary" :disabled="busy || !model">{{ busy ? '保存中…' : '保存' }}</button>
    </div>
  </form>
</template>

<style scoped>
fieldset { border: 1px solid var(--line); border-radius: var(--r-md); margin: 0; padding: 4px 14px 14px; display: flex; flex-direction: column; gap: 12px; min-width: 0; }
legend { padding: 0 6px; font-weight: 600; font-size: 13px; }
.back { background: var(--curtain-light); }
.rules { display: flex; flex-direction: column; gap: 8px; }
.minutes { max-width: 220px; }
</style>
