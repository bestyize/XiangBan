<template>
  <div class="admin-categories-page">
    <div class="page-header">
      <div class="header-intro">
        <h2>8 大特色分类体系</h2>
        <p>创新场景与软件基因设计，精准连接极客与优质独立软件。</p>
      </div>
    </div>

    <div class="categories-table-card">
      <table class="cat-table">
        <thead>
          <tr>
            <th>分类图标</th>
            <th>分类名称</th>
            <th>唯一 Slug</th>
            <th>分类定位与理念说明</th>
            <th>排序权重</th>
            <th class="text-center">收录应用数</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="cat in categories" :key="cat.id">
            <td>
              <div class="icon-avatar">
                <component :is="getCategoryIcon(cat.icon)" :size="18" />
              </div>
            </td>
            <td>
              <span class="cat-name-cell">{{ cat.name }}</span>
            </td>
            <td>
              <code class="slug-cell">{{ cat.slug }}</code>
            </td>
            <td>
              <span class="desc-cell">{{ cat.description }}</span>
            </td>
            <td>
              <span class="sort-cell">{{ cat.sort_order }}</span>
            </td>
            <td class="text-center">
              <span class="count-badge">{{ cat.app_count || 0 }} 款</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { Category } from '../../types';
import { api } from '../../api';
import {
  Grid,
  Bot,
  Sparkles,
  Github,
  Rocket,
  Terminal,
  Feather,
  PenTool,
  Palette,
} from 'lucide-vue-next';

const categories = ref<Category[]>([]);

function getCategoryIcon(iconName: string) {
  const map: Record<string, any> = {
    Bot,
    Sparkles,
    Github,
    Rocket,
    Terminal,
    Feather,
    PenTool,
    Palette,
  };
  return map[iconName] || Grid;
}

async function loadCategories() {
  try {
    categories.value = await api.getCategories();
  } catch (err) {
    console.error('加载分类失败:', err);
  }
}

onMounted(() => {
  loadCategories();
});
</script>

<style scoped>
.admin-categories-page {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.page-header h2 {
  font-size: 18px;
  font-weight: 700;
  color: #0f172a;
}

.page-header p {
  font-size: 13px;
  color: #64748b;
  margin-top: 4px;
}

.categories-table-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.cat-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
  text-align: left;
}

.cat-table th {
  background: #f8fafc;
  padding: 12px 18px;
  font-weight: 600;
  color: #475569;
  border-bottom: 1px solid #e2e8f0;
}

.cat-table td {
  padding: 14px 18px;
  border-bottom: 1px solid #f1f5f9;
  vertical-align: middle;
}

.icon-avatar {
  width: 36px;
  height: 36px;
  background: #eff6ff;
  color: #2563eb;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.cat-name-cell {
  font-weight: 600;
  color: #0f172a;
}

.slug-cell {
  background: #f1f5f9;
  color: #475569;
  padding: 2px 6px;
  border-radius: 4px;
  font-family: monospace;
  font-size: 12px;
}

.desc-cell {
  color: #64748b;
  font-size: 13px;
}

.count-badge {
  background: #ecfdf5;
  color: #059669;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 600;
  font-size: 12px;
}

.text-center {
  text-align: center;
}
</style>
