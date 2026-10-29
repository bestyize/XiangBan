<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import type { WallpaperItem } from '@/types'
import { cardImage } from '@/utils'

const props = defineProps<{ item: WallpaperItem; src: string; videoSrc?: string }>()

const now = ref(new Date())
let timer: ReturnType<typeof setInterval> | undefined
onMounted(() => {
  timer = setInterval(() => (now.value = new Date()), 20_000)
})
onBeforeUnmount(() => clearInterval(timer))
const kind = computed(() => (props.item.orientation === 'portrait' ? 'phone' : 'desktop'))
const clock = computed(() =>
  new Intl.DateTimeFormat('zh-CN', { hour: '2-digit', minute: '2-digit', hour12: false }).format(now.value),
)
const date = computed(() =>
  new Intl.DateTimeFormat('zh-CN', { month: 'long', day: 'numeric', weekday: 'long' }).format(now.value),
)
</script>

<template>
  <div class="wrap">
    <div class="frame" :class="kind" aria-label="设备效果预览">
      <video v-if="videoSrc" :src="videoSrc" :poster="cardImage(item)" autoplay muted loop playsinline />
      <img v-else :src="src" :alt="item.title" draggable="false" />

      <template v-if="kind === 'phone'">
        <div class="island" />
        <div class="lock">
          <span class="date">{{ date }}</span>
          <span class="time">{{ clock }}</span>
        </div>
        <div class="quick">
          <span /><span />
        </div>
      </template>

      <template v-else>
        <div class="menubar"><span class="grow" /><span>{{ clock }}</span></div>
        <div class="dock"><i v-for="n in 6" :key="n" /></div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.wrap {
  display: grid;
  place-items: center;
  width: 100%;
  height: 100%;
  padding: 72px 20px 24px;
}

.frame {
  position: relative;
  overflow: hidden;
  container-type: size;
  background: #000;
  box-shadow:
    0 0 0 2px #2c2c31,
    0 0 0 5px #121214,
    0 30px 80px rgba(0, 0, 0, 0.55);
}

.frame.phone {
  height: 100%;
  max-width: 100%;
  aspect-ratio: 9 / 19.5;
  border-radius: 9cqw;
}

.frame.desktop {
  width: 100%;
  max-height: 100%;
  aspect-ratio: 16 / 10;
  border-radius: 14px;
}

img,
video {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.island {
  position: absolute;
  top: 3cqw;
  left: 50%;
  width: 28cqw;
  height: 8cqw;
  border-radius: 999px;
  background: #000;
  transform: translateX(-50%);
}

.lock {
  position: absolute;
  inset: 17cqh 0 auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  color: #fff;
  text-shadow: 0 2px 18px rgba(0, 0, 0, 0.35);
}

.date {
  font-size: 4.4cqw;
  font-weight: 500;
}

.time {
  font-size: 24cqw;
  font-weight: 300;
  line-height: 1.05;
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.02em;
}

.quick {
  position: absolute;
  inset: auto 9cqw 6cqw;
  display: flex;
  justify-content: space-between;
}

.quick span {
  width: 13cqw;
  height: 13cqw;
  border-radius: 50%;
  background: rgba(30, 30, 34, 0.45);
  -webkit-backdrop-filter: blur(14px);
  backdrop-filter: blur(14px);
}

.menubar {
  position: absolute;
  inset: 0 0 auto;
  display: flex;
  height: 2.6cqw;
  min-height: 18px;
  padding: 0 1.4cqw;
  align-items: center;
  background: rgba(20, 20, 24, 0.35);
  -webkit-backdrop-filter: blur(16px);
  backdrop-filter: blur(16px);
  color: #fff;
  font-size: max(1.35cqw, 10px);
  font-variant-numeric: tabular-nums;
}

.grow {
  flex: 1;
}

.dock {
  position: absolute;
  inset: auto 50% 1.6cqw auto;
  display: flex;
  gap: 1cqw;
  padding: 0.9cqw 1.2cqw;
  border-radius: 2.2cqw;
  background: rgba(30, 30, 34, 0.38);
  -webkit-backdrop-filter: blur(18px);
  backdrop-filter: blur(18px);
  transform: translateX(50%);
}

.dock i {
  width: 4.2cqw;
  height: 4.2cqw;
  border-radius: 1.1cqw;
  background: rgba(241, 239, 234, 0.3);
}
</style>
