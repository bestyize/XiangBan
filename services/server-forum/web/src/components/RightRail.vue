<script setup lang="ts">
import { computed } from 'vue'
import { useMeta } from '@/stores/meta'
import { useSession } from '@/stores/session'
import { TRUST_LABEL } from '@/lib/format'

const meta = useMeta()
const session = useSession()

const topProducts = computed(() => meta.products.filter(p => p.slug !== 'other').slice(0, 10))

/** 新人离自动升级还差什么。 */
const progress = computed(() => {
  const info = session.info
  if (!info || info.user.trust_level !== 0) return null
  const days = Math.floor((Date.now() - new Date(info.user.created_at).getTime()) / 86_400_000)
  return {
    days: Math.min(days, info.limits.tl1_days),
    needDays: info.limits.tl1_days,
    reads: Math.min(info.user.read_count, info.limits.tl1_reads),
    needReads: info.limits.tl1_reads,
  }
})
</script>

<template>
  <section v-if="session.user" class="panel box">
    <h2>我的等级</h2>
    <p class="level">
      <strong>{{ TRUST_LABEL[session.user.trust_level] }}</strong>
      <span class="muted">信任等级 {{ session.user.trust_level }}</span>
    </p>
    <template v-if="progress">
      <p class="hint">再多看看帖子就能发帖。</p>
      <div class="meter">
        <span>注册天数</span>
        <span class="num">{{ progress.days }} / {{ progress.needDays }}</span>
      </div>
      <div class="track"><i :style="{ width: `${(progress.days / progress.needDays) * 100}%` }" /></div>
      <div class="meter">
        <span>读过的帖子</span>
        <span class="num">{{ progress.reads }} / {{ progress.needReads }}</span>
      </div>
      <div class="track"><i :style="{ width: `${(progress.reads / progress.needReads) * 100}%` }" /></div>
    </template>
  </section>

  <section class="panel box">
    <h2>羊毛最多的产品</h2>
    <ul class="products">
      <li v-for="p in topProducts" :key="p.id">
        <RouterLink :to="`/p/${p.slug}`">{{ p.name }}</RouterLink>
        <span class="num">{{ p.active_deal_count || '' }}</span>
      </li>
    </ul>
    <RouterLink to="/products" class="more">全部产品</RouterLink>
  </section>

  <section class="panel box">
    <h2>发帖前看一眼</h2>
    <ul class="rules">
      <li>只贴官方活动页，不带个人推广链接</li>
      <li>不分享、不交易 API Key 和账号</li>
      <li>写清楚领取条件和有效期，帮后来的人省时间</li>
    </ul>
    <RouterLink to="/rules" class="more">社区规则</RouterLink>
  </section>
</template>

<style scoped>
.box {
  padding: 14px 16px 16px;
}
h2 {
  margin: 0 0 10px;
  font-size: var(--fs-sm);
  color: var(--ink-2);
}
.level {
  margin: 0;
  display: flex;
  align-items: baseline;
  gap: 8px;
}
.level strong {
  font-size: var(--fs-lg);
}
.hint {
  margin: 6px 0 10px;
  font-size: var(--fs-xs);
  color: var(--ink-3);
}
.meter {
  display: flex;
  justify-content: space-between;
  font-size: var(--fs-xs);
  color: var(--ink-2);
  margin-top: 8px;
}
.track {
  height: 5px;
  border-radius: 3px;
  background: var(--surface-2);
  border: 1px solid var(--line);
  overflow: hidden;
  margin-top: 4px;
}
.track i {
  display: block;
  height: 100%;
  background: var(--valid);
}
.num {
  font-family: var(--font-display);
  font-variant-numeric: tabular-nums;
  color: var(--ink-3);
  font-size: 12px;
}
.products,
.rules {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 2px;
}
.products li {
  display: flex;
  justify-content: space-between;
  padding: 4px 0;
  font-size: var(--fs-sm);
}
.products a:hover {
  text-decoration: underline;
}
.rules li {
  font-size: var(--fs-xs);
  color: var(--ink-2);
  padding: 4px 0 4px 14px;
  position: relative;
  line-height: 1.55;
}
.rules li::before {
  content: '';
  position: absolute;
  left: 2px;
  top: 11px;
  width: 5px;
  height: 5px;
  border-radius: 1px;
  background: var(--brand);
}
.more {
  display: inline-block;
  margin-top: 8px;
  font-size: var(--fs-xs);
  color: var(--ink-2);
  text-decoration: underline;
  text-underline-offset: 3px;
}
</style>
