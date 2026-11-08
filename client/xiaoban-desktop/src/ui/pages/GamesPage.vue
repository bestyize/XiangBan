<script setup lang="ts">
import AppsPage from "./AppsPage.vue";
import type { ControlStore } from "../control-types.ts";

defineOptions({ name: 'GamesPage' });
defineProps<{ store: ControlStore }>();
</script>

<template>
  <AppsPage :store="store" />
</template>
