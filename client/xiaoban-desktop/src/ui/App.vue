<script setup lang="ts">
import { computed, watchEffect } from 'vue';
import { NConfigProvider, NMessageProvider, NDialogProvider, darkTheme, zhCN, dateZhCN } from 'naive-ui';
import ControlShell from './ControlShell.vue';
import { themeFor } from './theme.ts';
import { applyUiTokens } from './design-tokens.ts';
import { accent, isDark } from './appearance.ts';
import './control.css';
const naiveTheme = computed(() => isDark.value ? darkTheme : null);
const overrides = computed(() => themeFor(isDark.value, accent.value));
watchEffect(() => { applyUiTokens(isDark.value, accent.value); document.documentElement.dataset.theme = isDark.value ? 'dark' : 'light'; });
</script>
<template><NConfigProvider :theme="naiveTheme" :theme-overrides="overrides" :locale="zhCN" :date-locale="dateZhCN"><NMessageProvider placement="top-right"><NDialogProvider><ControlShell /></NDialogProvider></NMessageProvider></NConfigProvider></template>
