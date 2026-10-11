<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import ForumShell from '@/components/ForumShell.vue'
import TopicList from '@/components/TopicList.vue'
import { api, ApiError } from '@/api/forum'
import type { Product, TopicItem } from '@/api/types'
import { useToast } from '@/stores/toast'
import { useHead } from '@/lib/head'

const route = useRoute()
const toast = useToast()
const slug = computed(() => String(route.params.slug))

const product = ref<Product | null>(null)
const deals = ref<TopicItem[]>([])
const discussions = ref<TopicItem[]>([])
const loading = ref(true)
const missing = ref(false)
const showExpired = ref(false)

useHead(() => ({ title: product.value ? `${product.value.name} 羊毛` : '产品' }))

watch(
  slug,
  async s => {
    loading.value = true
    missing.value = false
    try {
      const res = await api.product(s)
      product.value = res.product
      deals.value = res.deals
      discussions.value = res.discussions
    } catch (e) {
      if (e instanceof ApiError && e.status === 404) missing.value = true
      else toast.error(e)
    } finally {
      loading.value = false
    }
  },
  { immediate: true },
)

const active = computed(() => deals.value.filter(d => d.deal?.deal_status !== 'expired'))
const expired = computed(() => deals.value.filter(d => d.deal?.deal_status === 'expired'))
</script>

<template>
  <ForumShell active="products" :rail="false">
    <div v-if="missing" class="panel missing">
      <h1>没有这个产品</h1>
      <RouterLink to="/products" class="btn">看看全部产品</RouterLink>
    </div>
    <template v-else>
      <header class="head">
        <span class="initial" aria-hidden="true">{{ product ? Array.from(product.name)[0] : '' }}</span>
        <div class="text">
          <h1>{{ product?.name ?? '' }}</h1>
          <p class="muted">
            <span v-if="product?.vendor && product.vendor !== product.name">{{ product.vendor }}</span>
            <a v-if="product?.website" :href="product.website" target="_blank" rel="noopener noreferrer" class="site">官网</a>
          </p>
          <p v-if="product?.description" class="desc">{{ product.description }}</p>
        </div>
        <RouterLink v-if="product" :to="`/new?category=deals&product=${product.slug}`" class="btn btn-primary">分享这家的羊毛</RouterLink>
      </header>

      <section class="block" aria-labelledby="active-title">
        <h2 id="active-title">在领的羊毛 <span class="n">{{ active.length }}</span></h2>
        <TopicList :items="active" :loading="loading" empty="现在没有能领的羊毛。" />
      </section>

      <section v-if="expired.length" class="block" aria-labelledby="expired-title">
        <h2 id="expired-title">
          历史羊毛 <span class="n">{{ expired.length }}</span>
          <button type="button" class="btn btn-ghost btn-sm" @click="showExpired = !showExpired">{{ showExpired ? '收起' : '展开' }}</button>
        </h2>
        <TopicList v-if="showExpired" :items="expired" />
      </section>

      <section class="block" aria-labelledby="talk-title">
        <h2 id="talk-title">相关讨论</h2>
        <TopicList :items="discussions" :loading="loading" empty="还没有讨论。发帖时加上这个产品名做标签，帖子就会出现在这里。" />
      </section>
    </template>
  </ForumShell>
</template>

<style scoped>
.head {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 16px;
  align-items: center;
  margin-bottom: 20px;
}
.initial {
  width: 56px;
  height: 56px;
  border-radius: 14px;
  display: grid;
  place-items: center;
  background: var(--brand);
  color: var(--brand-ink);
  font-family: var(--font-display);
  font-weight: 800;
  font-size: 26px;
}
h1 {
  margin: 0;
  font-size: var(--fs-2xl);
  font-weight: 800;
}
.text p {
  margin: 2px 0 0;
  font-size: var(--fs-sm);
  display: flex;
  gap: 12px;
}
.site {
  text-decoration: underline;
  text-underline-offset: 3px;
}
.desc {
  color: var(--ink-2);
}
.block {
  margin-bottom: 24px;
}
.block h2 {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: var(--fs-md);
  margin: 0 0 10px;
}
.n {
  font-family: var(--font-display);
  color: var(--ink-3);
  font-weight: 600;
}
.missing {
  padding: 48px 24px;
  text-align: center;
}
.missing h1 {
  margin-bottom: 16px;
}
@media (max-width: 767px) {
  .head {
    grid-template-columns: auto minmax(0, 1fr);
  }
  .head .btn {
    grid-column: 1 / -1;
  }
  h1 {
    font-size: var(--fs-xl);
  }
}
</style>
