<script setup lang="ts">
import { onMounted, ref } from "vue";
import { getDesktopConfig, saveDesktopConfig } from "@/admin/api";
import {
  targets,
  type DesktopConfig,
  type DesktopRelease,
  type Announcement,
} from "@/desktop";
const config = ref<DesktopConfig | null>(null),
  error = ref(""),
  message = ref(""),
  busy = ref(false);
const newTarget = ref("windows-x86_64");
async function load() {
  try {
    config.value = await getDesktopConfig();
    error.value = "";
  } catch (e) {
    error.value = (e as Error).message;
  }
}
onMounted(load);
function addRelease() {
  config.value?.releases.unshift({
    version: "",
    notes: "",
    publishedAt: new Date().toISOString(),
    published: false,
    platforms: { "windows-x86_64": { url: "" } },
  });
}
function addNotice() {
  config.value?.announcements.unshift({
    id: crypto.randomUUID(),
    title: "",
    body: "",
    published: false,
    audience: ["desktop", "website"],
    startsAt: null,
    endsAt: null,
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
  record: DesktopRelease | Announcement,
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
      <p>管理官网介绍、各平台下载版本和网站 / 客户端公告。</p>
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
          <div class="a-section-head">
            <h2>版本与下载</h2>
            <button type="button" class="a-btn" @click="addRelease">
              添加版本
            </button>
          </div>
          <p class="a-small">
            草稿不展示；未来的发布时间用于定时发布。每个平台独立选择已发布的最新稳定版本。
          </p>
          <article
            v-for="(release, index) in config.releases"
            :key="index"
            class="editor"
          >
            <div class="row">
              <label
                >版本号<input
                  v-model="release.version"
                  required
                  placeholder="1.0.0" /></label
              ><label
                >发布时间<input
                  type="datetime-local"
                  :value="date(release.publishedAt)"
                  required
                  @change="setDate(release, 'publishedAt', $event)"
              /></label>
            </div>
            <label>更新说明<textarea v-model="release.notes" rows="3" /></label>
            <div
              v-for="(pkg, target) in release.platforms"
              :key="target"
              class="package"
            >
              <div class="row">
                <b>{{ targets.find((t) => t.id === target)?.label }}</b
                ><button
                  type="button"
                  class="a-btn"
                  @click="delete release.platforms[target]"
                >
                  移除平台
                </button>
              </div>
              <label
                >下载链接<input
                  v-model="pkg.url"
                  type="url"
                  required
                  placeholder="https://…/XiaoBan.zip"
              /></label>
              <details>
                <summary>可选的包校验信息</summary>
                <label
                  >SHA-256<input
                    v-model="pkg.sha256"
                    placeholder="64 位小写十六进制"
                    @change="!pkg.sha256 && delete pkg.sha256" /></label
                ><label
                  >文件字节数<input
                    v-model.number="pkg.bytes"
                    type="number"
                    min="1"
                    @change="!pkg.bytes && delete pkg.bytes"
                /></label>
              </details>
            </div>
            <div class="row">
              <select v-model="newTarget">
                <option
                  v-for="target in targets"
                  :key="target.id"
                  :value="target.id"
                >
                  {{ target.label }}
                </option></select
              ><button
                type="button"
                class="a-btn"
                @click="release.platforms[newTarget] ??= { url: '' }"
              >
                添加平台
              </button>
            </div>
            <div class="row">
              <label class="toggle"
                ><input
                  v-model="release.published"
                  type="checkbox"
                />发布这个版本</label
              ><button
                type="button"
                class="a-btn"
                @click="config.releases.splice(index, 1)"
              >
                删除版本
              </button>
            </div>
          </article>
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
              >
            </div>
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
