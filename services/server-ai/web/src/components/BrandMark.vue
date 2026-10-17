<script setup lang="ts">
withDefaults(defineProps<{ size?: number }>(), { size: 32 });
</script>

<template>
  <svg :width="size" :height="size" viewBox="0 0 32 32" role="img" aria-label="相伴 AI">
    <rect width="32" height="32" rx="9" fill="#0d181d" />
    <rect x="0.5" y="0.5" width="31" height="31" rx="8.5" fill="none" stroke="#2c4a50" />
    <path d="M9.2 8.6h13.6l-1.5 14.1a2.3 2.3 0 0 1-2.3 2h-6a2.3 2.3 0 0 1-2.3-2z" fill="none" stroke="#dff5ef" stroke-width="1.6" stroke-linejoin="round" />
    <path d="M10.5 16.4c1.7-1.2 3.4 1.1 5.5 0s3.8 1.1 5.5 0l-.6 6.1a1.4 1.4 0 0 1-1.4 1.2h-7.6a1.4 1.4 0 0 1-1.4-1.2z" fill="#6fd8c2" />
  </svg>
</template>
