<script setup lang="ts">
import type { RouteLocationRaw } from 'vue-router';
import { formatCount, highlight } from '@/lib/format';
import { memberText, VARIANT_LABEL } from '@/lib/tabs';
import { CONNECTOR_AUTH } from '@/lib/labels';
import type { Item } from '@/lib/types';
import Avatar from './Avatar.vue';
import Icon from './Icon.vue';

defineProps<{ item: Item; to: RouteLocationRaw; keyword?: string; muted?: boolean }>();
</script>

<template>
  <RouterLink :to="to" class="card" :class="{ muted }">
    <div class="top">
      <Avatar :name="item.name" :seed="item.slug" :icon-url="item.iconUrl" :size="muted ? 48 : 56" />
      <div class="main">
        <div class="name">
          <span class="nm">
            <template v-for="(s, i) in highlight(item.name, keyword ?? '')" :key="i">
              <mark v-if="s.hit">{{ s.text }}</mark>
              <template v-else>{{ s.text }}</template>
            </template>
          </span>
          <span v-if="item.variant !== 'single'" class="vtag">{{ VARIANT_LABEL[item.variant] }}</span>
        </div>
        <div v-if="item.expert?.profession?.zh" class="profession">{{ item.expert.profession.zh }}<template v-if="item.expert.skills?.length"> · {{ item.expert.skills.length }} 个技能</template></div>
        <div v-if="item.connector" class="profession">{{ item.connector.authTypes.map(a => CONNECTOR_AUTH[a]).join(' · ') }} · {{ item.connector.skillCount }} 个随包技能</div>
        <div v-if="item.connector" class="summary">{{ (item.connector.examples.zh.length ? item.connector.examples.zh : item.connector.examples.en)[0] }}</div>
        <div class="summary">{{ item.summary || '暂无简介' }}</div>
      </div>
    </div>
    <div v-if="item.tags.length && !muted" class="tags">
      <span v-for="t in item.tags.slice(0, 3)" :key="t" class="chip">{{ t }}</span>
    </div>
    <div v-if="!muted" class="meta">
      <span class="stat"><Icon name="bolt" :size="16" />{{ formatCount(item.heat) }}</span>
      <span v-if="item.kind === 'skill'" class="stat"><Icon name="download" :size="16" />{{ formatCount(item.downloads) }}</span>
      <span v-if="memberText(item.variant, item.memberCount)" class="stat">{{ memberText(item.variant, item.memberCount) }}</span>
      <span class="slug mono">{{ item.slug }}</span>
    </div>
  </RouterLink>
</template>

<style scoped>
.card { display: flex; flex-direction: column; gap: 16px; padding: 22px 24px; border-radius: 20px; background: #fff; border: 1px solid var(--line); box-shadow: var(--shadow-card); min-width: 0; transition: border-color 0.15s, transform 0.15s; }
.card:hover { border-color: var(--line-2); transform: translateY(-1px); }
.card.muted { background: #fcfcfb; border-style: dashed; border-color: #dededc; box-shadow: none; padding: 18px 22px; gap: 0; }
.top { display: flex; align-items: center; gap: 16px; min-width: 0; }
.main { flex: 1; min-width: 0; }
.name { display: flex; align-items: center; gap: 8px; font-size: 17px; font-weight: 600; letter-spacing: -0.005em; min-width: 0; }
.nm { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; min-width: 0; }
.vtag { flex: 0 0 auto; padding: 1px 8px; border-radius: 6px; background: #141414; color: #fff; font-size: 12px; font-weight: 500; }
.profession { color: var(--ink-3); font-size: 13px; margin-bottom: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.summary { color: var(--ink-2); font-size: 14px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.tags { display: flex; flex-wrap: wrap; gap: 8px; }
.meta { display: flex; align-items: center; gap: 18px; color: var(--ink-3); font-size: 14px; font-variant-numeric: tabular-nums; }
.stat { display: inline-flex; align-items: center; gap: 6px; }
.slug { margin-left: auto; font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 45%; }
mark { background: #fbedb5; color: inherit; border-radius: 4px; padding: 0 2px; }
</style>
