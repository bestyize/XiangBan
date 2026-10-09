<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from "vue";
import {
  listAppReleases,
  saveAppRelease,
  deleteAppRelease,
  uploadAppPackage,
} from "@/admin/api";
import { targets, type DesktopRelease, type DesktopPackage } from "@/desktop";
let uploadController: AbortController | null = null;
onBeforeUnmount(() => uploadController?.abort());
function cancelUpload() {
  uploadController?.abort();
}
const releases = ref<DesktopRelease[]>([]),
  editor = ref<DesktopRelease | null>(null),
  editingVersion = ref(""),
  error = ref(""),
  message = ref(""),
  busy = ref(false),
  loading = ref(true);
const uploading = ref(""),
  progress = ref(0),
  uploadedNames = ref<Record<string, string>>({}),
  nextTarget = ref("darwin-aarch64"),
  form = ref<HTMLFormElement | null>(null);
const managed = (url: string) => url.startsWith("/api/v1/desktop/downloads/");
const size = (bytes?: number) =>
  !bytes
    ? ""
    : bytes < 1024
      ? `${bytes} B`
      : bytes < 1024 * 1024
        ? `${(bytes / 1024).toFixed(1)} KiB`
        : `${(bytes / 1024 / 1024).toFixed(2)} MiB`;
const label = (id: string) => targets.find((t) => t.id === id)?.label ?? id;
async function load() {
  loading.value = true;
  try {
    releases.value = await listAppReleases();
  } catch (e) {
    error.value = (e as Error).message;
  } finally {
    loading.value = false;
  }
}
onMounted(load);
function create() {
  editingVersion.value = "";
  uploadedNames.value = {};
  editor.value = {
    version: "",
    notes: "",
    publishedAt: new Date().toISOString(),
    published: false,
    platforms: { "windows-x86_64": { url: "" } },
  };
  error.value = "";
  message.value = "";
}
function edit(release: DesktopRelease) {
  editingVersion.value = release.version;
  uploadedNames.value = {};
  editor.value = JSON.parse(JSON.stringify(release));
  error.value = "";
  message.value = "";
}
function date(value: string) {
  if (!value) return "";
  const d = new Date(value);
  return new Date(d.getTime() - d.getTimezoneOffset() * 60000)
    .toISOString()
    .slice(0, 16);
}
function setDate(event: Event) {
  const value = (event.target as HTMLInputElement).value;
  if (editor.value)
    editor.value.publishedAt = value ? new Date(value).toISOString() : "";
}
function status(release: DesktopRelease) {
  return !release.published
    ? "未上架"
    : new Date(release.publishedAt) > new Date()
      ? "待发布"
      : "已上架";
}
function external(pkg: DesktopPackage, event: Event) {
  const url = (event.target as HTMLInputElement).value.trim();
  if (url !== pkg.url) {
    pkg.url = url;
    delete pkg.sha256;
    delete pkg.bytes;
  }
}
async function upload(target: string, event: Event) {
  const input = event.target as HTMLInputElement,
    file = input.files?.[0];
  if (!file || !editor.value) return;
  if (file.size > 512 * 1024 * 1024) {
    error.value = "安装包不能超过 512 MiB";
    input.value = "";
    return;
  }
  const draft = editor.value;
  uploadController = new AbortController();
  uploading.value = target;
  progress.value = 0;
  error.value = "";
  message.value = "";
  try {
    const result = await uploadAppPackage(
      target,
      file,
      (percent) => (progress.value = percent),
      uploadController.signal,
    );
    draft.platforms[target] = {
      url: result.url,
      sha256: result.sha256,
      bytes: result.bytes,
    };
    uploadedNames.value[target] = result.fileName;
    message.value = "安装包已上传，保存并上架这个版本后才开放下载。";
  } catch (e) {
    error.value = (e as Error).message;
  } finally {
    uploading.value = "";
    uploadController = null;
    input.value = "";
  }
}
async function persist(published: boolean) {
  if (
    !editor.value ||
    busy.value ||
    uploading.value ||
    !form.value?.reportValidity()
  )
    return;
  if (
    !Object.keys(editor.value.platforms).length ||
    Object.values(editor.value.platforms).some((pkg) => !pkg.url)
  ) {
    error.value = "请为每个平台上传安装包或填写下载链接";
    return;
  }
  editor.value.version = editor.value.version.trim();
  busy.value = true;
  error.value = "";
  message.value = "";
  try {
    const saved = await saveAppRelease({ ...editor.value, published });
    await load();
    editor.value = null;
    message.value = published
      ? new Date(saved.publishedAt) > new Date()
        ? "版本已保存，到发布时间自动上架。"
        : "App 已上架，官网和客户端更新检测已同步。"
      : "草稿已保存，安装包暂不对外下载。";
  } catch (e) {
    error.value = (e as Error).message;
  } finally {
    busy.value = false;
  }
}
async function toggle(release: DesktopRelease) {
  busy.value = true;
  error.value = "";
  message.value = "";
  try {
    await saveAppRelease({ ...release, published: !release.published });
    await load();
    message.value = release.published
      ? "版本已下架，站内安装包下载已关闭。"
      : new Date(release.publishedAt) > new Date()
        ? "版本已设置，等待发布时间上架。"
        : "版本已上架。";
  } catch (e) {
    error.value = (e as Error).message;
  } finally {
    busy.value = false;
  }
}
async function remove(release: DesktopRelease) {
  if (
    !window.confirm(
      `删除版本 ${release.version}？这个版本会从官网与更新检测中移除。`,
    )
  )
    return;
  busy.value = true;
  error.value = "";
  try {
    await deleteAppRelease(release.version);
    await load();
    message.value = "版本已删除。";
  } catch (e) {
    error.value = (e as Error).message;
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <div class="a-page app-page">
    <header class="a-head">
      <h1>App 上架</h1>
      <p>上传小伴桌面 Windows / macOS 安装包，管理版本、更新说明和上架状态。</p>
    </header>
    <p v-if="error" class="error" role="alert">{{ error }}</p>
    <p v-if="message" class="success" role="status">{{ message }}</p>
    <div class="app-actions">
      <button
        class="a-btn primary"
        :disabled="busy || !!uploading"
        @click="create"
      >
        新建版本</button
      ><RouterLink class="a-link" to="/admin/desktop"
        >官网介绍与公告设置</RouterLink
      >
    </div>
    <form
      v-if="editor"
      ref="form"
      class="release-editor"
      @submit.prevent="persist(false)"
    >
      <fieldset :disabled="busy || !!uploading">
        <h2>{{ editingVersion ? `编辑 v${editingVersion}` : "新版本上架" }}</h2>
        <div class="row">
          <label
            >版本号<input
              v-model="editor.version"
              placeholder="1.0.0"
              required
              :readonly="!!editingVersion" /></label
          ><label
            >发布时间<input
              type="datetime-local"
              :value="date(editor.publishedAt)"
              required
              @change="setDate($event)"
          /></label>
        </div>
        <label
          >更新说明<textarea
            v-model="editor.notes"
            rows="3"
            placeholder="告诉用户这次更新了什么"
          />
        </label>
        <article
          v-for="(pkg, target) in editor.platforms"
          :key="target"
          class="platform"
        >
          <div class="row">
            <h3>{{ label(target) }}</h3>
            <button
              class="a-btn"
              type="button"
              @click="delete editor.platforms[target]"
            >
              移除平台
            </button>
          </div>
          <label class="upload-control"
            >{{ managed(pkg.url) ? "更换安装包" : "上传安装包"
            }}<input
              type="file"
              :accept="
                target === 'windows-x86_64'
                  ? '.exe,.msi,.zip'
                  : '.dmg,.pkg,.zip'
              "
              @change="upload(target, $event)"
          /></label>
          <div v-if="managed(pkg.url)" class="uploaded">
            <b>✓ 服务器安装包已就绪</b>
            <p>
              {{ uploadedNames[target] || "已上传安装包" }} ·
              {{ size(pkg.bytes) }}
            </p>
            <small>SHA-256</small><code>{{ pkg.sha256 }}</code>
          </div>
          <details>
            <summary>或使用外部下载链接</summary>
            <label
              >下载链接<input
                type="url"
                :value="managed(pkg.url) ? '' : pkg.url"
                placeholder="https://…/XiaoBan.zip"
                @input="external(pkg, $event)"
            /></label>
            <p class="a-small">
              站内上传会自动生成校验信息；外部链接使用你填写的地址。
            </p>
          </details>
        </article>
        <div class="row">
          <select v-model="nextTarget" aria-label="添加平台">
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
            @click="editor.platforms[nextTarget] ??= { url: '' }"
          >
            添加平台
          </button>
        </div>
        <p class="a-small">
          每个平台上传一个安装包，最多 512
          MiB。草稿不公开，未来的发布时间用于定时上架。
        </p>
        <div class="app-actions">
          <button class="a-btn primary" type="button" @click="persist(true)">
            保存并上架</button
          ><button class="a-btn" type="submit">{{ editor.published ? "保存并下架" : "保存草稿" }}</button
          ><button class="a-btn" type="button" @click="editor = null">
            取消
          </button>
        </div>
      </fieldset>
      <div v-if="uploading" class="upload-progress" role="status">
        <span>{{ label(uploading) }} · 上传中 {{ progress }}%</span
        ><progress :value="progress" max="100"></progress
        ><button type="button" class="a-btn" @click="cancelUpload">
          取消上传
        </button>
      </div>
    </form>
    <section class="release-list">
      <p v-if="loading" role="status">正在加载版本…</p>
      <p v-else-if="!releases.length" class="empty">
        还没有 App 版本。新建版本并上传安装包，就可以上架。
      </p>
      <article
        v-for="release in releases"
        :key="release.version"
        class="release-card"
      >
        <div class="row">
          <h2>v{{ release.version }}</h2>
          <span class="status" :class="{ published: release.published }">{{
            status(release)
          }}</span>
        </div>
        <p class="a-small">
          {{ new Date(release.publishedAt).toLocaleString("zh-CN") }} ·
          {{
            release.version.split("+")[0]?.includes("-")
              ? "预发布版本"
              : "稳定版本"
          }}
        </p>
        <div
          v-for="(pkg, target) in release.platforms"
          :key="target"
          class="package-row"
        >
          <b>{{ label(target) }}</b
          ><span
            >{{ managed(pkg.url) ? "站内安装包" : "外部下载链接" }}
            {{ size(pkg.bytes) }}</span
          >
        </div>
        <pre>{{ release.notes || "暂无更新说明" }}</pre>
        <div class="app-actions">
          <button
            class="a-btn"
            :disabled="busy || !!uploading"
            @click="edit(release)"
          >
            编辑</button
          ><button
            class="a-btn"
            :disabled="busy || !!uploading"
            @click="toggle(release)"
          >
            {{ release.published ? "下架" : "上架" }}</button
          ><button
            class="a-btn danger"
            :disabled="busy || !!uploading"
            @click="remove(release)"
          >
            删除版本
          </button>
        </div>
      </article>
    </section>
  </div>
</template>
<style scoped>
.app-page {
  max-width: 960px;
}
.app-actions {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
  margin: 22px 0;
}
.release-editor,
.release-card {
  border: 1px solid var(--line);
  padding: 24px;
  border-radius: 14px;
  margin: 20px 0;
  background: var(--surface);
}
fieldset {
  border: 0;
  padding: 0;
  min-width: 0;
  display: grid;
  gap: 20px;
}
h2 {
  font-size: 25px;
}
h3 {
  font-size: 16px;
}
label {
  display: grid;
  gap: 8px;
  min-width: 0;
}
input:not([type="file"]),
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
input[readonly] {
  color: var(--text-3);
}
textarea {
  resize: vertical;
}
.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
}
.row > label {
  flex: 1;
}
.platform {
  display: grid;
  gap: 16px;
  border: 1px solid var(--line);
  border-radius: 10px;
  padding: 18px;
}
.upload-control {
  padding: 12px 14px;
  border: 1px dashed var(--line-strong);
  border-radius: 10px;
  cursor: pointer;
  color: var(--text-2);
}
.upload-control input {
  font: inherit;
  font-size: 12px;
  max-width: 100%;
  color: var(--text-3);
}
.uploaded {
  padding: 14px;
  background: #8fd9a810;
  border: 1px solid #8fd9a830;
  border-radius: 8px;
}
.uploaded > b,
.success {
  color: var(--ok);
}
.uploaded p {
  font-size: 12px;
  margin: 6px 0;
}
.uploaded small {
  font-size: 10px;
  color: var(--text-3);
}
code {
  display: block;
  overflow-wrap: anywhere;
  font-size: 10px;
  color: var(--text-3);
  margin-top: 4px;
}
.error {
  color: var(--danger);
  margin: 16px 0;
}
.success {
  margin: 16px 0;
}
.package-row {
  display: flex;
  justify-content: space-between;
  gap: 14px;
  font-size: 12px;
  padding: 12px 0;
  border-bottom: 1px solid var(--line);
}
.package-row > span {
  color: var(--text-3);
}
.status {
  font-size: 11px;
  border: 1px solid var(--line);
  border-radius: 20px;
  padding: 4px 10px;
  color: var(--text-3);
}
.status.published {
  color: var(--ok);
  border-color: #8fd9a850;
}
pre {
  white-space: pre-wrap;
  font-family: inherit;
  font-size: 13px;
  color: var(--text-2);
  margin: 18px 0;
}
.danger {
  color: var(--danger);
}
.empty {
  padding: 36px 0;
  color: var(--text-3);
}
summary {
  cursor: pointer;
  color: var(--text-3);
  font-size: 12px;
}
details label {
  margin: 12px 0;
}
.upload-progress {
  display: grid;
  gap: 10px;
  padding-top: 16px;
  color: var(--text-2);
  font-size: 12px;
}
.upload-progress progress {
  width: 100%;
}
</style>
