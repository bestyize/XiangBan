<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { api, errorMessage } from '../api';
import type { ModelOption, Product, Tier } from '../types';

const props = defineProps<{ product: Product; tier: Tier | null; models: ModelOption[] }>();
const emit = defineEmits<{ close: []; saved: [warnings: string[]] }>();

const isNew = computed(() => props.tier === null);
const slug = ref(props.tier?.tier ?? '');
const displayName = ref(props.tier?.display_name ?? '');
const description = ref(props.tier?.description ?? '');
const model = ref(props.tier?.model ?? '');
const requiresPaid = ref(props.tier?.requires_paid ?? false);
const isDefault = ref(props.tier?.is_default ?? false);
const sortOrder = ref(props.tier?.sort_order ?? 10);
const enabled = ref(props.tier?.enabled ?? true);
const busy = ref(false);
const error = ref('');

const picked = computed(() => props.models.find((m) => m.model_name === model.value) ?? null);
// 已保存的模型如果从网关消失了，仍要显示在下拉里，否则用户会误以为没选。
const options = computed(() => {
  const list = props.models.map((m) => m.model_name);
  return model.value && !list.includes(model.value) ? [model.value, ...list] : list;
});

// 默认档必须启用且免费，否则新用户无法直接使用；勾选默认时自动收敛这两项，免得保存后才被拒绝。
watch(isDefault, (on) => {
  if (on) {
    requiresPaid.value = false;
    enabled.value = true;
  }
});

async function submit() {
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
  <form class="dialog" role="dialog" aria-modal="true" aria-labelledby="tier-title" @submit.prevent="submit" @keydown.esc="emit('close')">
    <div class="dialog-head">
      <h2 id="tier-title">{{ isNew ? '添加档位' : '修改档位' }} · {{ product.name }}</h2>
    </div>
    <div class="dialog-body">
      <div v-if="error" class="notice error" role="alert">{{ error }}</div>

      <fieldset class="front">
        <legend>用户会看到</legend>
        <div class="field">
          <label for="t-name">档位名称</label>
          <input id="t-name" v-model="displayName" class="input" maxlength="20" placeholder="标准" required :autofocus="isNew" />
        </div>
        <div class="field">
          <label for="t-desc">一句话说明</label>
          <input id="t-desc" v-model="description" class="input" maxlength="100" placeholder="日常使用，响应快" />
        </div>
        <div class="field">
          <label for="t-slug">档位标识</label>
          <input id="t-slug" v-model="slug" class="input mono" :readonly="!isNew" placeholder="standard" required />
          <span class="hint">客户端请求时用它指定档位。{{ isNew ? '小写字母开头，创建后不能修改。' : '创建后不能修改。' }}</span>
        </div>
      </fieldset>

      <fieldset class="back">
        <legend>幕后调用（用户看不到）</legend>
        <div class="field">
          <label for="t-model">实际模型</label>
          <select id="t-model" v-model="model" class="select mono" required>
            <option value="" disabled>选择 AI 网关里已启用的模型</option>
            <option v-for="name in options" :key="name" :value="name">{{ name }}</option>
          </select>
          <span v-if="picked" class="hint num">
            输入倍率 {{ picked.model_ratio }}，输出倍率 {{ picked.completion_ratio }}
          </span>
        </div>
        <div v-if="picked?.ratio_warning" class="notice warn">
          这个模型的倍率和 New API 未配置时的默认值相同，很可能还没设置真实定价。继续保存可能导致扣费与成本不符，建议先去 New API 核对。
        </div>
        <div v-else-if="model && !picked" class="notice error">AI 网关里已经找不到这个模型，请换一个。</div>
      </fieldset>

      <div class="rules">
        <label class="check"><input v-model="isDefault" type="checkbox" /> 设为默认档位（客户端没指定档位时使用）</label>
        <label class="check" :class="{ disabled: isDefault }"><input v-model="requiresPaid" type="checkbox" :disabled="isDefault" /> 需要先兑换额度才能使用（试用额度不能用）</label>
        <label class="check" :class="{ disabled: isDefault }"><input v-model="enabled" type="checkbox" :disabled="isDefault" /> 启用</label>
      </div>

      <div class="field order">
        <label for="t-sort">排序</label>
        <input id="t-sort" v-model.number="sortOrder" class="input num" type="number" step="1" />
        <span class="hint">数字小的排在前面。</span>
      </div>
    </div>
    <div class="dialog-foot">
      <button type="button" class="btn" @click="emit('close')">取消</button>
      <button type="submit" class="btn primary" :disabled="busy || !model">{{ busy ? '保存中…' : '保存档位' }}</button>
    </div>
  </form>
</template>

<style scoped>
fieldset { border: 1px solid var(--line); border-radius: var(--radius); margin: 0; padding: 4px 14px 14px; display: flex; flex-direction: column; gap: 12px; min-width: 0; }
legend { padding: 0 6px; font-weight: 600; font-size: 13px; }
.back { background: var(--curtain); }
.rules { display: flex; flex-direction: column; gap: 8px; }
.check.disabled { opacity: 0.55; cursor: default; }
.order { max-width: 140px; }
</style>
