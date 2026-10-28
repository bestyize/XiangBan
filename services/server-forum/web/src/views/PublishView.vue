<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ContentShell from '@/components/ContentShell.vue'
import { contentApi, type PackDetail } from '@/api/content'
import { COVER_LIMIT, fmtSize, KIND_LIMIT, PACK_KINDS } from '@/lib/content'
import { uploadFile } from '@/lib/upload'
import { useHead } from '@/lib/head'
import { track } from '@/lib/analytics'
import { useToast } from '@/stores/toast'

const route = useRoute()
const router = useRouter()
const toast = useToast()

/** 带 ?pack=编号 时是给自己已发布的内容包传新版本，类型与标题沿用原来的。 */
const packId = computed(() => (route.query.pack ? Number(route.query.pack) : null))
const existing = ref<PackDetail | null>(null)
useHead(() => ({ title: packId.value ? '发布新版本' : '发布内容包' }))

const form = reactive({ kind: 'effect', title: '', summary: '', tags: '', changelog: '', rights: false })
const zip = ref<File | null>(null)
const cover = ref<File | null>(null)
const busy = ref(false)
const step = ref('')
const progress = ref(0)
const error = ref('')

watch(
  packId,
  async id => {
    existing.value = null
    if (!id) return
    try {
      existing.value = await contentApi.pack(id)
      form.kind = existing.value.kind
    } catch (e) {
      toast.error(e)
    }
  },
  { immediate: true },
)

const kind = computed(() => existing.value?.kind ?? form.kind)
const limit = computed(() => KIND_LIMIT[kind.value] ?? 0)

function pickZip(e: Event) {
  zip.value = (e.target as HTMLInputElement).files?.[0] ?? null
}
function pickCover(e: Event) {
  cover.value = (e.target as HTMLInputElement).files?.[0] ?? null
}

function validate(): string {
  if (!zip.value) return '请选择内容包压缩包（.zip）'
  if (!zip.value.name.toLowerCase().endsWith('.zip')) return '内容包需要是 .zip 文件'
  if (zip.value.size > limit.value) return `这个类型的内容包不能超过 ${fmtSize(limit.value)}`
  if (cover.value && cover.value.size > COVER_LIMIT) return `封面不能超过 ${fmtSize(COVER_LIMIT)}`
  if (!packId.value) {
    if (form.title.trim().length < 1) return '请填写标题'
    if (!form.rights) return '请确认你拥有这份内容的发布权利'
  }
  return ''
}

async function submit() {
  error.value = validate()
  if (error.value || !zip.value) return
  busy.value = true
  try {
    step.value = '上传内容包'
    progress.value = 0
    const fileKey = await uploadFile(kind.value, zip.value, r => (progress.value = r))
    if (packId.value) {
      step.value = '发布新版本'
      await contentApi.addVersion(packId.value, fileKey, form.changelog.trim())
      toast.show('新版本已发布，正在等待重新审核')
      router.replace(`/packs/${packId.value}`)
      return
    }
    let coverKey: string | undefined
    if (cover.value) {
      step.value = '上传封面'
      progress.value = 0
      coverKey = await uploadFile('cover', cover.value, r => (progress.value = r))
    }
    step.value = '发布'
    const res = await contentApi.publish({
      kind: form.kind,
      title: form.title.trim(),
      summary: form.summary.trim(),
      tags: form.tags.split(/[,，、\s]+/).filter(Boolean),
      file_key: fileKey,
      cover_key: coverKey,
      rights_ack: form.rights,
    })
    track('publish_done', form.kind)
    toast.show('已发布')
    router.replace(`/packs/${res.id}`)
  } catch (e) {
    error.value = e instanceof Error ? e.message : '发布失败，请稍后重试'
  } finally {
    busy.value = false
    step.value = ''
  }
}
</script>

<template>
  <ContentShell>
    <form class="publish" novalidate @submit.prevent="submit">
      <h1>{{ packId ? `发布新版本${existing ? `：${existing.title}` : ''}` : '发布内容包' }}</h1>

      <label v-if="!packId" class="field">
        <span>类型</span>
        <select v-model="form.kind" class="select">
          <option v-for="k in PACK_KINDS" :key="k.key" :value="k.key">{{ k.label }}</option>
        </select>
        <small>特效和提醒发布后别人可以直接一键应用；其他类型需要审核通过后才能一键应用。</small>
      </label>

      <label class="field">
        <span>内容包（.zip，不超过 {{ fmtSize(limit) }}）</span>
        <input type="file" accept=".zip,application/zip" @change="pickZip" />
        <small>压缩包根目录需要有 <code>manifest.json</code>；特效、提醒、任务这类纯数据内容还需要 <code>payload.json</code>。上传后由服务器检查，不合格会说明原因。</small>
      </label>

      <template v-if="!packId">
        <label class="field">
          <span>封面（可选，PNG / JPEG / WebP，不超过 {{ fmtSize(COVER_LIMIT) }}）</span>
          <input type="file" accept="image/png,image/jpeg,image/webp" @change="pickCover" />
        </label>
        <label class="field">
          <span>标题</span>
          <input v-model="form.title" class="input" maxlength="60" />
        </label>
        <label class="field">
          <span>简介</span>
          <textarea v-model="form.summary" class="input" rows="4" maxlength="500" />
        </label>
        <label class="field">
          <span>标签（最多 8 个，空格或逗号分隔）</span>
          <input v-model="form.tags" class="input" />
        </label>
        <label class="check">
          <input v-model="form.rights" type="checkbox" />
          我确认这是我的原创，或已获得授权，并同意按社区规则发布。
        </label>
      </template>
      <label v-else class="field">
        <span>更新说明</span>
        <textarea v-model="form.changelog" class="input" rows="3" maxlength="500" />
        <small>新版本会重新进入审核，审核通过前以未审核状态展示。</small>
      </label>

      <p v-if="error" class="error" role="alert">{{ error }}</p>
      <p v-if="busy" class="progress">{{ step }}<template v-if="step.startsWith('上传')"> {{ Math.round(progress * 100) }}%</template></p>
      <div class="actions">
        <button class="btn btn-primary" type="submit" :disabled="busy">{{ packId ? '发布新版本' : '发布' }}</button>
        <RouterLink class="btn" to="/me/works">我的作品</RouterLink>
      </div>
    </form>
  </ContentShell>
</template>

<style scoped>
.publish {
  padding: 16px;
  display: grid;
  gap: 14px;
}
h1 {
  margin: 0;
  font-size: var(--fs-xl);
  font-weight: 600;
}
.field {
  display: grid;
  gap: 4px;
  font-size: var(--fs-sm);
}
.field > span {
  font-weight: 500;
}
.field small {
  color: var(--ink-3);
}
.check {
  display: flex;
  gap: 8px;
  font-size: var(--fs-sm);
  align-items: flex-start;
}
.error {
  margin: 0;
  color: var(--brand);
  font-size: var(--fs-sm);
}
.progress {
  margin: 0;
  color: var(--ink-2);
  font-size: var(--fs-sm);
}
.actions {
  display: flex;
  gap: 8px;
}
</style>
