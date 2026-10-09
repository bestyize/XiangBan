<script setup lang="ts">
import {
  onActivated,
  onBeforeUnmount,
  onDeactivated,
  onMounted,
  ref,
} from "vue";
import InteractiveXiaoban from "./InteractiveXiaoban.vue";
const mode = ref("wallpaper"),
  playing = ref(false),
  video = ref<HTMLVideoElement | null>(null);
const cursor = ref<{ x: number; y: number } | null>(null),
  mood = ref("平静"),
  message = ref("移动鼠标、戳戳小伴，或把它拖到喜欢的位置。");
const startOpen = ref(false),
  explorerOpen = ref(false),
  selected = ref(""),
  clock = ref(""),
  date = ref("");
let timer: ReturnType<typeof setInterval> | undefined;
function tick() {
  const now = new Date();
  clock.value = now.toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
  date.value = now.toLocaleDateString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
  });
}
function pause() {
  video.value?.pause();
}
function toggle() {
  const player = video.value;
  if (!player) return;
  if (player.paused)
    void player.play().catch(() => {
      message.value = "点击播放，让桌面的风景动起来。";
    });
  else player.pause();
}
onMounted(() => {
  tick();
  timer = setInterval(tick, 30000);
});
onActivated(() => {
  void video.value?.play().catch(() => {});
});
onDeactivated(pause);
onBeforeUnmount(() => {
  pause();
  clearInterval(timer);
});
function pointer(event: PointerEvent) {
  cursor.value = { x: event.clientX, y: event.clientY };
}
function openExplorer(name: string) {
  selected.value = name;
  explorerOpen.value = true;
  startOpen.value = false;
}
</script>
<template>
  <div class="desktop-demo">
    <div class="demo-modes" aria-label="陪伴方式演示">
      <button
        :aria-pressed="mode === 'floating'"
        :class="{ active: mode === 'floating' }"
        @click="mode = 'floating'"
      >
        悬浮桌宠</button
      ><button
        :aria-pressed="mode === 'wallpaper'"
        :class="{ active: mode === 'wallpaper' }"
        @click="mode = 'wallpaper'"
      >
        融入动态壁纸
      </button>
    </div>
    <div class="desktop-frame">
      <div class="window-title">
        <span><img src="/xiaoban.svg" alt="" />小伴桌面 · 互动演示</span>
        <div aria-hidden="true">—　▢　×</div>
      </div>
      <div
        class="windows-scene"
        :class="mode"
        @pointermove="pointer"
        @pointerleave="cursor = null"
      >
        <video
          ref="video"
          class="desktop-video"
          src="/xiaoban-demo.mp4"
          poster="/xiaoban-demo.jpg"
          autoplay
          muted
          loop
          playsinline
          @playing="playing = true"
          @pause="playing = false"
        />
        <div class="desktop-shortcuts" aria-label="演示桌面图标">
          <button
            :class="{ selected: selected === '此电脑' }"
            @click="selected = '此电脑'"
            @dblclick="openExplorer('此电脑')"
          >
            <span class="monitor-icon"></span><small>此电脑</small>
          </button>
          <button
            :class="{ selected: selected === '我的壁纸' }"
            @click="selected = '我的壁纸'"
            @dblclick="openExplorer('我的壁纸')"
          >
            <span class="folder-icon"></span><small>我的壁纸</small>
          </button>
          <button
            :class="{ selected: selected === '小伴桌面' }"
            @click="
              selected = '小伴桌面';
              message = '戳一下猫脸，看看小伴的回应。';
            "
          >
            <img src="/xiaoban.svg" alt="" /><small>小伴桌面</small>
          </button>
          <button
            :class="{ selected: selected === '回收站' }"
            @click="selected = '回收站'"
          >
            <span class="bin-icon">♻</span><small>回收站</small>
          </button>
        </div>
        <div v-if="explorerOpen" class="explorer">
          <header>
            <span class="folder-icon"></span
            ><b>{{ selected === "此电脑" ? "此电脑" : "我的壁纸" }}</b
            ><button aria-label="关闭文件窗口" @click="explorerOpen = false">
              ×
            </button>
          </header>
          <div class="explorer-address">⌂　桌面　›　我的壁纸</div>
          <div class="explorer-content">
            <aside>快速访问<br />桌面<br />下载<br />图片</aside>
            <div>
              <img src="/xiaoban-demo.jpg" alt="晴海壁纸" /><span
                >晴海 · 动态壁纸</span
              ><small>下载后，也可以离线陪伴。</small>
            </div>
          </div>
        </div>
        <InteractiveXiaoban
          :floating="mode === 'floating'"
          :cursor="cursor"
          @mood="mood = $event"
          @interaction="message = $event"
        />
        <div v-if="startOpen" class="start-menu">
          <div class="start-search">⌕　搜索应用、设置和文件</div>
          <b>已固定</b>
          <div class="start-apps">
            <button
              @click="
                startOpen = false;
                message = '欢迎使用小伴，点击桌上的猫脸试试。';
              "
            >
              <img src="/xiaoban.svg" alt="" />小伴桌面</button
            ><button @click="openExplorer('我的壁纸')">
              <span class="folder-icon"></span>文件管理器</button
            ><button
              @click="
                toggle();
                startOpen = false;
              "
            >
              <span class="play-icon">▶</span>动态壁纸
            </button>
          </div>
          <footer><span>◉　桌面小伙伴</span><span>⏻</span></footer>
        </div>
        <div class="scene-mood">小伴：{{ mood }}</div>
        <div class="taskbar">
          <span class="weather">☀<small>晴朗</small></span>
          <div class="pinned-apps">
            <button
              class="start-button"
              aria-label="打开开始菜单"
              :aria-expanded="startOpen"
              @click="startOpen = !startOpen"
            >
              <span></span><span></span><span></span><span></span></button
            ><button
              class="task-search"
              aria-label="演示搜索"
              @click="startOpen = !startOpen"
            >
              ⌕<span>搜索</span></button
            ><button
              aria-label="打开我的壁纸"
              @click="openExplorer('我的壁纸')"
            >
              <span class="folder-icon"></span></button
            ><button
              class="task-pet"
              aria-label="查看小伴互动提示"
              @click="message = '小伴会跟着鼠标看，也会回应你的点击。'"
            >
              <img src="/xiaoban.svg" alt="" />
            </button>
          </div>
          <div class="system-tray">
            <span aria-hidden="true">⌁　◖</span
            ><span
              >{{ clock }}<small>{{ date }}</small></span
            >
          </div>
        </div>
      </div>
    </div>
    <div class="demo-controls">
      <span class="interaction-message" role="status">{{ message }}</span
      ><button @click="toggle">
        {{ playing ? "暂停动态壁纸" : "播放动态壁纸" }}
      </button>
    </div>
    <p class="demo-note">Windows 风格演示 · 视频静音循环 · 双击文件夹试试</p>
  </div>
</template>
<style scoped>
.desktop-demo {
  min-width: 0;
}
.demo-modes {
  display: flex;
  justify-content: center;
  gap: 8px;
  margin-bottom: 18px;
}
.demo-modes button {
  padding: 9px 18px;
  border: 1px solid var(--line);
  border-radius: 999px;
  font-size: 13px;
  color: var(--text-2);
  transition:
    background 0.25s,
    color 0.25s;
}
.demo-modes .active {
  background: var(--text);
  color: var(--bg);
  border-color: transparent;
}
.desktop-frame {
  border: 1px solid #d7e1ec;
  border-radius: 12px;
  overflow: hidden;
  box-shadow:
    0 24px 70px #49739c28,
    0 4px 14px #41618a0d;
}
.window-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 9px 14px;
  background: #f6f9fd;
  color: #526277;
  font-size: 11px;
  border-bottom: 1px solid #e0e7ef;
}
.window-title > span {
  display: flex;
  align-items: center;
  gap: 7px;
}
.window-title img {
  width: 22px;
  height: 20px;
}
.windows-scene {
  aspect-ratio: 16/9;
  container-type: inline-size;
  position: relative;
  overflow: hidden;
  background: #b5d8ec;
}
.desktop-video {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.desktop-shortcuts {
  position: absolute;
  left: 10px;
  top: 12px;
  z-index: 4;
  display: grid;
  gap: 5px;
}
.desktop-shortcuts button {
  width: 62px;
  height: 67px;
  border: 1px solid transparent;
  border-radius: 4px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 5px;
  color: #2e4963;
  text-shadow: 0 1px 2px #fff8;
}
.desktop-shortcuts button.selected,
.desktop-shortcuts button:hover {
  border-color: #ffffff95;
  background: #ffffff40;
}
.desktop-shortcuts img {
  width: 37px;
  height: 33px;
}
.desktop-shortcuts small {
  font-size: 10px;
}
.monitor-icon {
  width: 31px;
  height: 23px;
  border: 2px solid #d5e5f3;
  border-radius: 3px;
  background: linear-gradient(140deg, #2365c2, #86bdea);
  box-shadow: 0 2px 4px #38577544;
  position: relative;
}
.monitor-icon:after {
  content: "";
  position: absolute;
  width: 19px;
  height: 3px;
  background: #e2eaf5;
  bottom: -6px;
  left: 4px;
}
.folder-icon {
  display: inline-block;
  position: relative;
  width: 27px;
  height: 19px;
  background: linear-gradient(#ffdf78, #eab74f);
  border-radius: 2px;
  box-shadow: 0 2px 3px #93733220;
}
.folder-icon:before {
  content: "";
  position: absolute;
  top: -4px;
  left: 0;
  width: 12px;
  height: 6px;
  background: #f2c858;
  border-radius: 3px 3px 0 0;
}
.bin-icon {
  font-size: 28px;
  color: #6d97ac;
  background: #eaf5f6a8;
  padding: 0 4px;
  border-radius: 3px;
}
.taskbar {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 10;
  height: 45px;
  background: #f4f8fbd9;
  backdrop-filter: blur(18px);
  border-top: 1px solid #fff8;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 12px;
  color: #40566d;
  font-size: 11px;
}
.weather {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 20px;
  min-width: 50px;
}
.weather small {
  font-size: 9px;
}
.pinned-apps {
  display: flex;
  align-items: center;
  gap: 6px;
}
.pinned-apps > button {
  height: 34px;
  min-width: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
}
.pinned-apps > button:hover {
  background: #fff8;
}
.pinned-apps .start-button {
  display: grid;
  grid-template-columns: repeat(2, 9px);
  grid-template-rows: repeat(2, 9px);
  gap: 2px;
  justify-content: center;
  align-content: center;
}
.start-button span {
  background: #1683d3;
  border-radius: 1px;
}
.pinned-apps .task-search {
  gap: 5px;
  padding: 0 10px;
  background: #ffffffad;
  border: 1px solid #d4e1e9;
  border-radius: 18px;
  font-size: 19px;
  min-width: 85px;
  height: 28px;
}
.task-search span {
  font-size: 10px;
}
.task-pet img {
  width: 28px;
  height: 27px;
}
.system-tray {
  display: flex;
  align-items: center;
  gap: 8px;
  text-align: right;
  font-size: 9px;
  line-height: 1.4;
  min-width: 65px;
}
.system-tray small {
  display: block;
}
.scene-mood {
  position: absolute;
  left: 85px;
  bottom: 58px;
  background: #ffffff85;
  color: #42617d;
  padding: 5px 9px;
  border-radius: 20px;
  font-size: 10px;
  z-index: 4;
  pointer-events: none;
}
.start-menu {
  position: absolute;
  z-index: 9;
  bottom: 54px;
  left: 50%;
  transform: translateX(-50%);
  width: 290px;
  padding: 20px;
  background: #f4f8fcf2;
  backdrop-filter: blur(22px);
  border: 1px solid #fff;
  border-radius: 9px;
  box-shadow: 0 12px 50px #35567830;
  color: #4b6074;
  font-size: 11px;
}
.start-search {
  padding: 7px 10px;
  border: 1px solid #cadbe8;
  border-bottom: 2px solid #4a95d2;
  background: #fff;
  border-radius: 5px;
  margin-bottom: 20px;
  font-size: 10px;
}
.start-menu > b {
  font-size: 10px;
}
.start-apps {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
  margin: 15px 0 25px;
}
.start-apps button {
  font-size: 9px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  border-radius: 5px;
}
.start-apps button:hover {
  background: #fff;
}
.start-apps img {
  width: 34px;
  height: 30px;
}
.play-icon {
  font-size: 22px;
  color: #318ec2;
}
.start-menu footer {
  display: flex;
  justify-content: space-between;
  border-top: 1px solid #dfe7ee;
  padding-top: 12px;
  font-size: 10px;
}
.explorer {
  position: absolute;
  left: 21%;
  top: 9%;
  width: 72%;
  z-index: 5;
  background: #f7fafdeb;
  border: 1px solid #e0e7ef;
  border-radius: 8px;
  box-shadow: 0 15px 42px #375a7b38;
  color: #456079;
  font-size: 10px;
  overflow: hidden;
}
.explorer header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background: #eef3f8;
}
.explorer header .folder-icon {
  width: 14px;
  height: 10px;
}
.explorer header .folder-icon:before {
  width: 7px;
  height: 4px;
  top: -2px;
}
.explorer header b {
  font-weight: 500;
}
.explorer header button {
  margin-left: auto;
  font-size: 17px;
  line-height: 1;
}
.explorer-address {
  margin: 9px 12px;
  padding: 6px;
  background: #fff;
  border: 1px solid #dbe5ee;
  border-radius: 4px;
  font-size: 9px;
}
.explorer-content {
  display: grid;
  grid-template-columns: 80px 1fr;
  min-height: 132px;
}
.explorer-content aside {
  padding: 10px 12px;
  background: #edf3f9;
  line-height: 2.2;
  color: #7790a4;
  font-size: 9px;
}
.explorer-content > div {
  padding: 12px;
  display: flex;
  align-items: flex-start;
  flex-direction: column;
  gap: 5px;
}
.explorer-content img {
  width: 96px;
  border-radius: 4px;
  border: 2px solid #fff;
}
.explorer-content small {
  font-size: 8px;
  color: #90a2b2;
}
.demo-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-top: 16px;
}
.interaction-message {
  font-size: 12px;
  color: var(--text-2);
  flex: 1;
}
.demo-controls > button {
  flex: none;
  padding: 7px 12px;
  border: 1px solid var(--line);
  border-radius: 20px;
  color: var(--text-2);
  font-size: 11px;
}
.demo-note {
  margin-top: 10px;
  font-size: 11px;
  color: var(--text-3);
  text-align: center;
}
@media (max-width: 768px) {
  .windows-scene {
    aspect-ratio: 16/9;
  }
  .demo-controls {
    align-items: flex-start;
  }
  .interaction-message {
    font-size: 11px;
    line-height: 1.8;
  }
  .task-search {
    min-width: 34px !important;
    padding: 0 6px !important;
  }
  .task-search span {
    display: none;
  }
  .weather {
    min-width: 30px;
  }
  .weather small {
    display: none;
  }
  .start-menu {
    width: 265px;
  }
}
</style>

<style scoped>
@media (max-width: 500px) {
  .desktop-shortcuts {
    gap: 1px;
    top: 8px;
    left: 6px;
  }
  .desktop-shortcuts button {
    width: 43px;
    height: 30px;
    gap: 1px;
  }
  .desktop-shortcuts small {
    font-size: 7px;
  }
  .desktop-shortcuts img {
    width: 19px;
    height: 17px;
  }
  .desktop-shortcuts .monitor-icon {
    width: 23px;
    height: 15px;
  }
  .desktop-shortcuts .folder-icon {
    width: 22px;
    height: 14px;
  }
  .bin-icon {
    font-size: 20px;
  }
  .taskbar {
    height: 32px;
    padding: 0 8px;
  }
  .pinned-apps > button {
    height: 28px;
    min-width: 27px;
  }
  .task-pet img {
    width: 23px;
    height: 23px;
  }
  .taskbar .folder-icon {
    width: 23px;
    height: 16px;
  }
  .system-tray {
    font-size: 7px;
    min-width: 50px;
    gap: 5px;
  }
  .scene-mood {
    bottom: 39px;
    left: 58px;
    font-size: 8px;
    padding: 3px 7px;
  }
  .start-menu {
    bottom: 39px;
    width: 240px;
    padding: 12px;
  }
  .start-search {
    margin-bottom: 12px;
  }
  .start-apps {
    margin: 8px 0 12px;
  }
  .explorer {
    top: 4%;
    left: 17%;
    width: 78%;
  }
  .explorer-content {
    min-height: 92px;
    grid-template-columns: 58px 1fr;
  }
  .explorer-content aside {
    padding: 6px;
    font-size: 7px;
  }
  .explorer-content > div {
    padding: 6px;
  }
  .explorer-content img {
    width: 64px;
  }
  .explorer header {
    padding: 7px;
  }
  .explorer-address {
    margin: 5px 7px;
    padding: 4px;
  }
  .explorer-content small {
    display: none;
  }
}
</style>
