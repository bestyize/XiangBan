<script setup lang="ts">
import { computed } from 'vue';
import { CHANGE, ITEM_STATUS, JOB_STATUS, STAGED_STATUS, type Tone } from '@/lib/labels';

const props = defineProps<{ type: 'job' | 'item' | 'staged' | 'change'; value: string | null | undefined }>();
const map = { job: JOB_STATUS, item: ITEM_STATUS, staged: STAGED_STATUS, change: CHANGE } as const;
const entry = computed<[string, Tone] | null>(() => (props.value ? map[props.type][props.value] ?? [props.value, 'idle'] : null));
</script>

<template>
  <span v-if="entry" class="badge" :class="`badge-${entry[1]}`">{{ entry[0] }}</span>
</template>
