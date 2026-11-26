<script setup lang="ts">
import { onMounted, ref } from 'vue'
import Icon from '@/components/Icon.vue'
import AdminMoreMenu, { type MoreMenuItem } from '@/components/admin/AdminMoreMenu.vue'
import ToggleSwitch from '@/components/admin/ToggleSwitch.vue'
import { getDesktopConfig, saveDesktopConfig } from '@/admin/api'
import { toast } from '@/composables/toast'
import type { Announcement, DesktopConfig } from '@/desktop'

const config = ref<DesktopConfig | null>(null)
const error = ref('')
const busy = ref(false)
const dirtySite = ref(false)

// Split Announcement Drawer
const noticeDraft = ref<Announcement | null>(null)
const editingNoticeIdx = ref<number | null>(null)

const AUD_LABEL: Record<string, string> = {
  website: '网站',
  desktop: '桌面',
  android: 'Android',
}

async function load() {
  try {
    config.value = await getDesktopConfig()
    dirtySite.value = false
    error.value = ''
  } catch (e) {
    error.value = (e as Error).message
  }
}

onMounted(load)

function dateInput(value: string | null) {
  if (!value) return ''
  const d = new Date(value)
  return new Date(d.getTime() - d.getTimezoneOffset() * 60000).toISOString().slice(0, 16)
}

function setNoticeDate(field: 'startsAt' | 'endsAt', event: Event) {
  if (!noticeDraft.value) return
  const value = (event.target as HTMLInputElement).value
  noticeDraft.value[field] = value ? new Date(value).toISOString() : null
}

async function persistAll(msg: string) {
  if (!config.value || busy.value) return false
  busy.value = true
  error.value = ''
  try {
    config.value = await saveDesktopConfig(config.value)
    dirtySite.value = false
    toast(msg)
    return true
  } catch (e) {
    error.value = (e as Error).message
    return false
  } finally {
    busy.value = false
  }
}

function openNoticeDrawer(idx: number | null = null) {
  editingNoticeIdx.value = idx
  if (idx !== null && config.value) {
    noticeDraft.value = JSON.parse(JSON.stringify(config.value.announcements[idx]))
  } else {
    noticeDraft.value = {
      id: crypto.randomUUID(),
      title: '',
      body: '',
      published: false,
      audience: ['website', 'desktop'],
      startsAt: null,
      endsAt: null,
      skippable: true,
      confirmText: '',
      cancelText: '',
      confirmUrl: '',
    }
  }
}

function toggleAudience(aud: string) {
  if (!noticeDraft.value) return
  const idx = noticeDraft.value.audience.indexOf(aud)
  if (idx >= 0) noticeDraft.value.audience.splice(idx, 1)
  else noticeDraft.value.audience.push(aud)
}

async function saveNoticeDrawer() {
  if (!config.value || !noticeDraft.value) return
  if (!noticeDraft.value.title.trim()) {
    toast('请输入公告标题')
    return
  }
  const prev = JSON.parse(JSON.stringify(config.value.announcements))
  if (editingNoticeIdx.value !== null) {
    config.value.announcements[editingNoticeIdx.value] = noticeDraft.value
  } else {
    config.value.announcements.unshift(noticeDraft.value)
  }
  const ok = await persistAll('公告已保存并同步')
  if (ok) noticeDraft.value = null
  else config.value.announcements = prev
}

async function toggleNoticePublish(idx: number, val: boolean) {
  if (!config.value) return
  const old = config.value.announcements[idx].published
  config.value.announcements[idx].published = val
  const ok = await persistAll(val ? '公告已发布' : '公告已下架')
  if (!ok) config.value.announcements[idx].published = old
}

async function deleteNotice(idx: number) {
  if (!config.value) return
  const prev = [...config.value.announcements]
  config.value.announcements.splice(idx, 1)
  const ok = await persistAll('公告已删除')
  if (!ok) config.value.announcements = prev
}

function menuForNotice(idx: number): MoreMenuItem[] {
  return [
    { label: '编辑与真机预览', icon: 'phone', action: () => openNoticeDrawer(idx) },
    { sep: true },
    {
      label: '删除公告',
      icon: 'trash',
      danger: true,
      confirmText: '确认删除？',
      action: () => void deleteNotice(idx),
    },
  ]
}

function fmtShortDate(iso: string | null) {
  if (!iso) return ''
  return new Date(iso).toLocaleDateString('zh-CN')
}
</script>

<template>
  <div class="xb-page is-narrow">
    <header class="xb-page-head">
      <h1 class="xb-page-title">官网与公告</h1>
      <div class="xb-page-actions">
        <RouterLink to="/admin/apps" class="xb-btn-ghost">App 版本</RouterLink>
        <button type="button" class="xb-btn-primary" :disabled="!config || busy" @click="openNoticeDrawer()">
          <Icon name="plus" :size="14" /><span>新建公告</span>
        </button>
      </div>
    </header>

    <div v-if="error" class="xb-alert is-danger" role="alert">
      <div class="xb-alert-l"><Icon name="alert" :size="15" /><span>{{ error }}</span></div>
      <button v-if="!config" type="button" class="xb-btn-ghost is-sm" @click="load">重试</button>
    </div>

    <div v-if="!config && !error" class="xb-group xb-empty-box">正在加载…</div>

    <template v-if="config">
      <section>
        <div class="xb-sec-head">
          <h2 class="xb-sec-title">官网基础信息</h2>
          <button
            type="button"
            class="xb-btn-primary is-sm"
            :disabled="busy || !dirtySite"
            @click="persistAll('官网基础信息已保存')"
          >
            {{ busy ? '保存中…' : '保存' }}
          </button>
        </div>
        <div class="xb-group xb-group-pad form-col">
          <label class="xb-field" title="正式环境使用 HTTPS，本机调试可填 http://localhost:8080">
            <span class="xb-field-label">网站地址</span>
            <input
              v-model="config.siteUrl"
              type="url"
              class="xb-input xb-mono"
              required
              placeholder="https://wallpaper.xiangban.xyz"
              @input="dirtySite = true"
            />
          </label>
          <label class="xb-field">
            <span class="xb-field-label">产品介绍</span>
            <textarea
              v-model="config.introduction"
              class="xb-textarea"
              rows="3"
              @input="dirtySite = true"
            />
          </label>
        </div>
      </section>

      <section>
        <div class="xb-sec-head">
          <h2 class="xb-sec-title">
            多端公告 <span class="xb-pill">{{ config.announcements.length }}</span>
          </h2>
        </div>

        <div class="xb-group">
          <div v-if="!config.announcements.length" class="xb-empty-box" style="padding: 32px">
            <span>暂无公告</span>
          </div>
          <div v-for="(notice, idx) in config.announcements" :key="notice.id" class="xb-row" :title="notice.body">
            <div style="flex: 1; min-width: 200px">
              <div class="xb-row-title">
                <span>{{ notice.title || '未命名公告' }}</span>
                <span class="xb-pill" :class="notice.published ? 'is-ok' : 'is-warn'">
                  <i class="dot" />{{ notice.published ? '生效中' : '草稿' }}
                </span>
                <span v-if="notice.audience.includes('android') && !notice.skippable" class="xb-pill is-danger">
                  强弹
                </span>
              </div>
              <div class="xb-row-meta">
                <span>{{ notice.audience.map((a) => AUD_LABEL[a] || a).join(' · ') || '未指定端' }}</span>
                <span>·</span>
                <span>{{ fmtShortDate(notice.startsAt) || '立即' }} ~ {{ fmtShortDate(notice.endsAt) || '长期' }}</span>
              </div>
            </div>
            <div class="xb-row-actions">
              <ToggleSwitch
                :model-value="notice.published"
                :label="notice.published ? '下架公告' : '发布公告'"
                :disabled="busy"
                @update:model-value="toggleNoticePublish(idx, $event)"
              />
              <button type="button" class="xb-btn-ghost is-sm" @click="openNoticeDrawer(idx)">编辑</button>
              <AdminMoreMenu :items="menuForNotice(idx)" :disabled="busy" />
            </div>
          </div>
        </div>
      </section>
    </template>

    <!-- 20px Split Announcement Drawer with Live Android Popup Preview -->
    <Teleport to="body">
      <div v-if="noticeDraft" class="xb-backdrop" @click.self="noticeDraft = null">
        <form class="xb-drawer is-wide" @submit.prevent="saveNoticeDrawer">
          <header class="xb-ov-head">
            <h2>{{ editingNoticeIdx !== null ? '编辑公告' : '新建公告' }}</h2>
            <button type="button" class="xb-icon-btn" aria-label="关闭" @click="noticeDraft = null">
              <Icon name="close" :size="18" />
            </button>
          </header>

          <div class="xb-ov-body">
            <div class="split-preview">
              <div class="form-col">
                <label class="xb-field">
                  <span class="xb-field-label">标题</span>
                  <input v-model="noticeDraft.title" class="xb-input" required />
                </label>

                <label class="xb-field">
                  <span class="xb-field-label">正文</span>
                  <textarea v-model="noticeDraft.body" class="xb-textarea" rows="4" />
                </label>

                <div class="xb-split-eq">
                  <label class="xb-field">
                    <span class="xb-field-label">开始时间</span>
                    <input
                      type="datetime-local"
                      class="xb-input xb-num"
                      :value="dateInput(noticeDraft.startsAt)"
                      @change="setNoticeDate('startsAt', $event)"
                    />
                  </label>
                  <label class="xb-field">
                    <span class="xb-field-label">结束时间</span>
                    <input
                      type="datetime-local"
                      class="xb-input xb-num"
                      :value="dateInput(noticeDraft.endsAt)"
                      @change="setNoticeDate('endsAt', $event)"
                    />
                  </label>
                </div>

                <div class="xb-field">
                  <span class="xb-field-label">投放终端</span>
                  <div class="aud-row">
                    <button
                      v-for="aud in [
                        { v: 'website', l: '网站' },
                        { v: 'desktop', l: '桌面端' },
                        { v: 'android', l: 'Android 弹窗' },
                      ]"
                      :key="aud.v"
                      type="button"
                      class="is-sm"
                      :class="noticeDraft.audience.includes(aud.v) ? 'xb-btn-primary' : 'xb-btn-soft'"
                      @click="toggleAudience(aud.v)"
                    >
                      {{ aud.l }}
                    </button>
                  </div>
                </div>

                <div v-if="noticeDraft.audience.includes('android')" class="xb-group xb-group-pad form-col" style="padding: 12px 14px">
                  <div class="and-row" title="不可跳过时无取消按钮，每次启动均弹出直到下架">
                    <span style="font-size: 12.5px; font-weight: 550">允许跳过弹窗</span>
                    <ToggleSwitch v-model="noticeDraft.skippable" label="允许跳过弹窗" />
                  </div>
                  <div class="xb-split-eq">
                    <input
                      v-model="noticeDraft.confirmText"
                      class="xb-input"
                      maxlength="12"
                      placeholder="确定文案（默认：确定）"
                    />
                    <input
                      v-model="noticeDraft.cancelText"
                      class="xb-input"
                      maxlength="12"
                      :disabled="!noticeDraft.skippable"
                      placeholder="取消文案（默认：取消）"
                    />
                  </div>
                  <input
                    v-model="noticeDraft.confirmUrl"
                    type="url"
                    class="xb-input xb-mono"
                    placeholder="HTTPS 跳转链接（可选）"
                  />
                </div>
              </div>

              <div class="phone-col">
                <span class="xb-pill">Android 弹窗预览</span>
                <div class="phone-mock">
                  <div class="phone-screen">
                    <div class="phone-dim" />
                    <div class="android-dlg">
                      <h4>{{ noticeDraft.title || '公告标题' }}</h4>
                      <p>{{ noticeDraft.body || '输入公告内容实时预览手机弹窗效果' }}</p>
                      <div class="android-btns">
                        <span v-if="noticeDraft.skippable" class="c">{{ noticeDraft.cancelText || '取消' }}</span>
                        <span class="o">{{ noticeDraft.confirmText || '确定' }}</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <footer class="xb-ov-foot" style="justify-content: space-between">
            <div class="pub-toggle">
              <ToggleSwitch v-model="noticeDraft.published" label="立即发布" />
              <span>立即发布</span>
            </div>
            <div style="display: flex; gap: 8px">
              <button type="button" class="xb-btn-ghost" @click="noticeDraft = null">取消</button>
              <button type="submit" class="xb-btn-primary" :disabled="busy">
                {{ busy ? '保存中…' : '保存公告' }}
              </button>
            </div>
          </footer>
        </form>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.form-col {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.aud-row {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.and-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.split-preview {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(248px, 0.9fr);
  gap: 18px;
  align-items: start;
}
.phone-col {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}
.phone-mock {
  width: 252px;
  border-radius: 28px;
  background: #18181B;
  padding: 9px;
}
.phone-screen {
  border-radius: 20px;
  background: var(--ui-tint);
  height: 392px;
  position: relative;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 14px;
}
.phone-dim {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.42);
}
.android-dlg {
  position: relative;
  z-index: 2;
  width: 100%;
  border-radius: 22px;
  background: var(--ui-surface);
  padding: 18px 16px 14px;
  box-shadow: var(--ui-shadow-dialog);
  display: flex;
  flex-direction: column;
  gap: 9px;
}
.android-dlg h4 {
  margin: 0;
  font-size: 14.5px;
  font-weight: 650;
  line-height: 1.3;
}
.android-dlg p {
  margin: 0;
  font-size: 12.5px;
  color: var(--ui-text-secondary);
  line-height: 1.55;
  white-space: pre-wrap;
  max-height: 132px;
  overflow-y: auto;
}
.android-btns {
  display: flex;
  gap: 8px;
  margin-top: 6px;
}
.android-btns span {
  flex: 1;
  height: 34px;
  border-radius: 9px;
  font-size: 12.5px;
  font-weight: 600;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.android-btns .c {
  background: var(--ui-control);
  color: var(--ui-text-secondary);
}
.android-btns .o {
  background: var(--ui-btn);
  color: #fff;
}
.pub-toggle {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

@media (max-width: 720px) {
  .split-preview { grid-template-columns: 1fr; }
}
</style>
