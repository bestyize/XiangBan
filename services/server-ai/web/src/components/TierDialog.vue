<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { api, errorMessage } from '../api';
import { useModal } from '../composables/useModal';
import type { ModelOption, Product, Tier, TierCapabilities } from '../types';
import Icon from './Icon.vue';
import ModelPicker from './ModelPicker.vue';
import SwitchRow from './SwitchRow.vue';

const props = defineProps<{ product: Product; tier: Tier | null; models: ModelOption[] }>();
const emit = defineEmits<{ close: []; saved: [warnings: string[]] }>();

const root = ref<HTMLElement | null>(null);
useModal(root, () => emit('close'));

const isNew = computed(() => props.tier === null);
const slug = ref(props.tier?.tier ?? '');
const displayName = ref(props.tier?.display_name ?? '');
const description = ref(props.tier?.description ?? '');
const model = ref(props.tier?.model ?? '');
const requiresPaid = ref(props.tier?.requires_paid ?? false);
const isDefault = ref(props.tier?.is_default ?? false);
const sortOrder = ref(props.tier?.sort_order ?? 10);
const enabled = ref(props.tier?.enabled ?? true);
const reasoningMode = ref<'auto' | 'manual' | 'off'>(
  props.tier?.reasoning_efforts == null ? 'auto' : props.tier.reasoning_efforts.length ? 'manual' : 'off',
);
const reasoningLevels = ref<string[]>(
  props.tier?.reasoning_efforts?.length ? [...props.tier.reasoning_efforts] : ['low', 'high', 'max'],
);
const contextWindow = ref<string>(
  props.tier?.capabilities?.context_window_tokens ? String(props.tier.capabilities.context_window_tokens) : '',
);
const maxOutput = ref<string>(
  props.tier?.capabilities?.max_output_tokens ? String(props.tier.capabilities.max_output_tokens) : '',
);
const busy = ref(false);
const error = ref('');

const ALL_LEVELS = ['low', 'medium', 'high', 'xhigh', 'max'] as const;
const picked = computed(() => props.models.find((m) => m.model_name === model.value) ?? null);

watch(isDefault, (on) => {
  if (on) {
    requiresPaid.value = false;
    enabled.value = true;
  }
});

function toggleLevel(lv: string) {
  if (reasoningLevels.value.includes(lv)) {
    reasoningLevels.value = reasoningLevels.value.filter((x) => x !== lv);
  } else {
    reasoningLevels.value = [...reasoningLevels.value, lv];
  }
}

function buildCapabilities(): TierCapabilities | null {
  const ctx = contextWindow.value.trim() ? Number(contextWindow.value.trim()) : null;
  const out = maxOutput.value.trim() ? Number(maxOutput.value.trim()) : null;
  const prev = props.tier?.capabilities;
  const supports_tools = prev?.supports_tools ?? null;
  const supports_vision = prev?.supports_vision ?? null;
  if (ctx === null && out === null && supports_tools === null && supports_vision === null) return null;
  return {
    context_window_tokens: ctx,
    max_output_tokens: out,
    supports_tools,
    supports_vision,
  };
}

async function submit() {
  if (!model.value) return;
  busy.value = true;
  error.value = '';
  try {
    const result = await api.saveTier(props.product.product_id, slug.value.trim(), {
      display_name: displayName.value.trim(),
      description: description.value.trim(),
      model: model.value,
      requires_paid: requiresPaid.value,
      is_default: isDefault.value,
      sort_order: Number(sortOrder.value) || 0,
      enabled: enabled.value,
      reasoning_efforts: reasoningMode.value === 'auto' ? null : reasoningMode.value === 'off' ? [] : reasoningLevels.value,
      capabilities: buildCapabilities(),
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
  <form ref="root" class="dialog wide" role="dialog" aria-modal="true" aria-labelledby="tier-title" @submit.prevent="submit">
    <div class="dialog-head">
      <div class="head-lead">
        <span class="modal-head-ico"><Icon name="layers" :size="17" /></span>
        <div>
          <h2 id="tier-title">{{ isNew ? '添加档位' : '编辑档位' }} · {{ product.name }}</h2>
          <p>左侧名称面向客户端展示，真实模型与倍率仅保存在网关服务端。</p>
        </div>
      </div>
      <button type="button" class="btn icon sm ghost" aria-label="关闭" @click="emit('close')"><Icon name="x" :size="16" /></button>
    </div>

    <div class="dialog-body">
      <div v-if="error" class="notice error" role="alert"><Icon name="circle-alert" :size="16" /><span>{{ error }}</span></div>

      <div class="form-section">
        <div class="form-section-title"><Icon name="eye" :size="13" />客户端展示信息</div>
        <div class="pair-grid">
          <div class="field">
            <label for="t-name">档位名称</label>
            <input id="t-name" v-model="displayName" class="input" maxlength="20" placeholder="如：日常 / 深入" required :autofocus="isNew" />
          </div>
          <div class="field">
            <label for="t-slug">档位标识（slug）</label>
            <input id="t-slug" v-model="slug" class="input mono" :readonly="!isNew" placeholder="如：standard" required />
          </div>
        </div>
        <div class="pair-grid">
          <div class="field">
            <label for="t-desc">一句话说明</label>
            <input id="t-desc" v-model="description" class="input" maxlength="100" placeholder="展示在客户端档位选择器下方" />
          </div>
          <div class="field">
            <label for="t-sort">显示排序<span class="hint">越小越靠前</span></label>
            <input id="t-sort" v-model.number="sortOrder" class="input num" type="number" step="1" />
          </div>
        </div>
      </div>

      <div class="form-section">
        <div class="form-section-title"><Icon name="eye-off" :size="13" />幕后真实路由与能力</div>
        <div class="field">
          <label>实际调用模型<span class="hint">点击卡片展开搜索</span></label>
          <ModelPicker v-model="model" :models="models" recommend="chat" placeholder="选择 AI 网关里已启用的模型…" />
        </div>

        <div v-if="picked?.ratio_warning" class="notice warn">
          <Icon name="alert" :size="15" />
          <span>该模型倍率与 New API 未配置默认值相同，继续保存可能导致计费偏差，建议先在 New API 核对真实定价。</span>
        </div>
        <div v-else-if="model && !picked && models.length" class="notice error">
          <Icon name="circle-alert" :size="15" />
          <span>AI 网关里已经找不到这个模型，请重新选择。</span>
        </div>

        <div class="field">
          <label>思考等级开放策略</label>
          <div class="seg-pills full">
            <button type="button" :class="{ active: reasoningMode === 'auto' }" @click="reasoningMode = 'auto'">按模型自动识别</button>
            <button type="button" :class="{ active: reasoningMode === 'manual' }" @click="reasoningMode = 'manual'">手动声明等级</button>
            <button type="button" :class="{ active: reasoningMode === 'off' }" @click="reasoningMode = 'off'">不开放调整</button>
          </div>
        </div>

        <div v-if="reasoningMode === 'manual'" class="field">
          <label>允许客户端传入的思考等级</label>
          <div class="effort-pills">
            <button
              v-for="lv in ALL_LEVELS"
              :key="lv"
              type="button"
              class="effort-pill"
              :class="{ on: reasoningLevels.includes(lv) }"
              @click="toggleLevel(lv)"
            >
              <Icon v-if="reasoningLevels.includes(lv)" name="check" :size="11" :stroke="2.6" />
              {{ lv }}
            </button>
          </div>
          <span class="hint">只向客户端公开思考等级选项，不泄露真实模型名。</span>
        </div>

        <div class="pair-grid">
          <div class="field">
            <label for="t-ctx">上下文窗口（选填）</label>
            <div class="affix-wrap">
              <input id="t-ctx" v-model="contextWindow" class="input num" type="number" min="1" max="10000000" placeholder="如 200000" />
              <span class="affix">tokens</span>
            </div>
          </div>
          <div class="field">
            <label for="t-out">最大输出（选填）</label>
            <div class="affix-wrap">
              <input id="t-out" v-model="maxOutput" class="input num" type="number" min="1" max="10000000" placeholder="如 16384" />
              <span class="affix">tokens</span>
            </div>
          </div>
        </div>
      </div>

      <div class="switch-group">
        <SwitchRow
          v-model="isDefault"
          title="设为默认档位"
          desc="客户端未指定档位时默认使用；默认档会自动保持启用且试用可用"
        />
        <SwitchRow
          v-model="requiresPaid"
          title="仅限付费额度使用"
          desc="开启后，仅有试用额度的用户调用会返回 403 提示兑换"
          :disabled="isDefault"
        />
        <SwitchRow
          v-model="enabled"
          title="启用该档位"
          desc="停用后新请求会被拒绝，在途请求不受影响"
          :disabled="isDefault"
        />
      </div>
    </div>

    <div class="dialog-foot">
      <button type="button" class="btn" @click="emit('close')">取消</button>
      <button type="submit" class="btn primary" :disabled="busy || !model">{{ busy ? '保存中…' : '保存并立即生效' }}</button>
    </div>
  </form>
</template>

<style scoped>
.head-lead { display: flex; align-items: center; gap: 11px; }
.seg-pills.full { width: 100%; }
.seg-pills.full button { flex: 1; justify-content: center; }
.effort-pills { display: flex; flex-wrap: wrap; gap: 6px; }
.effort-pill {
  height: 28px;
  padding: 0 11px;
  border-radius: 7px;
  border: 1px solid var(--line-strong);
  background: var(--surface);
  font-family: var(--mono);
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-2);
  display: inline-flex;
  align-items: center;
  gap: 5px;
  transition: 0.12s;
}
.effort-pill:hover { border-color: var(--line-focus); color: var(--ink); }
.effort-pill.on {
  background: var(--btn-primary-bg);
  border-color: var(--btn-primary-bg);
  color: var(--btn-primary-ink);
}
</style>
