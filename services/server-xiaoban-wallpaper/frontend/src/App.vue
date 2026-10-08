<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watchEffect } from "vue";
import {
  Cat,
  ArrowRight,
  ArrowUpRight,
  Download,
  Image as ImageIcon,
  Film,
  Sparkles,
  Monitor,
  Check,
  Plus,
  Search,
  LogOut,
  Settings,
  FolderHeart,
  Package,
  Lock,
  Upload,
  FileJson,
  X,
  Play,
  Copy,
  ShieldCheck,
  CloudSun,
  Heart,
  LoaderCircle,
  ExternalLink,
  RefreshCw,
  Trash2,
  Pencil,
} from "lucide-vue-next";
import type {
  Resource,
  Wallpaper,
  Entry,
  ReleaseState,
  Site,
  Session,
} from "./types";

const adminRoute = ref(location.pathname.startsWith("/admin")),
  session = ref<Session | null>(null),
  initializing = ref(true),
  busy = ref(false),
  toast = ref(""),
  error = ref("");
const site = ref<Site | null>(null),
  siteError = ref(false),
  section = ref("wallpapers"),
  entries = ref<Entry[]>([]),
  query = ref(""),
  filter = ref("all"),
  publicFilter = ref("all");
const loginUsername = ref("admin"),
  loginPassword = ref(""),
  loginError = ref(""),
  account = ref({ username: "", currentPassword: "", newPassword: "" });
const release = ref<ReleaseState | null>(null),
  releaseText = ref(""),
  editor = ref<Entry | null>(null),
  newEntry = ref(false),
  editorTab = ref("form"),
  rawDefinition = ref("");
const importOpen = ref(false),
  importText = ref(""),
  overwrite = ref(false),
  deleteEntry = ref<Entry | null>(null),
  heroHappy = ref(false);
let toastTimer: ReturnType<typeof setTimeout>;
const kinds: Record<string, string> = {
  image: "图片",
  video: "视频",
  canvas: "Canvas 动画",
};
watchEffect(() => {
  const names: Record<string, string> = {
    wallpapers: "壁纸资源",
    release: "产品发布",
    account: "账号设置",
  };
  document.title = adminRoute.value
    ? `${session.value ? names[section.value] : "管理员登录"} · 小伴桌面管理中心`
    : "小伴桌面 · 让桌面多一点温暖";
});
const publicItems = computed(
  () =>
    site.value?.catalog.items.filter(
      (w) => publicFilter.value === "all" || w.kind === publicFilter.value,
    ) ?? [],
);
const visibleEntries = computed(() =>
  entries.value.filter(
    (e) =>
      (filter.value === "all" ||
        filter.value === (e.published ? "published" : "draft") ||
        filter.value === e.definition.kind) &&
      `${e.definition.title} ${e.id} ${e.definition.author}`
        .toLowerCase()
        .includes(query.value.toLowerCase()),
  ),
);
const publishedCount = computed(
  () => entries.value.filter((e) => e.published).length,
);
const downloadPlatforms = [
  { id: "windows-x86_64", name: "Windows", detail: "Windows 10 / 11 · 64 位" },
  { id: "darwin-aarch64", name: "macOS", detail: "macOS 12+ · Apple 芯片" },
];
const cover = (w: Wallpaper) =>
  w.cover?.url || (w.kind === "image" ? w.resource.url : "");
const size = (n: number) =>
  n ? `${(n / 1024 / 1024).toFixed(2)} MB` : "待探测";
const clone = <T,>(v: T): T => JSON.parse(JSON.stringify(v));
const notify = (text: string) => {
  toast.value = text;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (toast.value = ""), 5000);
};
async function api<T>(
  path: string,
  method = "GET",
  body?: unknown,
): Promise<T> {
  const res = await fetch(`/api${path}`, {
    method,
    credentials: "same-origin",
    headers: {
      ...(body === undefined ? {} : { "Content-Type": "application/json" }),
      ...(method === "GET"
        ? {}
        : { "X-CSRF-Token": session.value?.csrfToken ?? "" }),
    },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (res.status === 204) return null as T;
  const data = await res
    .json()
    .catch(() => ({ message: "请求未完成，请稍后重试" }));
  if (!res.ok) {
    if (res.status === 401 && path !== "/admin/login") session.value = null;
    throw new Error(data.message ?? "请求失败");
  }
  return data;
}
async function task(fn: () => Promise<void>) {
  if (busy.value) return;
  busy.value = true;
  error.value = "";
  try {
    await fn();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}
async function load() {
  entries.value = (await api<{ items: Entry[] }>("/admin/wallpapers")).items;
}
async function loadSite() {
  try {
    site.value = await api<Site>("/site");
    siteError.value = false;
  } catch {
    siteError.value = true;
  }
}
async function signIn() {
  await task(async () => {
    loginError.value = "";
    try {
      await api("/admin/login", "POST", {
        username: loginUsername.value,
        password: loginPassword.value,
      });
      session.value = await api<Session>("/admin/me");
      loginPassword.value = "";
      account.value.username = session.value.username;
      await load();
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      if (session.value) error.value = message;
      else loginError.value = message;
    }
  });
}
async function signOut() {
  await task(async () => {
    await api("/admin/logout", "POST");
    session.value = null;
    editor.value = null;
    loginPassword.value = "";
  });
}
async function selectSection(next: string) {
  section.value = next;
  error.value = "";
  if (next === "release")
    await task(async () => {
      release.value = await api<ReleaseState>("/admin/release");
      releaseText.value = JSON.stringify(release.value.definition, null, 2);
    });
}
function edit(entry?: Entry) {
  newEntry.value = !entry;
  editorTab.value = "form";
  error.value = "";
  editor.value = entry
    ? clone(entry)
    : {
        id: "",
        definition: {
          id: `wall-${crypto.randomUUID()}`,
          version: "1.0.0",
          title: "",
          description: "",
          author: "小伴",
          kind: "image",
          resource: { url: "", sha256: "", bytes: 0, mime: "image/jpeg" },
          assets: {},
          controls: [],
        },
        published: false,
        sortOrder: 0,
        revision: 0,
        verifiedAt: null,
        updatedAt: "",
      };
  rawDefinition.value = JSON.stringify(editor.value.definition, null, 2);
}
function changeKind() {
  if (editor.value)
    editor.value.definition.resource.mime = {
      image: "image/jpeg",
      video: "video/mp4",
      canvas: "application/javascript",
    }[editor.value.definition.kind];
}
function changeEditorTab(next: string) {
  if (next === "json" && editor.value)
    rawDefinition.value = JSON.stringify(editor.value.definition, null, 2);
  editorTab.value = next;
}
async function save() {
  await task(async () => {
    if (!editor.value) return;
    const entry = editor.value;
    const definition =
      editorTab.value === "json"
        ? JSON.parse(rawDefinition.value)
        : entry.definition;
    await api(
      newEntry.value
        ? "/admin/wallpapers"
        : `/admin/wallpapers/${encodeURIComponent(entry.id)}`,
      newEntry.value ? "POST" : "PUT",
      {
        definition,
        published: entry.published,
        sortOrder: entry.sortOrder,
        ...(newEntry.value ? {} : { revision: entry.revision }),
      },
    );
    editor.value = null;
    await load();
    await loadSite();
    notify("壁纸元数据已保存");
  });
}
async function togglePublish(e: Entry) {
  await task(async () => {
    await api(`/admin/wallpapers/${encodeURIComponent(e.id)}/publish`, "POST", {
      published: !e.published,
      revision: e.revision,
    });
    await load();
    await loadSite();
    notify(e.published ? "壁纸已下架" : "壁纸已发布，桌面订阅可见");
  });
}
async function verify(e: Entry) {
  await task(async () => {
    await api(`/admin/wallpapers/${encodeURIComponent(e.id)}/verify`, "POST", {
      revision: e.revision,
    });
    await load();
    await loadSite();
    notify("大小与 SHA-256 已更新，服务器未保存媒体文件");
  });
}
async function remove() {
  await task(async () => {
    if (!deleteEntry.value) return;
    await api(
      `/admin/wallpapers/${encodeURIComponent(deleteEntry.value.id)}`,
      "DELETE",
      { revision: deleteEntry.value.revision },
    );
    deleteEntry.value = null;
    await load();
    await loadSite();
    notify("壁纸元数据已删除");
  });
}
async function importJson() {
  await task(async () => {
    const result = await api<{ imported: number }>(
      "/admin/wallpapers/import",
      "POST",
      { data: JSON.parse(importText.value), overwrite: overwrite.value },
    );
    importOpen.value = false;
    await load();
    await loadSite();
    notify(`已导入 ${result.imported} 张壁纸，保存为草稿`);
  });
}
async function readFile(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (file) {
    if (file.size > 2 * 1024 * 1024) {
      error.value = "JSON 文件不能超过 2 MB";
    } else importText.value = await file.text();
  }
  input.value = "";
}
async function copyCatalog() {
  try {
    await navigator.clipboard.writeText(`${location.origin}/api/catalog`);
    notify("壁纸订阅地址已复制");
  } catch {
    notify(`订阅地址：${location.origin}/api/catalog`);
  }
}
function exportJson() {
  const blob = new Blob(
    [
      JSON.stringify(
        {
          schemaVersion: 1,
          revision: new Date().toISOString(),
          items: entries.value.map((e) => e.definition),
        },
        null,
        2,
      ),
    ],
    { type: "application/json" },
  );
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = "xiaoban-wallpapers.json";
  a.click();
  setTimeout(() => URL.revokeObjectURL(a.href), 1000);
}
async function saveRelease() {
  await task(async () => {
    if (!release.value) return;
    release.value = await api<ReleaseState>("/admin/release", "PUT", {
      definition: JSON.parse(releaseText.value),
      revision: release.value.revision,
    });
    releaseText.value = JSON.stringify(release.value.definition, null, 2);
    notify("发布草稿已保存");
  });
}
async function publishRelease() {
  await task(async () => {
    if (!release.value) return;
    release.value = await api<ReleaseState>("/admin/release/publish", "POST", {
      revision: release.value.revision,
    });
    releaseText.value = JSON.stringify(release.value.definition, null, 2);
    await loadSite();
    notify("应用版本已发布");
  });
}
function addPlatform(id: string) {
  if (!release.value) return;
  try {
    const value = JSON.parse(releaseText.value);
    value.platforms ??= {};
    value.platforms[id] ??= {
      url: "",
      sha256: "",
      bytes: 0,
      mime: "application/zip",
    };
    releaseText.value = JSON.stringify(value, null, 2);
  } catch {
    error.value = "请先修正 JSON 格式";
  }
}
async function changeAccount() {
  await task(async () => {
    if (account.value.newPassword && account.value.newPassword.length < 6)
      throw new Error("新密码至少需要 6 个字符");
    await api("/admin/account", "PUT", account.value);
    loginUsername.value = account.value.username;
    loginPassword.value = "";
    account.value.currentPassword = "";
    account.value.newPassword = "";
    session.value = null;
    notify("账号已更新，请使用新账号重新登录");
  });
}
function route() {
  adminRoute.value = location.pathname.startsWith("/admin");
}
onMounted(async () => {
  window.addEventListener("popstate", route);
  await loadSite();
  if (adminRoute.value) {
    try {
      session.value = await api<Session>("/admin/me");
      account.value.username = session.value.username;
      await load();
    } catch {}
  }
  initializing.value = false;
});
onUnmounted(() => {
  clearTimeout(toastTimer);
  window.removeEventListener("popstate", route);
});
</script>

<template>
  <div v-if="!adminRoute" class="website">
    <nav class="site-nav wrap">
      <a class="brand" href="/"
        ><span class="brand-mark"><Cat :size="25" /></span
        ><span>小伴桌面</span></a
      >
      <div class="site-links">
        <a href="#features">功能</a><a href="#wallpapers">壁纸</a
        ><a href="#download">下载</a>
      </div>
      <a class="button small primary" href="#download"
        >让小伴住进桌面 <ArrowUpRight :size="16"
      /></a>
    </nav>
    <main>
      <section class="hero wrap">
        <div class="hero-copy">
          <span class="pill"><span class="dot" /> 陪伴，就在桌面上</span>
          <h1>让桌面，<br />多一点<span>温暖。</span></h1>
          <p>
            一只会回应的小伴，一幅喜欢的风景。<br />安静陪你工作，也陪你慢下来。
          </p>
          <div class="hero-actions">
            <a class="button primary" href="#download"
              ><Download :size="18" /> 下载小伴桌面</a
            ><a class="text-link" href="#wallpapers"
              >看看桌面里的风景 <ArrowRight :size="17"
            /></a>
          </div>
          <div class="platform-line">
            <Monitor :size="15" /> Windows <span>·</span> macOS
            <span class="line-divider" /> 轻盈，自在，不打扰
          </div>
        </div>
        <div class="hero-art">
          <span class="art-orbit"></span>
          <div class="desktop-preview">
            <div class="preview-bar">
              <span class="traffic"><i /><i /><i /></span><span>小伴的桌面</span
              ><span>✧</span>
            </div>
            <div class="preview-scene">
              <div class="sun-window"><i /><b /></div>
              <div class="preview-desk" />
              <div class="pet-halo" />
              <button
                class="hero-pet"
                @click="heroHappy = !heroHappy"
                aria-label="和小伴打招呼"
              >
                <img
                  :src="heroHappy ? '/companion-happy.svg' : '/companion.svg'"
                  alt="小伴桌面中的金色小猫"
                /></button
              ><span class="hello-bubble"
                >{{ heroHappy ? "见到你，真开心。" : "今天，也有我陪着你。"
                }}<Heart :size="13"
              /></span>
              <div class="desktop-label">
                <span>午后的片刻</span><b>留一点时间，给自己。</b>
              </div>
            </div>
          </div>
          <span class="floating-label"
            ><Heart :size="17" /> 会回应的桌面伙伴</span
          ><span class="art-spark one">✧</span
          ><span class="art-spark two">✦</span>
        </div>
      </section>
      <section class="benefits wrap">
        <div>
          <span>轻盈的陪伴</span><b>约 4.8 MB<small>Windows 程序实测</small></b>
        </div>
        <div>
          <span>熟悉的桌面</span
          ><b>Windows + macOS<small>系统 WebView，轻装上阵</small></b>
        </div>
        <div>
          <span>你自己的风景</span
          ><b>图片 · 视频 · 动画<small>本地导入，也支持壁纸订阅</small></b>
        </div>
      </section>
      <section id="features" class="section wrap">
        <div class="section-heading">
          <span class="eyebrow">每一个平常的日子</span>
          <h2>小小的陪伴，刚刚好。</h2>
          <p>把喜欢留在桌面，让工作和休息都多一点松弛。</p>
        </div>
        <div class="feature-grid">
          <article>
            <span class="feature-icon"><Cat /></span>
            <h3>随时相伴</h3>
            <p>让小伴悬浮在桌面，跟随你的视线。轻轻戳一下，看看它的回应。</p>
            <div class="feature-bottom">
              悬浮桌宠 · 表情互动 <ArrowUpRight :size="17" />
            </div>
          </article>
          <article>
            <span class="feature-icon"><CloudSun /></span>
            <h3>风景随心</h3>
            <p>一张照片，一段视频，或流动的动画。让小伴住进你喜欢的壁纸里。</p>
            <div class="feature-bottom">
              自定义壁纸 · 内容订阅 <ArrowUpRight :size="17" />
            </div>
          </article>
          <article>
            <span class="feature-icon"><Sparkles /></span>
            <h3>安静自在</h3>
            <p>
              收起设置，小伴继续陪伴。无需注册账号，下载后的壁纸可以离线使用。
            </p>
            <div class="feature-bottom">
              托盘常驻 · 登录启动 <ArrowUpRight :size="17" />
            </div>
          </article>
        </div>
      </section>
      <section id="wallpapers" class="section wallpaper-section">
        <div class="wrap">
          <div class="library-heading">
            <div>
              <span class="eyebrow">把喜欢的风景，留在桌面</span>
              <h2>给桌面换个心情。</h2>
              <p>选择发布的壁纸，或把自己的照片、视频导入小伴。</p>
            </div>
            <button class="button secondary" @click="copyCatalog">
              <Copy :size="16" /> 复制壁纸订阅地址
            </button>
          </div>
          <div class="filter-tabs">
            <button
              v-for="(label, key) in {
                all: '全部壁纸',
                image: '图片',
                video: '视频',
                canvas: '动画',
              }"
              :key="key"
              :class="{ active: publicFilter === key }"
              @click="publicFilter = key"
            >
              {{ label }}
            </button>
          </div>
          <p v-if="siteError" class="inline-note">
            壁纸库暂时无法连接，请稍后再来。
          </p>
          <div v-else-if="publicItems.length" class="public-gallery">
            <article v-for="w in publicItems" :key="w.id" class="public-card">
              <div class="public-cover">
                <img
                  v-if="cover(w)"
                  :src="cover(w)"
                  :alt="w.title"
                  loading="lazy"
                  @error="
                    ($event.target as HTMLImageElement).style.visibility =
                      'hidden'
                  "
                /><span v-else class="cover-placeholder"
                  ><Film v-if="w.kind === 'video'" /><Sparkles v-else /></span
                ><span class="kind-tag">{{ kinds[w.kind] }}</span>
              </div>
              <div class="public-card-body">
                <h3>{{ w.title }}</h3>
                <p>{{ w.description || "把喜欢留在你的桌面。" }}</p>
                <small
                  >{{ w.author || "小伴" }} <span>v{{ w.version }}</span></small
                >
              </div>
            </article>
          </div>
          <div v-else class="public-empty">
            <div class="empty-landscape"><i /><b /><span>✧</span></div>
            <div>
              <h3>新的风景，正在准备。</h3>
              <p>
                现在也可以把自己的图片、视频导入小伴，<br />把一份喜欢的日常，放在桌面上。
              </p>
            </div>
          </div>
        </div>
      </section>
      <section id="download" class="section wrap">
        <div class="download-panel">
          <div>
            <span class="eyebrow">给日常，一点温暖</span>
            <h2>让小伴，住进你的桌面。</h2>
            <p>从今天开始，让平常的日子多一点小美好。</p>
            <span class="release-label">{{
              site?.release
                ? `当前发布 v${site.release.version}`
                : "安装包即将发布"
            }}</span>
          </div>
          <div class="download-options">
            <template v-for="p in downloadPlatforms" :key="p.id"
              ><a
                v-if="site?.release?.platforms[p.id]?.url"
                class="download-button"
                :href="site.release.platforms[p.id].url"
                target="_blank"
                rel="noopener noreferrer"
                ><Monitor :size="22" /><span
                  ><b>下载 {{ p.name }} 版</b
                  ><small
                    >{{ p.detail }} ·
                    {{ size(site.release.platforms[p.id].bytes) }}</small
                  ></span
                ><Download :size="19" /></a
              ><button v-else class="download-button" disabled>
                <Monitor :size="22" /><span
                  ><b>{{ p.name }} · 安装包准备中</b
                  ><small>{{ p.detail }}</small></span
                >
              </button></template
            >
          </div>
        </div>
      </section>
    </main>
    <footer class="site-footer wrap">
      <a class="brand" href="/"><Cat :size="23" /> 小伴桌面</a
      ><span>认真陪伴，慢慢变好。</span
      ><a href="/admin">内容管理 <ArrowUpRight :size="14" /></a>
    </footer>
  </div>

  <div v-else-if="initializing" class="loading-screen">
    <LoaderCircle class="spin" /> 正在打开管理中心…
  </div>
  <div v-else-if="!session" class="login-page">
    <a href="/" class="brand login-brand"
      ><span class="brand-mark"><Cat :size="25" /></span>小伴桌面</a
    >
    <div class="login-story">
      <span class="eyebrow">内容管理中心</span>
      <h1>每一份风景，<br />都值得被<span>喜欢。</span></h1>
      <p>整理壁纸，管理发布。<br />把新鲜的风景，送到小伴的桌面。</p>
      <div class="login-pet"><img src="/companion.svg" alt="小伴" /></div>
      <span class="story-foot">图片、视频保留在你提供的外部地址。</span>
    </div>
    <form class="login-card" @submit.prevent="signIn">
      <span class="login-symbol"><Lock :size="23" /></span>
      <h2>欢迎回来</h2>
      <p>登录小伴桌面管理中心</p>
      <label
        >用户名<input
          v-model="loginUsername"
          autocomplete="username"
          required
          maxlength="64" /></label
      ><label
        >密码<input
          v-model="loginPassword"
          autocomplete="current-password"
          type="password"
          placeholder="输入管理员密码"
          required
          maxlength="256"
      /></label>
      <div v-if="loginError" role="alert" class="form-error">
        {{ loginError }}
      </div>
      <button class="button primary" :disabled="busy">
        <LoaderCircle v-if="busy" class="spin" :size="17" />{{
          busy ? "正在登录" : "登录管理中心"
        }}<ArrowRight v-if="!busy" :size="17" />
      </button>
      <div class="initial-account">
        <span>首次管理员</span><code>admin / 123456</code
        ><small>登录后可在账号设置中修改。</small>
      </div>
      <a href="/" class="back-site">返回产品网站 <ArrowUpRight :size="14" /></a>
    </form>
  </div>

  <div v-else class="admin-shell">
    <aside class="admin-sidebar">
      <a class="brand" href="/"
        ><span class="brand-mark"><Cat :size="24" /></span
        ><span>小伴桌面<small>内容管理中心</small></span></a
      ><span class="nav-label">工作空间</span>
      <nav>
        <button
          @click="selectSection('wallpapers')"
          :class="{ active: section === 'wallpapers' }"
        >
          <FolderHeart :size="20" /><span
            >壁纸资源<small>管理桌面里的风景</small></span
          ></button
        ><button
          @click="selectSection('release')"
          :class="{ active: section === 'release' }"
        >
          <Package :size="20" /><span
            >产品发布<small>安装包与版本信息</small></span
          ></button
        ><button
          @click="selectSection('account')"
          :class="{ active: section === 'account' }"
        >
          <Settings :size="20" /><span
            >账号设置<small>管理登录信息</small></span
          >
        </button>
      </nav>
      <div class="sidebar-end">
        <a href="/" target="_blank"><ExternalLink :size="16" /> 打开产品网站</a>
        <div>
          <span class="avatar">{{
            session.username.slice(0, 1).toUpperCase()
          }}</span
          ><span>{{ session.username }}<small>管理员</small></span
          ><button @click="signOut" aria-label="退出登录" :disabled="busy">
            <LogOut :size="17" />
          </button>
        </div>
      </div>
    </aside>
    <main class="admin-main">
      <header class="admin-heading">
        <div>
          <span class="eyebrow">小伴桌面 / 管理中心</span>
          <h1>
            {{
              {
                wallpapers: "壁纸资源",
                release: "产品发布",
                account: "账号设置",
              }[section]
            }}
          </h1>
          <p>
            {{
              {
                wallpapers: "让每一份喜欢的风景，都有一个好位置。",
                release: "发布新版本，让小伴继续变好。",
                account: "管理用户名、密码与登录会话。",
              }[section]
            }}
          </p>
        </div>
        <span class="header-status"><span class="dot" /> 仅保存资源元数据</span>
      </header>
      <div v-if="error" class="error-banner" role="alert">
        {{ error
        }}<button @click="error = ''" aria-label="关闭提示">
          <X :size="16" />
        </button>
      </div>
      <div v-if="busy" class="busy-strip">
        <LoaderCircle :size="16" class="spin" /> 正在处理，请稍候…
      </div>
      <template v-if="section === 'wallpapers'"
        ><div class="stats-grid">
          <article>
            <span>全部资源</span><b>{{ entries.length }}<small>张壁纸</small></b
            ><FolderHeart :size="21" />
          </article>
          <article>
            <span>已发布</span
            ><b>{{ publishedCount }}<small>桌面订阅可见</small></b
            ><Check :size="21" />
          </article>
          <article>
            <span>草稿</span
            ><b
              >{{ entries.length - publishedCount
              }}<small>等待整理与发布</small></b
            ><Pencil :size="21" />
          </article>
        </div>
        <section class="resource-panel">
          <div class="resource-tools">
            <div class="search-field">
              <Search :size="17" /><input
                v-model="query"
                placeholder="搜索标题、ID 或作者"
                aria-label="搜索壁纸"
              />
            </div>
            <div class="tool-actions">
              <button class="button secondary small" @click="exportJson">
                <Download :size="16" /> 导出元数据</button
              ><button
                class="button secondary small"
                @click="
                  importOpen = true;
                  error = '';
                "
                :disabled="busy"
              >
                <Upload :size="16" /> 导入 JSON</button
              ><button
                class="button primary small"
                @click="edit()"
                :disabled="busy"
              >
                <Plus :size="17" /> 新建壁纸
              </button>
            </div>
          </div>
          <div class="panel-filter">
            <div class="filter-tabs">
              <button
                v-for="(label, key) in {
                  all: '全部',
                  published: '已发布',
                  draft: '草稿',
                  image: '图片',
                  video: '视频',
                  canvas: '动画',
                }"
                :key="key"
                :class="{ active: filter === key }"
                @click="filter = key"
              >
                {{ label }}
              </button>
            </div>
            <button class="text-link" @click="copyCatalog">
              <Copy :size="14" /> 订阅地址
            </button>
          </div>
          <div class="table-scroll" v-if="visibleEntries.length">
            <table>
              <thead>
                <tr>
                  <th>壁纸</th>
                  <th>类型</th>
                  <th>状态</th>
                  <th>大小</th>
                  <th>排序</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="e in visibleEntries" :key="e.id">
                  <td>
                    <div class="resource-name">
                      <div class="resource-thumb">
                        <img
                          v-if="cover(e.definition)"
                          :src="cover(e.definition)"
                          alt=""
                          loading="lazy"
                          @error="
                            (
                              $event.target as HTMLImageElement
                            ).style.visibility = 'hidden'
                          "
                        /><Film
                          v-else-if="e.definition.kind === 'video'"
                          :size="20"
                        /><ImageIcon v-else :size="20" />
                      </div>
                      <div>
                        <b>{{ e.definition.title }}</b
                        ><small>{{ e.id }} · v{{ e.definition.version }}</small>
                      </div>
                    </div>
                  </td>
                  <td>{{ kinds[e.definition.kind] }}</td>
                  <td>
                    <span
                      :class="[
                        'status-badge',
                        e.published ? 'published' : 'draft',
                      ]"
                      >{{ e.published ? "已发布" : "草稿" }}</span
                    >
                  </td>
                  <td>
                    <span>{{ size(e.definition.resource.bytes) }}</span
                    ><small class="verified-text">{{
                      e.verifiedAt
                        ? "已探测"
                        : e.definition.resource.sha256
                          ? "已填写校验值"
                          : "校验信息待补齐"
                    }}</small>
                  </td>
                  <td>{{ e.sortOrder }}</td>
                  <td>
                    <div class="row-actions">
                      <button
                        @click="edit(e)"
                        :disabled="busy"
                        :aria-label="`编辑 ${e.definition.title}`"
                        title="编辑"
                      >
                        <Pencil :size="16" /></button
                      ><button
                        @click="verify(e)"
                        :disabled="busy"
                        :aria-label="`探测 ${e.definition.title}`"
                        title="流式探测文件大小与 SHA-256"
                      >
                        <ShieldCheck :size="16" /></button
                      ><button
                        @click="togglePublish(e)"
                        :disabled="busy"
                        :aria-label="`${e.published ? '下架' : '发布'} ${e.definition.title}`"
                        :title="e.published ? '下架' : '发布'"
                      >
                        <Download v-if="e.published" :size="16" /><ArrowUpRight
                          v-else
                          :size="16"
                        /></button
                      ><button
                        @click="deleteEntry = e"
                        :disabled="busy"
                        :aria-label="`删除 ${e.definition.title}`"
                        title="删除元数据"
                      >
                        <Trash2 :size="16" />
                      </button>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <div v-else class="admin-empty">
            <span><ImageIcon :size="34" /></span>
            <h3>
              {{
                query || filter !== "all"
                  ? "没有匹配的壁纸"
                  : "从一份喜欢的风景开始"
              }}
            </h3>
            <p>
              新建资源，或导入 JSON 元数据。<br />图片、视频仍保留在你提供的外部下载地址。
            </p>
            <button
              v-if="!query && filter === 'all'"
              class="button primary small"
              @click="importOpen = true"
            >
              <Upload :size="16" /> 导入第一份 JSON
            </button>
          </div>
          <footer class="panel-foot">
            <span>{{ visibleEntries.length }} 项资源</span
            ><span>仅发布内容进入公开桌面清单。</span>
          </footer>
        </section></template
      >
      <template v-else-if="section === 'release'"
        ><section class="settings-card">
          <div class="card-title">
            <span class="feature-icon"><Package /></span>
            <div>
              <h2>应用版本与安装包</h2>
              <p>安装包使用外部下载地址，不上传到本服务。</p>
            </div>
            <span class="status-badge">{{
              release?.published
                ? `已发布 v${release.published.version}`
                : "尚未发布版本"
            }}</span>
          </div>
          <div class="release-actions">
            <button
              v-for="p in downloadPlatforms"
              :key="p.id"
              class="button secondary small"
              @click="addPlatform(p.id)"
            >
              <Plus :size="16" /> {{ p.name }} 安装包
            </button>
          </div>
          <label
            >发布清单 JSON<textarea
              v-model="releaseText"
              class="json-editor release-editor"
              spellcheck="false"
              aria-label="发布清单 JSON"
            />
          </label>
          <p class="field-note">
            填写 SemVer 版本、说明及平台安装包的
            URL、SHA-256、bytes、mime。先保存草稿，再发布；产品网站与桌面更新接口同步使用发布记录。
          </p>
          <div class="settings-actions">
            <button
              class="button secondary"
              @click="saveRelease"
              :disabled="busy"
            >
              保存草稿</button
            ><button
              class="button primary"
              @click="publishRelease"
              :disabled="busy"
            >
              发布已保存版本 <ArrowUpRight :size="16" />
            </button>
          </div></section
      ></template>
      <template v-else
        ><section class="settings-card account-card">
          <div class="card-title">
            <span class="feature-icon"><Lock /></span>
            <div>
              <h2>管理员登录信息</h2>
              <p>修改后需要重新登录，所有旧会话会失效。</p>
            </div>
          </div>
          <p v-if="session.initialPassword" class="initial-notice">
            当前仍使用初始密码，可以在这里设置自己的密码。
          </p>
          <form @submit.prevent="changeAccount">
            <label
              >用户名<input
                v-model="account.username"
                autocomplete="username"
                required
                maxlength="64" /></label
            ><label
              >当前密码<input
                v-model="account.currentPassword"
                type="password"
                autocomplete="current-password"
                required
                maxlength="256" /></label
            ><label
              >新密码<input
                v-model="account.newPassword"
                type="password"
                autocomplete="new-password"
                placeholder="不修改密码可留空"
                minlength="6"
                maxlength="128" /></label
            ><button class="button primary" :disabled="busy">
              保存账号信息 <Check :size="17" />
            </button>
          </form></section
      ></template>
    </main>
  </div>

  <div v-if="editor" class="modal-backdrop">
    <form class="modal editor-modal" @submit.prevent="save">
      <header>
        <div>
          <span class="eyebrow">壁纸元数据</span>
          <h2>{{ newEntry ? "新建壁纸" : "编辑壁纸" }}</h2>
        </div>
        <button
          type="button"
          class="icon-button"
          @click="
            editor = null;
            error = '';
          "
          :disabled="busy"
          aria-label="关闭编辑"
        >
          <X />
        </button>
      </header>
      <div class="filter-tabs editor-tabs">
        <button
          type="button"
          :class="{ active: editorTab === 'form' }"
          @click="changeEditorTab('form')"
        >
          基本信息</button
        ><button
          type="button"
          :class="{ active: editorTab === 'json' }"
          @click="changeEditorTab('json')"
        >
          完整 JSON
        </button>
      </div>
      <div class="modal-body">
        <template v-if="editorTab === 'form'"
          ><div class="form-grid">
            <label class="full"
              >标题<input
                v-model="editor.definition.title"
                required
                placeholder="为这份风景取个名字" /></label
            ><label
              >ID<input
                v-model="editor.definition.id"
                :disabled="!newEntry"
                required /></label
            ><label
              >壁纸版本<input
                v-model="editor.definition.version"
                required
                placeholder="1.0.0" /></label
            ><label
              >类型<select
                v-model="editor.definition.kind"
                @change="changeKind"
              >
                <option value="image">图片</option>
                <option value="video">视频</option>
                <option value="canvas">Canvas 动画</option>
              </select></label
            ><label>作者<input v-model="editor.definition.author" /></label
            ><label class="full"
              >介绍<textarea
                v-model="editor.definition.description"
                rows="2"
              /></label
            ><label class="full"
              >下载地址<input
                v-model="editor.definition.resource.url"
                type="url"
                placeholder="https://…" /></label
            ><label
              >MIME<select v-model="editor.definition.resource.mime">
                <template v-if="editor.definition.kind === 'image'"
                  ><option>image/jpeg</option>
                  <option>image/png</option>
                  <option>image/webp</option>
                  <option>image/avif</option></template
                >
                <option v-else-if="editor.definition.kind === 'video'">
                  video/mp4
                </option>
                <option v-else>application/javascript</option>
              </select></label
            ><label
              >文件字节数<input
                v-model.number="editor.definition.resource.bytes"
                type="number"
                min="0" /></label
            ><label class="full"
              >SHA-256<input
                v-model="editor.definition.resource.sha256"
                class="mono"
                placeholder="64 位小写十六进制；可保存后探测" /></label
            ><label
              >排序<input
                v-model.number="editor.sortOrder"
                type="number"
                step="1" /></label
            ><label class="checkbox-field"
              ><input v-model="editor.published" type="checkbox" />
              保存并发布</label
            >
          </div>
          <button
            v-if="!editor.definition.cover"
            type="button"
            class="text-link cover-add"
            @click="
              editor.definition.cover = {
                url: '',
                sha256: '',
                bytes: 0,
                mime: 'image/jpeg',
              }
            "
          >
            <Plus :size="15" /> 添加封面地址
          </button>
          <fieldset v-else class="cover-form">
            <legend>
              封面资源
              <button type="button" @click="delete editor.definition.cover">
                移除
              </button>
            </legend>
            <label
              >封面下载地址<input
                v-model="editor.definition.cover.url"
                type="url"
                placeholder="https://…"
            /></label>
            <div class="form-grid">
              <label
                >封面 MIME<select v-model="editor.definition.cover.mime">
                  <option>image/jpeg</option>
                  <option>image/png</option>
                  <option>image/webp</option>
                  <option>image/avif</option>
                </select></label
              ><label
                >封面字节数<input
                  v-model.number="editor.definition.cover.bytes"
                  type="number"
                  min="0" /></label
              ><label class="full"
                >封面 SHA-256<input
                  v-model="editor.definition.cover.sha256"
                  class="mono"
              /></label>
            </div>
          </fieldset>
          <p class="field-note">
            只有地址时可先保存草稿，再在资源列表探测大小和
            SHA-256。动画参数与附属资源使用完整 JSON 编辑。
          </p></template
        ><template v-else>
          <textarea
            v-model="rawDefinition"
            class="json-editor"
            spellcheck="false"
            aria-label="完整壁纸 JSON"
          />
          <p class="field-note">
            JSON 修改在保存后生效。切回基本信息会使用原先表单内容。
          </p></template
        >
        <p v-if="error" class="form-error" role="alert">{{ error }}</p>
      </div>
      <footer>
        <button
          type="button"
          class="button secondary"
          @click="
            editor = null;
            error = '';
          "
          :disabled="busy"
        >
          取消</button
        ><button class="button primary" :disabled="busy">
          <LoaderCircle v-if="busy" class="spin" :size="17" /> 保存壁纸
        </button>
      </footer>
    </form>
  </div>
  <div v-if="importOpen" class="modal-backdrop">
    <form class="modal import-modal" @submit.prevent="importJson">
      <header>
        <div>
          <span class="eyebrow">批量整理</span>
          <h2>导入壁纸 JSON</h2>
        </div>
        <button
          type="button"
          class="icon-button"
          @click="
            importOpen = false;
            error = '';
          "
          :disabled="busy"
          aria-label="关闭导入"
        >
          <X />
        </button>
      </header>
      <div class="modal-body">
        <p class="modal-description">
          支持桌面 v1 清单、壁纸数组或一条简化记录。所有导入默认保存为草稿。
        </p>
        <label class="file-select button secondary"
          ><Upload :size="16" /> 选择 JSON 文件<input
            type="file"
            accept=".json,application/json"
            @change="readFile"
            hidden /></label
        ><textarea
          v-model="importText"
          class="json-editor"
          spellcheck="false"
          required
          aria-label="导入 JSON"
          :placeholder="
            JSON.stringify(
              {
                title: '午后的海',
                kind: 'video',
                downloadUrl: 'https://你的资源域名/afternoon.mp4',
              },
              null,
              2,
            )
          "
        /><label class="checkbox-field import-check"
          ><input v-model="overwrite" type="checkbox" /> 更新相同 ID
          的资源，并转为草稿</label
        >
        <p class="field-note">
          导入是一次事务；任何记录失败，整批都不会保存。图片和视频不会上传到本服务器。
        </p>
        <p v-if="error" class="form-error" role="alert">{{ error }}</p>
      </div>
      <footer>
        <button
          type="button"
          class="button secondary"
          @click="
            importOpen = false;
            error = '';
          "
          :disabled="busy"
        >
          取消</button
        ><button class="button primary" :disabled="busy">
          导入为草稿 <ArrowRight :size="16" />
        </button>
      </footer>
    </form>
  </div>
  <div v-if="deleteEntry" class="modal-backdrop">
    <section class="modal confirm-modal" role="dialog" aria-modal="true">
      <span class="feature-icon"><Trash2 /></span>
      <h2>删除这份壁纸元数据？</h2>
      <p>{{ deleteEntry.definition.title }}</p>
      <p class="field-note">
        会从壁纸库与公开订阅移除，不影响外部地址的原始媒体文件。
      </p>
      <p v-if="error" class="form-error">{{ error }}</p>
      <div class="settings-actions">
        <button
          class="button secondary"
          @click="deleteEntry = null"
          :disabled="busy"
        >
          取消</button
        ><button class="button primary" @click="remove" :disabled="busy">
          确认删除
        </button>
      </div>
    </section>
  </div>
  <div v-if="toast" class="toast" role="status">
    <Check :size="17" /> {{ toast }}
  </div>
</template>
