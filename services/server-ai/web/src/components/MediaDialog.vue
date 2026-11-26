<script setup lang="ts">
import { computed, ref } from 'vue';
import { api, errorMessage } from '../api';
import { useModal } from '../composables/useModal';
import type { CapabilityInfo } from '../media';
import type { MediaSetting, ModelOption, Product } from '../types';
import ChipEditor from './ChipEditor.vue';
import Icon from './Icon.vue';
import ModelPicker from './ModelPicker.vue';
import SwitchRow from './SwitchRow.vue';

const props = defineProps<{ product: Product; capability: CapabilityInfo; setting: MediaSetting | null; models: ModelOption[] }>();
const emit = defineEmits<{ close: []; saved: [warnings: string[]] }>();

const root = ref<HTMLElement | null>(null);
useModal(root, () => emit('close'));

const model = ref(props.setting?.model ?? '');
const enabled = ref(props.setting?.enabled ?? true);
const requiresPaid = ref(props.setting?.requires_paid ?? false);
const voices = ref<string[]>(props.setting?.voices?.length ? [...props.setting.voices] : ['nova', 'alloy']);
const sizes = ref<string[]>(props.setting?.sizes?.length ? [...props.setting.sizes] : ['1024x1024', 'auto']);
const minutes = ref(Math.round((props.setting?.max_secs ?? 1800) / 60));
const busy = ref(false);
const error = ref('');

const picked = computed(() => props.models.find((m) => m.model_name === model.value) ?? null);

const capEndpoints: Record<string, string> = {
  speech: '/v1/audio/speech',
  image: '/v1/images/generations',
  transcribe: '/v1/audio/transcriptions',
  realtime: '/v1/realtime (WS)',
};

const VOICE_PRESETS = ['nova', 'alloy', 'shimmer', 'echo', 'onyx', 'fable'];
const SIZE_PRESETS = ['1024x1024', '1536x1024', '1024x1536', '1792x1024', '1024x1792', 'auto'];
const REALTIME_PRESETS = [15, 30, 60, 120];

const canSubmit = computed(() => {
  if (!model.value) return false;
  if (props.capability.id === 'speech' && voices.value.length === 0) return false;
  if (props.capability.id === 'image' && sizes.value.length === 0) return false;
  return true;
});

async function submit() {
  if (!canSubmit.value) return;
  busy.value = true;
  error.value = '';
  try {
    const result = await api.saveMedia(props.product.product_id, props.capability.id, {
      model: model.value,
      enabled: enabled.value,
      requires_paid: requiresPaid.value,
      voices: props.capability.id === 'speech' ? voices.value : [],
      sizes: props.capability.id === 'image' ? sizes.value : [],
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
  <form ref="root" class="dialog wide" role="dialog" aria-modal="true" aria-labelledby="media-title" @submit.prevent="submit">
    <div class="dialog-head">
      <div class="head-lead">
        <span class="modal-head-ico"><Icon :name="capability.icon" :size="17" /></span>
        <div>
          <h2 id="media-title" class="title-row">
            <span>{{ setting ? '修改' : '开放' }}{{ capability.label }}</span>
            <span class="tag mono">{{ capEndpoints[capability.id] }}</span>
          </h2>
          <p>客户端只见能力与选项、不见真实模型名，保存后新请求立即生效。</p>
        </div>
      </div>
      <button type="button" class="btn icon sm ghost" aria-label="关闭" @click="emit('close')"><Icon name="x" :size="16" /></button>
    </div>

    <div class="dialog-body">
      <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="16" /><span>{{ error }}</span></div>

      <div class="field">
        <label>上游实际模型<span class="hint">已自动将匹配该能力的模型排在前面</span></label>
        <ModelPicker v-model="model" :models="models" :recommend="capability.id" placeholder="选择 AI 网关里已启用的模型…" />
      </div>

      <div v-if="picked?.ratio_warning" class="notice warn">
        <Icon name="alert" :size="15" />
        <span>该模型倍率仍为 New API 默认值，建议先在 New API「同步价格」补齐真实倍率再对外开放。</span>
      </div>
      <div v-else-if="model && !picked && models.length" class="notice error">
        <Icon name="circle-alert" :size="15" />
        <span>AI 网关里已经找不到这个模型，请换一个。</span>
      </div>

      <div v-if="capability.id === 'speech'" class="field">
        <label>可选音色<span class="hint">首个胶囊为默认音色，点击星标可切换默认</span></label>
        <ChipEditor v-model="voices" :presets="VOICE_PRESETS" placeholder="输入上游音色 ID 后回车或点添加…" />
      </div>

      <div v-if="capability.id === 'image'" class="field">
        <label>可选出图尺寸<span class="hint">首个胶囊为默认尺寸</span></label>
        <ChipEditor v-model="sizes" :presets="SIZE_PRESETS" placeholder="如 1024x1024 或 auto…" />
      </div>

      <div v-if="capability.id === 'realtime'" class="field">
        <label for="m-minutes">单次对话最长时长（1–240 分钟）</label>
        <div class="rt-row">
          <div class="affix-wrap" style="width: 140px">
            <input id="m-minutes" v-model.number="minutes" class="input num" type="number" min="1" max="240" step="1" required />
            <span class="affix">分钟</span>
          </div>
          <div class="seg-pills">
            <button v-for="v in REALTIME_PRESETS" :key="v" type="button" :class="{ active: minutes === v }" @click="minutes = v">
              {{ v }} 分
            </button>
          </div>
        </div>
      </div>

      <div class="switch-group">
        <SwitchRow
          v-model="requiresPaid"
          title="仅限付费额度使用"
          desc="开启后试用额度无法调用该媒体能力，客户端会提示用户兑换额度"
        />
        <SwitchRow
          v-model="enabled"
          title="启用这项能力"
          desc="关闭后保留配置参数，但向客户端返回 503 暂未开放"
        />
      </div>

      <div class="notice info" style="padding: 9px 12px; font-size: 12px">
        <Icon name="sparkles" :size="14" />
        <span>{{ capability.caution }}</span>
      </div>
    </div>

    <div class="dialog-foot">
      <button type="button" class="btn" @click="emit('close')">取消</button>
      <button type="submit" class="btn primary" :disabled="busy || !canSubmit">{{ busy ? '保存中…' : '保存并生效' }}</button>
    </div>
  </form>
</template>

<style scoped>
.head-lead { display: flex; align-items: center; gap: 11px; }
.title-row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.rt-row { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
</style>
