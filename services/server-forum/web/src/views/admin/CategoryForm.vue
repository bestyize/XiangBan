<script setup lang="ts">
// 分类表单：新建和编辑共用。编辑时 slug、父分类、类型不可改。
import { computed, reactive, watch } from 'vue'
import type { AdminCategory, CategoryInput } from '@/api/admin'
import { CATEGORY_KIND, TRUST_LEVELS } from './shared'

const props = defineProps<{
  /** 传入则为编辑模式 */
  category?: AdminCategory
  /** 可选的父分类（一级分类） */
  parents: AdminCategory[]
  busy?: boolean
}>()
const emit = defineEmits<{ save: [body: CategoryInput]; cancel: [] }>()

const editing = computed(() => !!props.category)

const form = reactive({
  parent_id: props.category?.parent_id ?? (null as number | null),
  slug: props.category?.slug ?? '',
  kind: (props.category?.kind ?? 'normal') as 'deal' | 'normal',
  name: props.category?.name ?? '',
  description: props.category?.description ?? '',
  sort_order: props.category?.sort_order ?? 0,
  min_post_level: props.category?.min_post_level ?? 1,
  on_home: props.category?.on_home ?? true,
  award_points: props.category?.award_points ?? true,
  admin_only: props.category?.admin_only ?? false,
  // 内置标签在表单里用逗号分隔的一行文字编辑。
  preset_tags: (props.category?.preset_tags ?? []).join('，'),
})

// 选了父分类后，子分类类型必须和父分类一致。
watch(
  () => form.parent_id,
  id => {
    const p = props.parents.find(c => c.id === id)
    if (p) form.kind = p.kind
  },
)
const kindLocked = computed(() => editing.value || form.parent_id !== null)

function submit() {
  const body: CategoryInput = {
    name: form.name.trim(),
    description: form.description.trim(),
    sort_order: Number(form.sort_order) || 0,
    min_post_level: Number(form.min_post_level),
    on_home: form.on_home,
    award_points: form.award_points,
    admin_only: form.admin_only,
    preset_tags: form.preset_tags.split(/[,，、\s]+/).map(t => t.trim()).filter(Boolean),
  }
  if (!editing.value) {
    body.parent_id = form.parent_id
    body.slug = form.slug.trim()
    body.kind = form.kind
  }
  emit('save', body)
}
</script>

<template>
  <form class="adm-form" @submit.prevent="submit">
    <div class="adm-form-grid">
      <label v-if="!editing" class="field">
        <span>父分类</span>
        <select v-model="form.parent_id" class="select" style="width: 100%">
          <option :value="null">无（一级分类）</option>
          <option v-for="p in parents" :key="p.id" :value="p.id">{{ p.name }}</option>
        </select>
      </label>
      <label v-if="!editing" class="field">
        <span>slug</span>
        <input v-model="form.slug" class="input" required maxlength="40" placeholder="deals-example" />
        <small>2-40 位小写字母、数字或连字符，创建后不可修改</small>
      </label>
      <label class="field">
        <span>类型</span>
        <select v-model="form.kind" class="select" style="width: 100%" :disabled="kindLocked">
          <option v-for="(label, key) in CATEGORY_KIND" :key="key" :value="key">{{ label }}</option>
        </select>
      </label>
      <label class="field">
        <span>名称</span>
        <input v-model="form.name" class="input" required maxlength="20" />
      </label>
      <label class="field">
        <span>排序值</span>
        <input v-model.number="form.sort_order" class="input" type="number" />
        <small>数字越小越靠前</small>
      </label>
      <label class="field">
        <span>最低发帖等级</span>
        <select v-model.number="form.min_post_level" class="select" style="width: 100%">
          <option v-for="t in TRUST_LEVELS" :key="t.level" :value="t.level">{{ t.level }} {{ t.name }}</option>
        </select>
      </label>
      <label class="field wide">
        <span>说明</span>
        <input v-model="form.description" class="input" maxlength="200" />
      </label>
      <label class="field wide">
        <span>内置标签</span>
        <input v-model="form.preset_tags" class="input" placeholder="用逗号或空格分隔，比如：windows，macos，开源" />
        <small>发帖选了这个分类后，这些标签会出现在标签栏下面，点一下就加上；最多 20 个，每个最多 20 个字。只对二级分类有用。</small>
      </label>
    </div>
    <div class="adm-checks">
      <label><input v-model="form.on_home" type="checkbox" />显示在首页</label>
      <label><input v-model="form.award_points" type="checkbox" />发帖奖励积分</label>
      <label><input v-model="form.admin_only" type="checkbox" />仅管理员可发帖</label>
    </div>
    <div class="adm-form-actions">
      <button class="btn btn-primary" type="submit" :disabled="busy">{{ editing ? '保存修改' : '创建分类' }}</button>
      <button class="btn btn-ghost" type="button" @click="emit('cancel')">取消</button>
    </div>
  </form>
</template>
