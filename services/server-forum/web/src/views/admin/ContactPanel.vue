<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { request } from '@/api/admin'
import { contactChannels, type ContactChannel, type ContactSettings } from '@/api/contact'
import { errorText, notify } from './shared'

const config = ref<ContactSettings>()
const replacements = new Map<ContactChannel, File>()
const pendingPreviews = ref(new Map<ContactChannel, string>())
const savedPreviews = ref(new Map<ContactChannel, string>())
const busy = ref(false)
const error = ref('')

function clearPreviews(previews: Map<ContactChannel, string>) {
  for (const url of previews.values()) URL.revokeObjectURL(url)
  previews.clear()
}

watch(config, async (current, _, onCleanup) => {
  clearPreviews(savedPreviews.value)
  if (!current) return
  let cancelled = false
  onCleanup(() => { cancelled = true })
  const channels = contactChannels.filter(channel => current[channel.qr])
  const images = await Promise.allSettled(channels.map(channel => request<Blob>('GET', `/contact/qr/${channel.key}`, undefined, true, 'blob')))
  if (cancelled) return
  for (const [index, image] of images.entries()) {
    if (image.status === 'fulfilled') savedPreviews.value.set(channels[index]!.key, URL.createObjectURL(image.value))
  }
})

async function load() {
  busy.value = true
  error.value = ''
  try {
    config.value = await request<ContactSettings>('GET', '/contact')
  } catch (cause) {
    error.value = errorText(cause)
  } finally {
    busy.value = false
  }
}

function clearReplacement(channel: ContactChannel) {
  const url = pendingPreviews.value.get(channel)
  if (url) URL.revokeObjectURL(url)
  pendingPreviews.value.delete(channel)
  replacements.delete(channel)
}

function choose(channel: (typeof contactChannels)[number], event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file || !config.value) return
  if (!/\.(png|jpe?g|webp)$/i.test(file.name) || file.size > 5 * 1024 * 1024) {
    error.value = '请选择 5 MiB 以内的 PNG、JPG 或 WebP 图片。'
    return
  }
  error.value = ''
  clearReplacement(channel.key)
  replacements.set(channel.key, file)
  pendingPreviews.value.set(channel.key, URL.createObjectURL(file))
  config.value[channel.qr] = true
  if (channel.key === 'wechat-group') config.value.wechatGroupExpiresOn = null
}

function remove(channel: (typeof contactChannels)[number]) {
  if (!config.value) return
  clearReplacement(channel.key)
  config.value[channel.qr] = false
  if (channel.key === 'wechat-group') config.value.wechatGroupExpiresOn = null
}

async function save() {
  if (!config.value || busy.value) return
  const body = new FormData()
  body.append('config', JSON.stringify(config.value))
  for (const [channel, file] of replacements) body.append(channel, file)
  busy.value = true
  error.value = ''
  try {
    config.value = await request<ContactSettings>('PUT', '/contact', body)
    clearPreviews(pendingPreviews.value)
    replacements.clear()
    notify('联系方式已保存')
  } catch (cause) {
    error.value = errorText(cause)
  } finally {
    busy.value = false
  }
}

onMounted(load)
onUnmounted(() => {
  clearPreviews(savedPreviews.value)
  clearPreviews(pendingPreviews.value)
})
</script>

<template>
  <div class="adm-toolbar">
    <a class="btn" href="/contact" target="_blank" rel="noopener noreferrer">查看站务页</a>
  </div>
  <p v-if="error" role="alert">{{ error }}</p>
  <button v-if="!config && error" class="btn" type="button" :disabled="busy" @click="load">重试</button>
  <p v-if="!config && busy" role="status">正在读取联系方式…</p>
  <form v-if="config" @submit.prevent="save">
    <fieldset class="contact-fields" :disabled="busy">
      <section v-for="channel in contactChannels" :key="channel.key" class="panel adm-card contact-editor">
        <h2>{{ channel.title }}</h2>
        <label class="field">
          <span>{{ channel.title }}名称</span>
          <input v-model="config[channel.name]" maxlength="100">
        </label>
        <template v-if="channel.key === 'qq-group'">
          <label class="field"><span>QQ 群号</span><input v-model.trim="config.qqGroupNumber" inputmode="numeric" pattern="[0-9]+" maxlength="32"></label>
          <label class="field"><span>加群链接（可选）</span><input v-model.trim="config.qqGroupInviteUrl" type="url" maxlength="2048"></label>
        </template>
        <label class="field">
          <span>{{ channel.title }}二维码（最大 5 MiB）</span>
          <input type="file" accept=".png,.jpg,.jpeg,.webp" @change="choose(channel, $event)">
        </label>
        <div v-if="config[channel.qr]" class="qr-preview">
          <img v-if="pendingPreviews.has(channel.key) || savedPreviews.has(channel.key)" :alt="`${channel.title}二维码预览`" :src="pendingPreviews.get(channel.key) ?? savedPreviews.get(channel.key)">
          <button class="btn" type="button" :aria-label="`移除${channel.title}二维码`" @click="remove(channel)">移除二维码</button>
        </div>
        <label v-if="channel.key === 'wechat-group' && config.wechatGroupQr" class="field">
          <span>失效日期（中国标准时间，可选）</span>
          <input type="date" :value="config.wechatGroupExpiresOn ?? ''" @input="config.wechatGroupExpiresOn = ($event.target as HTMLInputElement).value || null">
          <small class="muted">到这一天时，社区站务会自动隐藏该二维码。</small>
        </label>
      </section>
      <button class="btn btn-primary" type="submit">{{ busy ? '正在保存…' : '保存联系方式' }}</button>
    </fieldset>
  </form>
</template>

<style scoped>
.contact-fields { border: 0; margin: 0; padding: 0; min-width: 0; }
.contact-editor { display: grid; gap: 12px; margin: 0 0 16px; padding: 18px; max-width: 620px; }
.contact-editor h2 { margin: 0; font-size: 17px; }
.qr-preview { display: flex; flex-wrap: wrap; align-items: center; gap: 12px; }
.qr-preview img { width: 90px; height: 90px; object-fit: contain; background: #fff; }
</style>
