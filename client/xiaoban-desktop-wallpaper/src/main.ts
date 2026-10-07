import './style.css'
import { EXPRESSIONS, EXPRESSION_NAMES } from './companion/expressions.ts'
import { desktop, defaults, invoke, listen, type CursorSample, type Settings } from './bridge.ts'
import { PetView } from './pet.ts'

const app = document.querySelector<HTMLElement>('#app')!
const mode = new URLSearchParams(location.search).get('scene')
let config = { ...defaults }
let pet: PetView
const sceneMarkup = `<div class="room"><div class="room-glow"></div><div class="room-window"><div class="moon"></div><div class="stars"></div><div class="skyline"></div></div><div class="plant"><i></i><i></i><i></i><i></i><b></b></div><div class="desk"></div><div class="room-floor"></div></div>`

function error(error: unknown) {
  const message = error instanceof Error ? error.message : String(error)
  const host = document.querySelector<HTMLElement>('#message')
  if (host) { host.textContent = message; host.dataset.error = 'true' }
  console.error(message)
}

function apply(settings: Settings) {
  config = settings
  document.body.dataset.theme = config.theme
  document.body.dataset.paused = String(config.paused)
  pet?.update(mode ? config : { ...config, size: 224 })
  if (!mode) {
    document.querySelectorAll<HTMLButtonElement>('[data-mode]').forEach(button => {
      const selected = button.dataset.mode === config.mode
      button.classList.toggle('selected', selected); button.setAttribute('aria-pressed', String(selected))
    })
    document.querySelector<HTMLSelectElement>('#expression')!.value = config.expression
    document.querySelector<HTMLInputElement>('#size')!.value = String(config.size)
    document.querySelector('#size-value')!.textContent = `${config.size}px`
    document.querySelector<HTMLSelectElement>('#theme')!.value = config.theme
    document.querySelector<HTMLSelectElement>('#fps')!.value = String(config.fps)
    document.querySelector('#pause')!.textContent = config.paused ? '继续陪伴' : '暂停动画'
    document.querySelector('#current-mode')!.textContent = config.mode === 'floating' ? '悬浮桌宠' : '动态壁纸'
  }
}

async function update(patch: Partial<Settings>) {
  const next = { ...config, ...patch }
  if (!desktop) { apply(next); return }
  try { apply(await invoke<Settings>('update_settings', { expression: next.expression, size: next.size,
    theme: next.theme, paused: next.paused, fps: next.fps })) }
  catch (failure) { apply(config); error(failure) }
}

async function init() {
  const platform = desktop ? await invoke<string>('get_platform') : 'preview'
  const platformLabel = platform === 'macos' ? 'MACOS' : platform === 'windows' ? 'WINDOWS' : '浏览器'
  if (mode) {
    document.body.className = mode
    app.innerHTML = mode === 'wallpaper'
      ? `<section class="scene full-scene">${sceneMarkup}<div class="wallpaper-pet"></div><div class="scene-caption"><span>小伴</span><p>陪你，把日子过慢一点。</p></div></section>`
      : '<section class="floating-stage"></section>'
    pet = new PetView(document.querySelector<HTMLElement>(mode === 'wallpaper' ? '.wallpaper-pet' : '.floating-stage')!, config)
    let pointer: { x: number; y: number; dragged: boolean } | null = null
    let hovering: boolean | null = null
    pet.host.addEventListener('pointerdown', event => {
      if (event.button !== 0 || mode !== 'floating' || !pet.hit(event.clientX, event.clientY)) return
      pointer = { x: event.clientX, y: event.clientY, dragged: false }
      pet.host.setPointerCapture(event.pointerId)
    })
    pet.host.addEventListener('pointermove', event => {
      if (!pointer || pointer.dragged) return
      if (Math.hypot(event.clientX - pointer.x, event.clientY - pointer.y) > 5) {
        pointer.dragged = true
        invoke('start_drag').catch(error)
      }
    })
    pet.host.addEventListener('pointerup', event => {
      if (pointer && !pointer.dragged && pet.hit(event.clientX, event.clientY)) pet.poke()
      pointer = null
    })
    pet.host.addEventListener('pointercancel', () => { pointer = null })
    pet.host.addEventListener('contextmenu', event => { event.preventDefault(); invoke('show_control').catch(error) })
    await listen<CursorSample>('desktop-cursor', sample => {
      pet.look(sample.x, sample.y, sample.moved)
      const hit = pet.hit(sample.x, sample.y)
      if (mode === 'floating' && hit !== hovering) {
        hovering = hit
        invoke('set_hit_test', { hit }).catch(error)
      }
      if (mode === 'wallpaper' && sample.pressed && sample.desktop && hit) pet.poke()
    })
  } else {
    document.body.className = 'control'
    app.innerHTML = `<div class="dashboard">
      <header class="header"><div class="brand"><span class="brand-icon">伴</span><div><h1>小伴</h1><p>桌面上的一点温暖</p></div></div><span class="prototype">${platformLabel} · 原型</span></header>
      <div class="workspace"><section class="preview-column"><div class="preview scene">${sceneMarkup}<div class="preview-pet"></div><div class="preview-label"><span class="live-dot"></span>实时互动预览</div></div>
        <div class="preview-note"><span id="pet-status">平静</span><p>戳一下，看看小伴的反应。<br>停下来，它也会慢慢打个盹。</p></div>
        <div class="detail"><span>原版 SVG</span><span>弹簧动画</span><span>鼠标跟随</span></div></section>
      <section class="settings-column"><div class="section-heading"><span class="eyebrow">陪伴方式</span><h2>让小伴住在你的桌面</h2></div>
        <div class="mode-switch"><button data-mode="floating"><span class="mode-symbol">◉</span><b>悬浮桌宠</b><small>随手拖动，陪在窗口旁</small></button><button data-mode="wallpaper"><span class="mode-symbol">▧</span><b>动态壁纸</b><small>住进背景，留在桌面里</small></button></div>
        <div class="field"><label for="expression">小伴的心情</label><select id="expression">${EXPRESSION_NAMES.map(name => `<option value="${name}">${EXPRESSIONS[name].label}</option>`).join('')}</select></div>
        <div class="field size-field"><label for="size">角色大小 <span id="size-value">260px</span></label><input id="size" type="range" min="160" max="420" step="10" value="260"><div class="range-label"><span>小巧</span><span>醒目</span></div></div>
        <div class="field-pair"><div class="field"><label for="theme">壁纸氛围</label><select id="theme"><option value="night">深夜暖灯</option><option value="dawn">晨光微亮</option></select></div><div class="field"><label for="fps">动画帧率</label><select id="fps"><option value="60">60 FPS · 流畅</option><option value="30">30 FPS · 节能</option></select></div></div>
        <div class="actions"><button id="poke" class="primary">戳戳小伴</button><button id="pause" class="secondary">暂停动画</button></div>
        <p id="message" role="status">${desktop ? '悬浮小伴已在桌面上，拖动可以换个位置。' : '这是浏览器预览，请启动桌面程序体验两种模式。'}</p>
        <div class="help"><span class="help-dot">i</span><p>悬浮时，透明处可点击穿透；右键小伴打开控制台。<br>壁纸时，桌面图标和应用窗口保持正常操作。</p></div>
      </section></div><footer><span><i></i>正在陪伴 · <b id="current-mode">悬浮桌宠</b></span><div><button id="diagnostics">运行状态</button><button id="hide">收起控制台</button><button id="quit">退出小伴</button></div></footer>
    </div>`
    pet = new PetView(document.querySelector<HTMLElement>('.preview-pet')!, { ...config, size: 224 }, state => {
      document.querySelector('#pet-status')!.textContent = EXPRESSIONS[state as keyof typeof EXPRESSIONS]?.label ?? state
    })
    pet.host.addEventListener('pointerdown', () => pet.poke())
    window.addEventListener('pointermove', event => pet.look(event.clientX, event.clientY))
    let changing = false
    document.querySelectorAll<HTMLButtonElement>('[data-mode]').forEach(button => button.addEventListener('click', async () => {
      if (changing || button.dataset.mode === config.mode) return
      if (!desktop) { error('请启动桌面程序体验此模式'); return }
      changing = true
      document.querySelectorAll<HTMLButtonElement>('[data-mode]').forEach(b => { b.disabled = true })
      try {
        apply(await invoke<Settings>('set_mode', { mode: button.dataset.mode }))
        const message = document.querySelector<HTMLElement>('#message')!
        message.dataset.error = 'false'
        message.textContent = config.mode === 'wallpaper' ? '小伴已住进桌面背景。收起控制台后查看壁纸。' : '悬浮小伴已回到桌面，拖动可以换个位置。'
      } catch (failure) { error(failure) }
      finally { changing = false; document.querySelectorAll<HTMLButtonElement>('[data-mode]').forEach(b => { b.disabled = false }) }
    }))
    document.querySelector<HTMLSelectElement>('#expression')!.addEventListener('change', event => update({ expression: (event.target as HTMLSelectElement).value }))
    document.querySelector<HTMLInputElement>('#size')!.addEventListener('input', event => { document.querySelector('#size-value')!.textContent = `${(event.target as HTMLInputElement).value}px` })
    document.querySelector<HTMLInputElement>('#size')!.addEventListener('change', event => update({ size: Number((event.target as HTMLInputElement).value) }))
    document.querySelector<HTMLSelectElement>('#theme')!.addEventListener('change', event => update({ theme: (event.target as HTMLSelectElement).value as Settings['theme'] }))
    document.querySelector<HTMLSelectElement>('#fps')!.addEventListener('change', event => update({ fps: Number((event.target as HTMLSelectElement).value) as 30 | 60 }))
    document.querySelector('#pause')!.addEventListener('click', () => update({ paused: !config.paused }))
    document.querySelector('#poke')!.addEventListener('click', () => { if (desktop) invoke('poke').catch(error); else pet.poke() })
    document.querySelector('#hide')!.addEventListener('click', () => invoke('minimize_control').catch(error))
    document.querySelector('#quit')!.addEventListener('click', () => invoke('quit').catch(error))
    document.querySelector('#diagnostics')!.addEventListener('click', async () => {
      try {
        const data = await invoke<{ platform: string; error?: string; wallpaper?: { parentClass?: string; host?: string; width: number; height: number }; floating?: { width: number; height: number } }>('get_diagnostics')
        if (data.error) throw new Error(data.error)
        const text = data.wallpaper ? `壁纸已嵌入 ${data.wallpaper.parentClass ?? data.wallpaper.host} · ${data.wallpaper.width} × ${data.wallpaper.height}` : `悬浮窗口 · ${data.floating?.width} × ${data.floating?.height}`
        const message = document.querySelector<HTMLElement>('#message')!; message.textContent = text; message.dataset.error = 'false'
      } catch (failure) { error(failure) }
    })
  }
  await listen<Settings>('settings-changed', apply)
  await listen('pet-poke', () => pet.poke())
  await listen<string>('desktop-error', error)
  if (desktop) apply(await invoke<Settings>('get_settings'))
  else apply(config)
  if (mode && desktop) await invoke('scene_ready')
}

init().catch(error)
window.addEventListener('beforeunload', () => pet?.destroy())
