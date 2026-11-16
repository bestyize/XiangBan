<script setup lang="ts">
import CommunityPostFeed from '@/components/community/CommunityPostFeed.vue'
import { useHead } from '@/lib/head'
import '@/components/community/community.css'

useHead(() => ({ title: '讨论' }))
</script>

<template>
  <main class="community-page discuss-page">
    <h1 class="sr-only">社区讨论</h1>
    <CommunityPostFeed />
  </main>
</template>

<style scoped>
@media (min-width: 1100px) { .discuss-page { max-width: 1120px; } }
</style>
