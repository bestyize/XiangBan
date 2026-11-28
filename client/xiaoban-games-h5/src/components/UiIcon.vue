<script setup lang="ts">
defineProps<{
  name: string;
  size?: number;
}>();
</script>

<template>
  <svg
    v-if="name === 'brand'"
    :width="size || 16"
    :height="size || 16"
    viewBox="0 0 24 24"
    fill="#ffffff"
    stroke="none"
    aria-hidden="true"
  >
    <path d="M12 2.5L14.2 9.2L21 11.4L14.2 13.6L12 20.5L9.8 13.6L3 11.4L9.8 9.2Z" />
    <circle cx="18.6" cy="5.4" r="2.1" />
  </svg>
  <svg
    v-else-if="name === 'play'"
    :width="size || 13"
    :height="size || 13"
    viewBox="0 0 24 24"
    fill="currentColor"
    stroke="none"
    aria-hidden="true"
  >
    <path d="M7 5.2v13.6a1 1 0 0 0 1.5.86l11-6.8a1 1 0 0 0 0-1.72l-11-6.8A1 1 0 0 0 7 5.2Z" />
  </svg>
  <svg
    v-else-if="name === 'moreV'"
    :width="size || 18"
    :height="size || 18"
    viewBox="0 0 24 24"
    fill="currentColor"
    stroke="none"
    aria-hidden="true"
  >
    <circle cx="12" cy="5.5" r="1.8" />
    <circle cx="12" cy="12" r="1.8" />
    <circle cx="12" cy="18.5" r="1.8" />
  </svg>
  <svg
    v-else-if="name === 'starFill'"
    :width="size || 18"
    :height="size || 18"
    viewBox="0 0 24 24"
    fill="currentColor"
    stroke="none"
    aria-hidden="true"
  >
    <path d="M12 3.2l2.8 5.7 6.3.9-4.5 4.4 1.1 6.2L12 17.5l-5.7 2.9 1.1-6.2-4.5-4.4 6.3-.9L12 3.2Z" />
  </svg>
  <svg
    v-else
    :width="size || 20"
    :height="size || 20"
    viewBox="0 0 24 24"
    aria-hidden="true"
  >
    <template v-if="name === 'gamepad'">
      <rect x="2.5" y="6.5" width="19" height="11" rx="5.5" />
      <path d="M8 10v4M6 12h4" />
      <circle cx="15.5" cy="11" r=".7" fill="currentColor" />
      <circle cx="17.5" cy="13" r=".7" fill="currentColor" />
    </template>
    <template v-else-if="name === 'trophy'">
      <path d="M8 21h8M12 17v4M7 4h10v5a5 5 0 0 1-10 0V4Z" />
      <path d="M7 6H4.5a1.5 1.5 0 0 0-1.5 1.5v1A3.5 3.5 0 0 0 6.5 12H7M17 6h2.5A1.5 1.5 0 0 1 21 7.5v1a3.5 3.5 0 0 1-3.5 3.5H17" />
    </template>
    <template v-else-if="name === 'user'">
      <circle cx="12" cy="8.5" r="3.8" />
      <path d="M4.8 19.5a7.2 7.2 0 0 1 14.4 0" />
    </template>
    <template v-else-if="name === 'moon'">
      <path d="M20.4 14.5A8.5 8.5 0 1 1 9.5 3.6a6.7 6.7 0 0 0 10.9 10.9Z" />
    </template>
    <template v-else-if="name === 'sun'">
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2.5v2.2M12 19.3v2.2M4.9 4.9l1.6 1.6M17.5 17.5l1.6 1.6M2.5 12h2.2M19.3 12h2.2M4.9 19.1l1.6-1.6M17.5 6.5l1.6-1.6" />
    </template>
    <template v-else-if="name === 'dice'">
      <rect x="3.5" y="3.5" width="17" height="17" rx="4" />
      <circle cx="8.5" cy="8.5" r="1" fill="currentColor" />
      <circle cx="15.5" cy="8.5" r="1" fill="currentColor" />
      <circle cx="12" cy="12" r="1" fill="currentColor" />
      <circle cx="8.5" cy="15.5" r="1" fill="currentColor" />
      <circle cx="15.5" cy="15.5" r="1" fill="currentColor" />
    </template>
    <template v-else-if="name === 'grid'">
      <rect x="3.5" y="3.5" width="7" height="7" rx="1.8" />
      <rect x="13.5" y="3.5" width="7" height="7" rx="1.8" />
      <rect x="3.5" y="13.5" width="7" height="7" rx="1.8" />
      <rect x="13.5" y="13.5" width="7" height="7" rx="1.8" />
    </template>
    <template v-else-if="name === 'list'">
      <path d="M8 6h12M8 12h12M8 18h12M4 6h.01M4 12h.01M4 18h.01" />
    </template>
    <template v-else-if="name === 'share'">
      <path d="M4 12v7a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-7" />
      <path d="M16 6l-4-4-4 4M12 2v13" />
    </template>
    <template v-else-if="name === 'refresh'">
      <path d="M20 11a8 8 0 1 0 2 5.3" />
      <path d="M20 4v7h-7" />
    </template>
    <template v-else-if="name === 'volume'">
      <path d="M11 5L6 9H3v6h3l5 4V5Z" />
      <path d="M15.5 8.5a5 5 0 0 1 0 7M18.8 5.8a9 9 0 0 1 0 12.4" />
    </template>
    <template v-else-if="name === 'volumeOff'">
      <path d="M11 5L6 9H3v6h3l5 4V5Z" />
      <path d="M16 9l5 6M21 9l-5 6" />
    </template>
    <template v-else-if="name === 'rotate'">
      <rect x="6.5" y="3.5" width="11" height="17" rx="2.2" />
      <path d="M11 17.5h2" />
    </template>
    <template v-else-if="name === 'back'">
      <path d="M15 18l-6-6 6-6" />
    </template>
    <template v-else-if="name === 'chevron'">
      <path d="M9 6l6 6-6 6" />
    </template>
    <template v-else-if="name === 'close'">
      <path d="M6 6l12 12M18 6L6 18" />
    </template>
    <template v-else-if="name === 'star'">
      <path d="M12 3.5l2.7 5.5 6 .9-4.3 4.2 1 6-5.4-2.8-5.4 2.8 1-6-4.3-4.2 6-.9L12 3.5Z" />
    </template>
    <template v-else-if="name === 'save'">
      <path d="M5 4h11l3 3v13a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V4Z" />
      <path d="M8 4v5h7V4M8 21v-6h8v6" />
    </template>
    <template v-else-if="name === 'palette'">
      <circle cx="12" cy="12" r="8.5" />
      <circle cx="9" cy="10" r="1" fill="currentColor" />
      <circle cx="13" cy="8.5" r="1" fill="currentColor" />
      <circle cx="15.5" cy="12" r="1" fill="currentColor" />
    </template>
    <template v-else-if="name === 'download'">
      <path d="M12 3v12M7.5 10.5L12 15l4.5-4.5M4 19.5h16" />
    </template>
    <template v-else-if="name === 'trash'">
      <path d="M4.5 7h15M9.5 4h5M7 7l.8 12.2a1.5 1.5 0 0 0 1.5 1.3h5.4a1.5 1.5 0 0 0 1.5-1.3L17 7" />
    </template>
    <template v-else-if="name === 'info'">
      <circle cx="12" cy="12" r="8.5" />
      <path d="M12 11v5M12 8h.01" />
    </template>
    <template v-else-if="name === 'spark'">
      <path d="M12 3l2 6 6 2-6 2-2 6-2-6-6-2 6-2 2-6Z" />
    </template>
    <template v-else-if="name === 'chat'">
      <path d="M5 18.5l-1.5 2.5 3.6-.8A8.8 8.8 0 1 0 5 18.5Z" />
      <path d="M8.5 11.5h7M8.5 14.5h4.5" />
    </template>
    <template v-else-if="name === 'arrowUpRight'">
      <path d="M7 17L17 7M9 7h8v8" />
    </template>
    <template v-else-if="name === 'phoneMini'">
      <rect x="6" y="2.5" width="12" height="19" rx="2.5" />
    </template>
    <template v-else-if="name === 'landMini'">
      <rect x="2.5" y="6" width="19" height="12" rx="2.5" />
    </template>
    <template v-else-if="name === 'eye'">
      <path d="M2.5 12s3.6-6.5 9.5-6.5S21.5 12 21.5 12s-3.6 6.5-9.5 6.5S2.5 12 2.5 12Z" />
      <circle cx="12" cy="12" r="2.6" />
    </template>
    <template v-else-if="name === 'eyeOff'">
      <path d="M3 3l18 18M10.6 10.6a2.5 2.5 0 0 0 2.8 2.8M6.5 6.8C4.1 8.5 2.5 12 2.5 12s3.6 6.5 9.5 6.5c1.8 0 3.4-.5 4.8-1.3M9.8 5.7c.7-.1 1.4-.2 2.2-.2 5.9 0 9.5 6.5 9.5 6.5a16.8 16.8 0 0 1-2.6 3.5" />
    </template>
    <template v-else-if="name === 'check'">
      <path d="M5 12.5l4.5 4.5L19 7.5" />
    </template>
  </svg>
</template>
