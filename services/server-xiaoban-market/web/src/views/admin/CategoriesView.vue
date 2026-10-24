<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { admin } from '@/lib/api';
import { KIND_NAME } from '@/lib/format';
import { toast, toastError } from '@/lib/toast';
import type { AdminCategory, Kind } from '@/lib/types';

const KINDS: Kind[] = ['skill', 'expert', 'connector'];
const kind = ref<Kind>('skill');
const list = ref<AdminCategory[]>([]);
const draft = ref({ name: '', slug: '' });
const loading = ref(true);

async function load() {
  loading.value = true;
  try { list.value = (await admin.categories(kind.value)).results; } catch (e) { toastError(e); } finally { loading.value = false; }
}

async function save(c: AdminCategory) {
  try {
    await admin.updateCategory(c.id, { name: c.name, nameEn: c.nameEn, sortOrder: Number(c.sortOrder) || 0, active: c.active });
    toast('已保存');
  } catch (e) { toastError(e); await load(); }
}

async function add() {
  try {
    await admin.createCategory({ kind: kind.value, name: draft.value.name, slug: draft.value.slug.trim() || undefined });
    draft.value = { name: '', slug: '' };
    toast('已添加');
    await load();
  } catch (e) { toastError(e); }
}

async function remove(c: AdminCategory) {
  const hint = c.items ? `有 ${c.items} 个条目使用它，删除后这些条目会变成「未分类」。` : '';
  if (!window.confirm(`确定删除分类「${c.name}」吗？${hint}`)) return;
  try { await admin.deleteCategory(c.id); toast('已删除'); await load(); } catch (e) { toastError(e); }
}

watch(kind, load);
onMounted(load);
</script>

<template>
  <div class="admin-page">
    <div class="page-h"><div><h1>分类</h1><div class="sub">技能、专家、连接器各有一套分类；停用的分类不会出现在前台</div></div></div>
    <div class="seg-tabs" role="group" aria-label="类型"><button v-for="k in KINDS" :key="k" type="button" :class="{ on: kind === k }" @click="kind = k">{{ KIND_NAME[k] }}</button></div>

    <section class="panel">
      <div v-if="loading" class="empty"><span class="spinner" /></div>
      <div v-else class="tbl-wrap">
        <table class="tbl">
          <thead><tr><th>名称</th><th>标识</th><th style="width: 90px">排序</th><th style="width: 70px">启用</th><th>条目</th><th /></tr></thead>
          <tbody>
            <tr v-for="c in list" :key="c.id">
              <td><input v-model="c.name" class="input" :aria-label="`分类名称 ${c.slug}`" /></td>
              <td class="mono sub">{{ c.slug }}</td>
              <td><input v-model.number="c.sortOrder" class="input" inputmode="numeric" :aria-label="`排序 ${c.name}`" /></td>
              <td><input v-model="c.active" type="checkbox" :aria-label="`启用 ${c.name}`" /></td>
              <td class="num">{{ c.items }}</td>
              <td class="acts"><button class="btn btn-sm" type="button" @click="save(c)">保存</button><button class="btn btn-sm btn-danger" type="button" @click="remove(c)">删除</button></td>
            </tr>
          </tbody>
        </table>
      </div>
      <form class="add" @submit.prevent="add">
        <div class="field"><label for="cn">新分类名称</label><input id="cn" v-model="draft.name" class="input" required /></div>
        <div class="field"><label for="cs">标识（可选，留空自动生成）</label><input id="cs" v-model="draft.slug" class="input mono-input" /></div>
        <button class="btn btn-primary" type="submit" :disabled="!draft.name.trim()">添加</button>
      </form>
    </section>
  </div>
</template>

<style scoped>
.acts { display: flex; gap: 6px; }
.add { display: flex; flex-wrap: wrap; align-items: flex-end; gap: 12px; padding: 16px 20px; border-top: 1px solid var(--line); }
.add .field { flex: 1 1 200px; }
</style>
