<script setup lang="ts">
import { onMounted, ref } from "vue";
import { getDesktopConfig, saveDesktopConfig } from "@/admin/api";
import { type DesktopConfig, type Announcement } from "@/desktop";
const config = ref<DesktopConfig | null>(null),
  error = ref(""),
  message = ref(""),
  busy = ref(false);
async function load() {
  try {
    config.value = await getDesktopConfig();
    error.value = "";
  } catch (e) {
    error.value = (e as Error).message;
  }
}
onMounted(load);
function addNotice() {
  config.value?.announcements.unshift({
    id: crypto.randomUUID(),
    title: "",
    body: "",
    published: false,
    audience: ["desktop", "website"],
    startsAt: null,
    endsAt: null,
    skippable: true,
  });
}
function date(value: string | null) {
  if (!value) return "";
  const d = new Date(value);
  return new Date(d.getTime() - d.getTimezoneOffset() * 60000)
    .toISOString()
    .slice(0, 16);
}
function setDate(
  record: Announcement,
  field: "publishedAt" | "startsAt" | "endsAt",
  event: Event,
) {
  const value = (event.target as HTMLInputElement).value;
  (record as unknown as Record<string, unknown>)[field] = value
    ? new Date(value).toISOString()
    : null;
}
async function save() {
  if (!config.value || busy.value) return;
  busy.value = true;
  error.value = "";
  message.value = "";
  try {
    config.value = await saveDesktopConfig(config.value);
    message.value = "已保存，网站下载页与客户端接口立即使用已发布的数据。";
  } catch (e) {
    error.value = (e as Error).message;
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <div class="a-page">
    <header class="a-head">
      <h1>小伴桌面</h1>
      <p>
        管理官网介绍和网站 / 客户端公告。安装包与版本使用独立的 App 上架入口。
      </p>
    </header>
    <p v-if="error" class="error" role="alert">
      {{ error }}
      <button v-if="!config" class="a-btn" @click="load">重试</button>
    </p>
    <p v-if="message" class="success" role="status">{{ message }}</p>
    <form v-if="config" @submit.prevent="save">
      <fieldset :disabled="busy">
        <section class="a-section">
          <h2>官网信息</h2>
          <label
            >网站地址<input
              v-model="config.siteUrl"
              type="url"
              required
              placeholder="https://wallpaper.xiangban.xyz"
          /></label>
          <p class="a-small">
            正式地址使用 HTTPS；本机测试可填写 http://localhost:8080。
          </p>
          <label
            >产品介绍<textarea v-model="config.introduction" rows="3" />
          </label>
        </section>
        <section class="a-section">
          <h2>App 安装包与版本</h2>
          <p class="a-note">
            上传安装包、保存草稿、上架和下架版本，请进入 App 上架。
          </p>
          <RouterLink class="a-link" to="/admin/apps"
            >前往 App 上架 →</RouterLink
          >
        </section>
        <section class="a-section">
          <div class="a-section-head">
            <h2>公告</h2>
            <button type="button" class="a-btn" @click="addNotice">
              添加公告
            </button>
          </div>
          <p class="a-small">
            内容修改后，客户端会识别为新修订；下架或过期后不再展示。
          </p>
          <article
            v-for="(notice, index) in config.announcements"
            :key="notice.id"
            class="editor"
          >
            <label>标题<input v-model="notice.title" required /></label
            ><label>内容<textarea v-model="notice.body" rows="4" /></label>
            <div class="row">
              <label
                >开始时间（可选）<input
                  type="datetime-local"
                  :value="date(notice.startsAt)"
                  @change="setDate(notice, 'startsAt', $event)" /></label
              ><label
                >结束时间（可选）<input
                  type="datetime-local"
                  :value="date(notice.endsAt)"
                  @change="setDate(notice, 'endsAt', $event)"
              /></label>
            </div>
            <div class="row">
              <label class="toggle"
                ><input
                  v-model="notice.audience"
                  type="checkbox"
                  value="website"
                />网站展示</label
              ><label class="toggle"
                ><input
                  v-model="notice.audience"
                  type="checkbox"
                  value="desktop"
                />桌面客户端展示</label
              ><label class="toggle"
                ><input
                  v-model="notice.audience"
                  type="checkbox"
                  value="android"
                />Android 客户端弹窗</label
              >
            </div>
            <template v-if="notice.audience.includes('android')">
              <p class="a-small">
                Android 弹窗：可跳过的公告有取消按钮，每次修改后只弹一次；不可跳过的公告只有确定按钮，每次启动都会弹，直到下架。
              </p>
              <div class="row">
                <label class="toggle"
                  ><input v-model="notice.skippable" type="checkbox" />允许跳过</label
                >
              </div>
              <div class="row">
                <label
                  >确定按钮文字（可选，最多 12 字）<input
                    v-model="notice.confirmText"
                    maxlength="12"
                    placeholder="默认：确定" /></label
                ><label
                  >取消按钮文字（可选，仅可跳过时显示）<input
                    v-model="notice.cancelText"
                    maxlength="12"
                    :disabled="!notice.skippable"
                    placeholder="默认：取消"
                /></label>
              </div>
              <label
                >确定按钮跳转链接（可选，须为 HTTPS；留空则只关闭弹窗）<input
                  v-model="notice.confirmUrl"
                  type="url"
                  placeholder="https://wallpaper.xiangban.xyz/#desktop-downloads"
              /></label>
            </template>
            <div class="row">
              <label class="toggle"
                ><input
                  v-model="notice.published"
                  type="checkbox"
                />发布公告</label
              ><button
                type="button"
                class="a-btn"
                @click="config.announcements.splice(index, 1)"
              >
                删除公告
              </button>
            </div>
          </article>
        </section>
        <button class="a-btn primary" type="submit">
          {{ busy ? "正在保存…" : "保存配置" }}
        </button>
      </fieldset>
    </form>
    <p v-else-if="!error">正在加载…</p>
  </div>
</template>
<style scoped>
fieldset {
  border: 0;
  padding: 0;
  margin: 0;
  min-width: 0;
}
.editor {
  display: grid;
  gap: 16px;
  padding: 22px;
  border: 1px solid var(--line);
  border-radius: 14px;
}
.editor label,
.a-section > label {
  display: grid;
  gap: 8px;
  min-width: 0;
}
input:not([type="checkbox"]),
textarea,
select {
  width: 100%;
  padding: 10px 12px;
  background: var(--field);
  border: 1px solid var(--line);
  border-radius: 8px;
  color: var(--text);
  font: inherit;
}
textarea {
  resize: vertical;
}
.row {
  display: flex;
  gap: 16px;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
}
.row > label {
  flex: 1;
}
.row .toggle {
  display: flex;
  gap: 8px;
  align-items: center;
}
.package {
  padding: 16px;
  border-radius: 10px;
  background: var(--surface);
  display: grid;
  gap: 12px;
}
.package details label {
  margin-top: 12px;
}
.error {
  color: var(--danger);
  margin: 20px 0;
}
.success {
  color: var(--ok);
  margin: 20px 0;
}
summary {
  cursor: pointer;
  color: var(--text-3);
}
.primary {
  margin-bottom: 30px;
}
</style>
