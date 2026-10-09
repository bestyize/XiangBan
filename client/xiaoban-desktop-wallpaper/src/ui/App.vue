<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NDialogProvider, zhCN, dateZhCN } from 'naive-ui';
import ControlShell from './ControlShell.vue';
import { theme } from './theme.ts';
import './control.css';
</script>
<template><NConfigProvider :theme-overrides="theme" :locale="zhCN" :date-locale="dateZhCN"><NMessageProvider placement="top-right"><NDialogProvider><ControlShell /></NDialogProvider></NMessageProvider></NConfigProvider></template>
