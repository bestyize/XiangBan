<script setup lang="ts">
import {
  ArrowUpRight, Check, ChevronLeft, ChevronRight, CircleAlert, CircleCheck, Clock, Copy, Download, Eye, EyeOff, Gift,
  Hourglass, Info, KeyRound, Layers, LayoutDashboard, Lock, LogOut, Pencil, Plug, Plus, RefreshCw, ScrollText, Search,
  ShieldCheck, Ticket, Trash2, TriangleAlert, Undo2, Users, Wallet, X, Zap,
} from '@lucide/vue';

const icons = {
  'arrow-up-right': ArrowUpRight, check: Check, 'chevron-left': ChevronLeft, 'chevron-right': ChevronRight,
  'circle-alert': CircleAlert, 'circle-check': CircleCheck, clock: Clock, copy: Copy, download: Download, eye: Eye,
  'eye-off': EyeOff, gift: Gift, hourglass: Hourglass, info: Info, key: KeyRound, layers: Layers, dashboard: LayoutDashboard,
  lock: Lock, 'log-out': LogOut, pencil: Pencil, plug: Plug, plus: Plus, refresh: RefreshCw, scroll: ScrollText,
  search: Search, shield: ShieldCheck, ticket: Ticket, trash: Trash2, alert: TriangleAlert, undo: Undo2, users: Users,
  wallet: Wallet, x: X, zap: Zap,
} as const;

export type IconName = keyof typeof icons;

withDefaults(defineProps<{ name: IconName; size?: number; stroke?: number }>(), { size: 18, stroke: 1.8 });
</script>

<template>
  <component :is="icons[name]" :size="size" :stroke-width="stroke" aria-hidden="true" focusable="false" />
</template>
