<script setup lang="ts">
import CenterDialog from './CenterDialog.vue'
import UiButton from '../controls/UiButton.vue'
import UiIcon from '../icons/UiIcon.vue'
import { settleConfirm, useConfirmState } from './confirm'

/**
 * 全局确认弹窗宿主：挂在 App.vue 根节点，响应 confirmDialog() 调用。
 * 双端（PC 与 H5）均以 24px 圆角居中卡片呈现，主操作 48px 全圆胶囊，取消 44px 无底色灰字。
 */
const active = useConfirmState()
</script>

<template>
  <CenterDialog
    :open="Boolean(active)"
    :title="active?.title"
    :width="368"
    @close="settleConfirm(false)"
  >
    <template v-if="active">
      <div class="cf-head">
        <div class="cf-tile" :class="{ 'is-danger': active.danger }" aria-hidden="true">
          <UiIcon :name="active.danger ? 'warning' : 'info'" :size="22" :stroke-width="1.9" />
        </div>
        <div class="cf-copy">
          <h3>{{ active.title }}</h3>
          <p v-if="active.description">{{ active.description }}</p>
        </div>
      </div>
      <blockquote v-if="active.quote" class="cf-quote">{{ active.quote }}</blockquote>
      <p v-if="active.notice" class="cf-notice">{{ active.notice }}</p>
    </template>
    <template #footer>
      <UiButton
        v-if="active"
        :variant="active.danger ? 'danger' : 'primary'"
        size="lg"
        block
        @click="settleConfirm(true)"
      >
        {{ active.confirmLabel || (active.danger ? '确认' : '确定') }}
      </UiButton>
      <UiButton
        v-if="active"
        variant="ghost"
        size="md"
        block
        @click="settleConfirm(false)"
      >
        {{ active.cancelLabel || '取消' }}
      </UiButton>
    </template>
  </CenterDialog>
</template>

<style scoped>
.cf-head {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding-right: 32px;
}
.cf-tile {
  width: 24px;
  height: 24px;
  flex: none;
  display: grid;
  place-items: center;
  color: var(--ink);
}
.cf-tile.is-danger {
  color: var(--ui-danger);
}
.cf-tile svg {
  width: 22px;
  height: 22px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.9;
  stroke-linecap: round;
  stroke-linejoin: round;
}
.cf-copy h3 {
  margin: 0;
  font-size: var(--fs-title);
  line-height: 24px;
  font-weight: 650;
  overflow-wrap: anywhere;
}
.cf-copy p {
  margin: 6px 0 0;
  font-size: var(--fs-secondary);
  line-height: 20px;
  color: var(--ink-2);
  overflow-wrap: anywhere;
}
.cf-quote {
  margin: 14px 0 0;
  padding: 10px 12px;
  border-radius: var(--r-md);
  background: var(--surface-2);
  font-size: var(--fs-secondary);
  line-height: 19px;
  color: var(--ink);
}
.cf-notice {
  margin: 12px 0 0;
  padding: 10px 12px;
  border-radius: var(--r-md);
  background: var(--surface-2);
  font-size: var(--fs-caption);
  line-height: 18px;
  color: var(--ink-2);
}
</style>
