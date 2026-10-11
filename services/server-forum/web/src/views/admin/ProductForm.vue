<script setup lang="ts">
// 产品表单：新建和编辑共用。slug 会出现在产品页地址里，创建后不可修改。
import { computed, reactive } from 'vue'
import type { AdminProduct, ProductInput } from '@/api/admin'

const props = defineProps<{ product?: AdminProduct; busy?: boolean }>()
const emit = defineEmits<{ save: [body: ProductInput]; cancel: [] }>()

const editing = computed(() => !!props.product)

const form = reactive({
  slug: props.product?.slug ?? '',
  name: props.product?.name ?? '',
  vendor: props.product?.vendor ?? '',
  website: props.product?.website ?? '',
  description: props.product?.description ?? '',
})

function submit() {
  const body: ProductInput = {
    name: form.name.trim(),
    vendor: form.vendor.trim(),
    website: form.website.trim(),
    description: form.description.trim(),
  }
  if (!editing.value) body.slug = form.slug.trim()
  emit('save', body)
}
</script>

<template>
  <form class="adm-form" @submit.prevent="submit">
    <div class="adm-form-grid">
      <label v-if="!editing" class="field">
        <span>slug</span>
        <input v-model="form.slug" class="input" required maxlength="40" placeholder="openai" />
        <small>2-40 位小写字母、数字或连字符，创建后不可修改</small>
      </label>
      <label class="field">
        <span>产品名</span>
        <input v-model="form.name" class="input" required maxlength="40" />
      </label>
      <label class="field">
        <span>厂商</span>
        <input v-model="form.vendor" class="input" maxlength="40" />
      </label>
      <label class="field">
        <span>官网</span>
        <input v-model="form.website" class="input" type="url" maxlength="300" placeholder="https://" />
      </label>
      <label class="field wide">
        <span>产品说明</span>
        <textarea v-model="form.description" class="textarea" maxlength="500" rows="3" />
      </label>
    </div>
    <div class="adm-form-actions">
      <button class="btn btn-primary" type="submit" :disabled="busy">{{ editing ? '保存修改' : '创建产品' }}</button>
      <button class="btn btn-ghost" type="button" @click="emit('cancel')">取消</button>
    </div>
  </form>
</template>
