<template>
  <div class="modal-backdrop" @click="$emit('close')">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h3>{{ isEdit ? '编辑应用信息' : '收录新优质应用' }}</h3>
        <button class="btn-close" @click="$emit('close')">
          <X :size="18" />
        </button>
      </div>

      <form class="modal-form" @submit.prevent="handleSubmit">
        <div class="form-grid">
          <!-- 基本信息 -->
          <div class="form-group">
            <label>应用名称 <span class="required">*</span></label>
            <input v-model="form.name" type="text" required placeholder="如 Claude Desktop, Raycast" />
          </div>

          <div class="form-group">
            <label>唯一标识 Slug <span class="required">*</span></label>
            <input v-model="form.slug" type="text" required placeholder="如 claude-desktop, raycast" />
          </div>

          <div class="form-group">
            <label>应用图标 URL <span class="required">*</span></label>
            <input v-model="form.icon_url" type="url" required placeholder="https://..." />
          </div>

          <div class="form-group">
            <label>所属分类 <span class="required">*</span></label>
            <select v-model="form.category_id" required>
              <option v-for="cat in categories" :key="cat.id" :value="cat.id">
                {{ cat.name }} ({{ cat.slug }})
              </option>
            </select>
          </div>

          <div class="form-group">
            <label>当前推荐版本</label>
            <input v-model="form.version" type="text" placeholder="如 1.2.0" />
          </div>

          <div class="form-group">
            <label>资费类型</label>
            <select v-model="form.pricing_type">
              <option value="free">完全免费 (Free)</option>
              <option value="open_source">开源免费 (Open Source)</option>
              <option value="freemium">免费 + 内购 (Freemium)</option>
              <option value="paid">买断/付费 (Paid)</option>
            </select>
          </div>

          <div class="form-group full-width">
            <label>一句话简介 <span class="required">*</span></label>
            <input v-model="form.summary" type="text" required placeholder="一句话概括核心功能与亮点" />
          </div>

          <!-- 柠檬风编辑推荐语 -->
          <div class="form-group full-width">
            <label>小伴精选推荐语（卡片核心寄语） <span class="required">*</span></label>
            <textarea
              v-model="form.comment"
              rows="3"
              required
              placeholder="以“编辑推荐”语气阐述为什么值得下载、解决了什么痛点..."
            ></textarea>
          </div>

          <div class="form-group">
            <label>推荐人/推荐团队</label>
            <input v-model="form.referrer" type="text" placeholder="如 小伴精选编辑部, 极客合伙人" />
          </div>

          <div class="form-group">
            <label>初始状态</label>
            <select v-model="form.status">
              <option value="published">🟢 立即上架 (Published)</option>
              <option value="unlisted">🔴 暂不公开/下架 (Unlisted)</option>
              <option value="draft">📝 草稿 (Draft)</option>
            </select>
          </div>

          <!-- 支持平台多选 -->
          <div class="form-group full-width">
            <label>支持平台 (多选)</label>
            <div class="checkbox-group">
              <label v-for="plat in platformOptions" :key="plat.id" class="checkbox-label">
                <input
                  type="checkbox"
                  :value="plat.id"
                  v-model="form.platforms"
                />
                <span>{{ plat.name }}</span>
              </label>
            </div>
          </div>

          <!-- 品质徽章多选 -->
          <div class="form-group full-width">
            <label>品质与安全徽章 (多选)</label>
            <div class="checkbox-group">
              <label v-for="badge in badgeOptions" :key="badge.id" class="checkbox-label">
                <input
                  type="checkbox"
                  :value="badge.id"
                  v-model="form.badges"
                />
                <span>{{ badge.name }}</span>
              </label>
            </div>
          </div>

          <!-- 下载源与直接安装包直链 -->
          <div class="form-group full-width direct-section-box">
            <div class="section-subhead">
              <span class="subhead-title">📦 直接安装包直链 (用户点击直接下载文件，无需跳转官网)</span>
              <button
                type="button"
                class="btn-auto-collect"
                :disabled="autoCollecting"
                @click="autoCollectVersion"
                title="根据 Homebrew Cask 或 GitHub 自动抓取最新直链"
              >
                {{ autoCollecting ? '抓取中...' : '⚡️ 从源自动抓取直链' }}
              </button>
            </div>

            <div class="direct-grid">
              <div class="form-group">
                <label>🍏 macOS 直接安装包 (.dmg / .zip 直链)</label>
                <input v-model="form.mac_download_url" type="url" placeholder="https://.../App.dmg" />
              </div>
              <div class="form-group">
                <label>🪟 Windows 直接安装包 (.exe / .msi 直链)</label>
                <input v-model="form.win_download_url" type="url" placeholder="https://.../Setup.exe" />
              </div>
              <div class="form-group">
                <label>🐧 Linux 直接安装包 (.AppImage / .deb 直链)</label>
                <input v-model="form.linux_download_url" type="url" placeholder="https://.../App.AppImage" />
              </div>
              <div class="form-group">
                <label>安装包大小 (可选)</label>
                <input v-model="form.file_size" type="text" placeholder="如 85 MB" />
              </div>
            </div>
          </div>

          <div class="form-group full-width">
            <label>备用/官网直达地址 <span class="required">*</span></label>
            <input v-model="form.download_url" type="url" required placeholder="https://..." />
          </div>

          <div class="form-group">
            <label>官方网站主页</label>
            <input v-model="form.official_url" type="url" placeholder="https://..." />
          </div>

          <div class="form-group">
            <label>Mac App Store 地址</label>
            <input v-model="form.appstore_url" type="url" placeholder="https://apps.apple.com/..." />
          </div>

          <div class="form-group">
            <label>GitHub 仓库主页</label>
            <input v-model="form.github_url" type="url" placeholder="https://github.com/..." />
          </div>

          <div class="form-group">
            <label>GitHub Star 数量</label>
            <input v-model.number="form.github_stars" type="number" placeholder="如 36000" />
          </div>

          <!-- 极客包管理器 -->
          <div class="form-group">
            <label>Homebrew Cask 标识</label>
            <input v-model="form.brew_cask" type="text" placeholder="如 claude, raycast" />
          </div>

          <div class="form-group">
            <label>Winget ID (Windows)</label>
            <input v-model="form.winget_id" type="text" placeholder="如 Anysphere.Cursor" />
          </div>

          <!-- 优缺点与平替 (逗号或换行分隔) -->
          <div class="form-group">
            <label>优势亮点 (逗号分隔)</label>
            <input v-model="prosInput" type="text" placeholder="如 原生流畅, 启动秒开, 零弹窗" />
          </div>

          <div class="form-group">
            <label>局限/适合谁 (逗号分隔)</label>
            <input v-model="consInput" type="text" placeholder="如 配置门槛稍高, 仅支持英文" />
          </div>

          <div class="form-group">
            <label>痛点平替 (逗号分隔)</label>
            <input v-model="altsInput" type="text" placeholder="如 Photoshop 全家桶, 迅雷" />
          </div>

          <div class="form-group">
            <label>排序权重 (数字越大越靠前)</label>
            <input v-model.number="form.sort_order" type="number" placeholder="0" />
          </div>
        </div>

        <div class="modal-actions">
          <button type="button" class="btn-secondary" @click="$emit('close')">取消</button>
          <button type="submit" class="btn-primary" :disabled="submitting">
            {{ submitting ? '保存中...' : '确认保存' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue';
import type { AppItem, Category } from '../../types';
import { X } from 'lucide-vue-next';
import { api } from '../../api';

const props = defineProps<{
  app?: AppItem | null;
  categories: Category[];
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'saved'): void;
}>();

const isEdit = computed(() => !!props.app?.id);
const submitting = ref(false);

const platformOptions = [
  { id: 'mac', name: 'macOS' },
  { id: 'windows', name: 'Windows' },
  { id: 'linux', name: 'Linux' },
  { id: 'web', name: 'Web 网页' },
  { id: 'ios', name: 'iOS' },
  { id: 'android', name: 'Android' },
];

const badgeOptions = [
  { id: 'safe', name: '🛡️ 安全认证' },
  { id: 'native', name: '⚡️ 原生极速' },
  { id: 'open_source', name: '🐙 开源之光' },
  { id: 'clean', name: '🍃 纯净无广告' },
  { id: 'local_first', name: '🔒 本地优先' },
  { id: 'ai', name: '🤖 AI 智能体' },
  { id: 'recommended', name: '⭐ 编辑推荐' },
];

const prosInput = ref('');
const consInput = ref('');
const altsInput = ref('');

const autoCollecting = ref(false);

const form = reactive({
  name: '',
  slug: '',
  icon_url: '',
  category_id: props.categories[0]?.id || 1,
  version: '',
  pricing_type: 'free' as AppItem['pricing_type'],
  summary: '',
  comment: '',
  referrer: '小伴精选编辑部',
  status: 'published' as AppItem['status'],
  platforms: ['mac'],
  badges: ['safe'],
  download_url: '',
  official_url: '',
  appstore_url: '',
  download_alt_url: '',
  github_url: '',
  github_stars: 0,
  brew_cask: '',
  winget_id: '',
  mac_download_url: '',
  win_download_url: '',
  linux_download_url: '',
  file_size: '',
  sort_order: 10,
});

onMounted(() => {
  if (props.app) {
    form.name = props.app.name;
    form.slug = props.app.slug;
    form.icon_url = props.app.icon_url;
    form.category_id = props.app.category_id;
    form.version = props.app.version || '';
    form.pricing_type = props.app.pricing_type;
    form.summary = props.app.summary;
    form.comment = props.app.comment;
    form.referrer = props.app.referrer || '小伴精选编辑部';
    form.status = props.app.status;
    form.platforms = Array.isArray(props.app.platforms) ? [...props.app.platforms] : ['mac'];
    form.badges = Array.isArray(props.app.badges) ? [...props.app.badges] : ['safe'];
    form.download_url = props.app.download_url;
    form.official_url = props.app.official_url || '';
    form.appstore_url = props.app.appstore_url || '';
    form.download_alt_url = props.app.download_alt_url || '';
    form.github_url = props.app.github_url || '';
    form.github_stars = props.app.github_stars || 0;
    form.brew_cask = props.app.brew_cask || '';
    form.winget_id = props.app.winget_id || '';
    form.mac_download_url = props.app.mac_download_url || '';
    form.win_download_url = props.app.win_download_url || '';
    form.linux_download_url = props.app.linux_download_url || '';
    form.file_size = props.app.file_size || '';
    form.sort_order = props.app.sort_order || 0;

    prosInput.value = (props.app.pros || []).join(', ');
    consInput.value = (props.app.cons || []).join(', ');
    altsInput.value = (props.app.alternatives || []).join(', ');
  }
});

async function autoCollectVersion() {
  if (props.app?.id) {
    autoCollecting.value = true;
    try {
      const res = await api.syncAppVersion(props.app.id);
      if (res.version) form.version = res.version;
      if (res.mac_download_url) form.mac_download_url = res.mac_download_url;
      if (res.win_download_url) form.win_download_url = res.win_download_url;
      if (res.linux_download_url) form.linux_download_url = res.linux_download_url;
      if (res.file_size) form.file_size = res.file_size;
      alert(`已成功自动采集到最新版本 v${res.version || ''} 及直接安装包！`);
    } catch (err: any) {
      alert(`采集失败: ${err.message || '未获取到直链，请手动输入'}`);
    } finally {
      autoCollecting.value = false;
    }
  } else {
    alert('请先保存新建软件后，点击“从源自动抓取直链”');
  }
}

function splitStr(str: string): string[] {
  return str
    .split(/[,，\n]/)
    .map((s) => s.trim())
    .filter(Boolean);
}

async function handleSubmit() {
  submitting.value = true;
  try {
    const payload = {
      ...form,
      pros: splitStr(prosInput.value),
      cons: splitStr(consInput.value),
      alternatives: splitStr(altsInput.value),
    };

    if (isEdit.value && props.app?.id) {
      await api.updateApp(props.app.id, payload);
    } else {
      await api.createApp(payload);
    }
    emit('saved');
    emit('close');
  } catch (err: any) {
    alert(err.message || '保存失败');
  } finally {
    submitting.value = false;
  }
}
</script>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.5);
  backdrop-filter: blur(4px);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.modal-content {
  background: #ffffff;
  width: 100%;
  max-width: 800px;
  max-height: 90vh;
  border-radius: 16px;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  padding: 20px 24px;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.modal-header h3 {
  font-size: 17px;
  font-weight: 700;
  color: #0f172a;
}

.btn-close {
  color: #94a3b8;
  padding: 4px;
  border-radius: 6px;
}

.btn-close:hover {
  background: #f1f5f9;
  color: #0f172a;
}

.modal-form {
  padding: 24px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.form-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.full-width {
  grid-column: 1 / -1;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 13px;
  font-weight: 600;
  color: #334155;
}

.required {
  color: #ef4444;
}

.form-group input,
.form-group select,
.form-group textarea {
  padding: 9px 12px;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  color: #1e293b;
  outline: none;
  transition: all 0.15s ease;
}

.form-group input:focus,
.form-group select:focus,
.form-group textarea:focus {
  border-color: #2563eb;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.15);
}

.checkbox-group {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  background: #f8fafc;
  padding: 10px 14px;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
}

.checkbox-label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: #334155;
  cursor: pointer;
}

.direct-section-box {
  background: #f8fafc;
  border: 1px solid #cbd5e1;
  border-radius: 10px;
  padding: 16px;
}

.section-subhead {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.subhead-title {
  font-size: 13px;
  font-weight: 700;
  color: #0f172a;
}

.btn-auto-collect {
  font-size: 12px;
  font-weight: 600;
  color: #2563eb;
  background: #eff6ff;
  border: 1px solid #bfdbfe;
  padding: 4px 10px;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.btn-auto-collect:hover:not(:disabled) {
  background: #2563eb;
  color: #ffffff;
}

.direct-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.modal-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid #e2e8f0;
}
</style>
