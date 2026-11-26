<script setup lang="ts">
// 站务联系方式：三列渠道卡片（公众号 / QQ 群 / 微信群），支持二维码即时预览与失效日期设置。
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { request } from '@/api/admin'
import { contactChannels, type ContactChannel, type ContactSettings } from '@/api/contact'
import AdminIcon from './AdminIcon.vue'
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
  onCleanup(() => {
    cancelled = true
  })
  const channels = contactChannels.filter(channel => current[channel.qr])
  const images = await Promise.allSettled(
    channels.map(channel => request<Blob>('GET', `/contact/qr/${channel.key}`, undefined, true, 'blob')),
  )
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
  <div class="adm-toolbar is-spread">
    <a class="btn btn-sm" href="/contact" target="_blank" rel="noopener noreferrer">
      <AdminIcon name="external" :size="13" /> 查看公开站务页（/contact）
    </a>
    <button v-if="config" class="btn btn-sm btn-primary" type="submit" form="contact-form" :disabled="busy">
      {{ busy ? '正在保存…' : '保存全部联系方式' }}
    </button>
  </div>

  <p v-if="error" class="error-text" role="alert">{{ error }}</p>
  <button v-if="!config && error" class="btn btn-sm" type="button" :disabled="busy" @click="load">重试</button>
  <p v-if="!config && busy" class="muted" role="status">正在读取联系方式…</p>

  <form v-if="config" id="contact-form" @submit.prevent="save">
    <fieldset class="contact-fields" :disabled="busy">
      <div class="contact-grid">
        <section v-for="channel in contactChannels" :key="channel.key" class="panel adm-card">
          <div class="adm-card-head">
            <h3>{{ channel.title }}</h3>
            <span class="adm-tag" :class="config[channel.qr] ? 'valid' : 'expired'">
              {{ config[channel.qr] ? '已配置二维码' : '未设二维码' }}
            </span>
          </div>
          <div class="adm-card-body contact-card-body">
            <div class="qr-row">
              <div class="qr-frame">
                <img
                  v-if="config[channel.qr] && (pendingPreviews.has(channel.key) || savedPreviews.has(channel.key))"
                  :alt="`${channel.title}二维码预览`"
                  :src="pendingPreviews.get(channel.key) ?? savedPreviews.get(channel.key)"
                />
                <span v-else class="muted" style="font-size: 11px">暂无二维码</span>
              </div>
              <div style="display: grid; gap: 6px; align-content: start">
                <label class="btn btn-sm contact-upload">
                  {{ config[channel.qr] ? '更换二维码' : '上传二维码' }}
                  <input type="file" accept=".png,.jpg,.jpeg,.webp" @change="choose(channel, $event)" />
                </label>
                <button
                  v-if="config[channel.qr]"
                  class="btn btn-sm btn-ghost btn-danger"
                  type="button"
                  :aria-label="`移除${channel.title}二维码`"
                  @click="remove(channel)"
                >
                  移除二维码
                </button>
                <small class="muted" style="font-size: 11px">≤ 5 MiB PNG/JPG/WebP</small>
              </div>
            </div>

            <label class="field">
              <span>{{ channel.title }}名称</span>
              <input v-model="config[channel.name]" class="input" maxlength="100" />
            </label>
            <template v-if="channel.key === 'qq-group'">
              <label class="field">
                <span>QQ 群号</span>
                <input v-model.trim="config.qqGroupNumber" class="input adm-mono" inputmode="numeric" pattern="[0-9]+" maxlength="32" />
              </label>
              <label class="field">
                <span>加群链接（可选）</span>
                <input v-model.trim="config.qqGroupInviteUrl" class="input" type="url" maxlength="2048" />
              </label>
            </template>
            <label v-if="channel.key === 'wechat-group' && config.wechatGroupQr" class="field">
              <span>失效日期（中国标准时间，可选）</span>
              <input
                type="date"
                class="input"
                :value="config.wechatGroupExpiresOn ?? ''"
                @input="config.wechatGroupExpiresOn = ($event.target as HTMLInputElement).value || null"
              />
              <small>到这一天零时，社区站务会自动隐藏该二维码。</small>
            </label>
          </div>
        </section>
      </div>
    </fieldset>
  </form>
</template>

<style scoped>
.contact-fields { border: 0; margin: 0; padding: 0; min-width: 0; }
.contact-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 16px; align-items: start; }
.contact-card-body { display: grid; gap: 12px; }
.qr-row { display: flex; align-items: center; gap: 14px; }
.qr-frame {
  width: 92px; height: 92px; flex: none; border-radius: 10px;
  border: 1px solid var(--adm-line); background: #fff;
  display: grid; place-items: center; overflow: hidden; padding: 6px;
}
.qr-frame img { width: 100%; height: 100%; object-fit: contain; }
.contact-upload { position: relative; overflow: hidden; display: inline-flex; }
.contact-upload input { position: absolute; inset: 0; opacity: 0; cursor: pointer; width: 100%; }
@media (max-width: 1023px) { .contact-grid { grid-template-columns: 1fr; } }
</style>
