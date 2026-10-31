<script setup lang="ts">
import { invoke } from '../../bridge.ts';
const edges = ['n','s','e','w','ne','nw','se','sw'];
function resize(event: MouseEvent, direction: string) {
  if (event.button !== 0) return;
  event.preventDefault();
  void invoke('configuration_window_resize', { direction }).catch(() => {});
}
</script>
<template><div class="window-resize-edges" aria-hidden="true"><i v-for="edge in edges" :key="edge" :class="edge" @mousedown="resize($event, edge)"></i></div></template>
<style scoped>
.window-resize-edges{position:fixed;inset:0;pointer-events:none;z-index:1900}.window-resize-edges i{position:absolute;pointer-events:auto}
.n,.s{left:10px;right:10px;height:4px;cursor:ns-resize}.n{top:0}.s{bottom:0}
.e,.w{top:10px;bottom:10px;width:4px;cursor:ew-resize}.e{right:0}.w{left:0}
.ne,.nw,.se,.sw{width:10px;height:10px}.ne,.sw{cursor:nesw-resize}.nw,.se{cursor:nwse-resize}.ne{top:0;right:0}.nw{top:0;left:0}.se{bottom:0;right:0}.sw{bottom:0;left:0}
</style>
