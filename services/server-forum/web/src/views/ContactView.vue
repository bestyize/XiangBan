<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { contactChannels, contactQrUrl, getContact, type ContactChannel } from '@/api/contact'
import RequestState from '@/components/community/RequestState.vue'
import { useCommunityResource } from '@/components/community/resource'
import { useHead } from '@/lib/head'
import { useToast } from '@/stores/toast'

const { data: contact, loading, error, load } = useCommunityResource(getContact)
const toast = useToast()
const failedImages = ref(new Set<ContactChannel>())
const dateFormat = new Intl.DateTimeFormat('zh-CN', { month: 'long', day: 'numeric', timeZone: 'UTC' })

function reload() {
  failedImages.value.clear()
  void load()
}

async function copy(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    toast.show('已复制')
  } catch {
    toast.show('复制失败，请选中文字后复制', 'error')
  }
}

useHead(() => ({ title: '站务' }))
onMounted(reload)
</script>

<template>
  <section class="contact-page" aria-labelledby="contact-heading">
    <header class="contact-heading">
      <h1 id="contact-heading">联系我们</h1>
      <p class="mobile-tip">可长按二维码保存到相册，再用微信或 QQ 的扫一扫识别。</p>
    </header>
    <RequestState :loading="loading" :error="error" @retry="reload" />
    <div v-if="contact" class="contact-grid">
      <section v-for="channel in contactChannels" :key="channel.key" class="panel contact-card" :aria-labelledby="channel.key">
        <header class="channel-heading">
          <h2 :id="channel.key">{{ channel.title }}</h2>
          <p v-if="contact[channel.name]" class="channel-name">{{ contact[channel.name] }}</p>
          <p v-if="channel.key === 'qq-group' && contact.qqGroupNumber" class="group-number">群号 <b>{{ contact.qqGroupNumber }}</b></p>
        </header>

        <figure v-if="contact[channel.qr] && !failedImages.has(channel.key)" class="channel-qr">
          <img :src="contactQrUrl(channel.key)" :alt="`${channel.title}二维码`" width="180" height="180" @error="failedImages.add(channel.key)">
          <figcaption>
            {{ channel.hint }}
            <time v-if="channel.key === 'wechat-group' && contact.wechatGroupExpiresOn" :datetime="contact.wechatGroupExpiresOn">{{ dateFormat.format(new Date(contact.wechatGroupExpiresOn)) }}前有效</time>
          </figcaption>
        </figure>
        <p v-else-if="contact[channel.qr]" class="channel-status">二维码暂时无法加载</p>
        <p v-else-if="channel.key === 'wechat-group' && contact.wechatGroupExpiresOn" class="channel-status">群二维码已过期，等待更新。</p>
        <p v-else-if="!contact[channel.name] || channel.key === 'wechat-group'" class="channel-status">二维码暂未提供</p>

        <div class="channel-actions">
          <button v-if="channel.key === 'wechat-official' && contact.wechatOfficialName" type="button" class="btn" @click="copy(contact.wechatOfficialName)">复制名称</button>
          <template v-if="channel.key === 'qq-group'">
            <button v-if="contact.qqGroupNumber" type="button" class="btn" @click="copy(contact.qqGroupNumber)">复制群号</button>
            <a v-if="contact.qqGroupInviteUrl" class="btn" :href="contact.qqGroupInviteUrl" target="_blank" rel="noopener noreferrer">打开加群链接</a>
          </template>
          <a v-if="contact[channel.qr] && !failedImages.has(channel.key)" class="btn" :href="contactQrUrl(channel.key)" download :aria-label="`保存${channel.title}二维码`">保存二维码</a>
        </div>
      </section>
    </div>
  </section>
</template>

<style scoped>
.contact-page {
  max-width: 1060px;
  margin-inline: auto;
  padding: clamp(28px, 6vw, 64px) 4px 32px;
}

.contact-heading { margin-bottom: 24px; }
.contact-heading h1 { margin: 0; font-size: clamp(27px, 4vw, 38px); }
.mobile-tip { display: none; color: var(--ink-2); margin: 8px 0 0; font-size: 14px; }
.contact-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 16px; }

.contact-card {
  min-width: 0;
  display: flex;
  flex-direction: column;
  padding: clamp(16px, 2vw, 24px);
  box-shadow: none;
}

.channel-heading { margin-bottom: 16px; min-height: 84px; }
.channel-heading h2 { margin: 0; font-size: 18px; }
.channel-name { margin: 6px 0 0; font-weight: 600; }
.group-number { margin: 0; color: var(--ink-2); font-size: 14px; }
.channel-name, .group-number { overflow-wrap: anywhere; }
.group-number b { font-variant-numeric: tabular-nums; }

.channel-qr, .channel-status {
  flex: 1;
  margin: 0;
  padding: 16px 12px;
  border-radius: 12px;
  background: var(--surface-2);
  color: var(--ink-2);
  font-size: 14px;
  text-align: center;
}

.channel-qr { display: grid; justify-items: center; grid-template-rows: auto 42px; align-content: center; gap: 8px; }
.channel-qr img { display: block; width: min(100%, 180px); height: auto; aspect-ratio: 1; object-fit: contain; background: #fff; }
.channel-qr time { display: block; font-size: 13px; }
.channel-status { display: grid; place-items: center; min-height: 96px; }
.channel-actions { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 14px; }
.channel-actions:empty { margin: 0; }
.channel-actions .btn { min-height: 44px; }

@media (max-width: 767px) {
  .contact-page { max-width: 460px; }
  .contact-grid { grid-template-columns: minmax(0, 1fr); gap: 12px; }
  .channel-heading { min-height: 0; }
  .mobile-tip { display: block; }
}
</style>
