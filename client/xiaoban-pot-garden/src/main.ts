import { Character } from './character.ts'
import { EXPRESSIONS, isExpression, LABELS, type Expression } from './poses.ts'
import { createStage } from './scene.ts'

const params = new URLSearchParams(location.search)
if (params.has('clean')) document.body.classList.add('clean')
const host = document.querySelector<HTMLElement>('#stage')!
const bar = document.querySelector<HTMLElement>('#bar')!

declare global {
  interface Window {
    __pot?: { ready: boolean; setExpression(expression: string): void; frame(): void }
  }
}

try {
  const stage = createStage(host)
  const character = new Character()
  stage.scene.add(character.group)

  const buttons = new Map<Expression, HTMLButtonElement>()
  const choose = (expression: Expression) => {
    character.setExpression(expression)
    for (const [name, button] of buttons) button.setAttribute('aria-pressed', String(name === expression))
  }
  for (const expression of EXPRESSIONS) {
    const button = document.createElement('button')
    button.type = 'button'
    button.textContent = LABELS[expression]
    button.addEventListener('click', () => choose(expression))
    buttons.set(expression, button)
    bar.append(button)
  }
  const initial = params.get('expr')
  choose(isExpression(initial) ? initial : 'idle')

  host.addEventListener('pointermove', event => {
    const rect = host.getBoundingClientRect()
    character.look(((event.clientX - rect.left) / rect.width) * 2 - 1, -(((event.clientY - rect.top) / rect.height) * 2 - 1))
  })
  host.addEventListener('pointerleave', () => character.look(0, 0))
  host.addEventListener('pointerdown', () => character.poke())

  const calm = matchMedia('(prefers-reduced-motion: reduce)').matches
  const fixed = params.has('t') ? Number(params.get('t')) : null
  let last = -1
  const frame = (now: number) => {
    const dt = last < 0 ? 0 : Math.max(0, Math.min(0.05, (now - last) / 1000))
    last = now
    character.update(dt, fixed ?? now / 1000, calm)
    stage.render()
  }
  if (fixed !== null) for (let i = 0; i < 90; i++) character.update(1 / 30, fixed)
  const loop = (now: number) => {
    requestAnimationFrame(loop)
    frame(now)
  }
  requestAnimationFrame(loop)
  window.__pot = {
    ready: true,
    setExpression: name => { if (isExpression(name)) choose(name) },
    frame: () => { for (let i = 0; i < 60; i++) character.update(1 / 30, fixed ?? performance.now() / 1000); stage.render() },
  }
} catch (error) {
  host.textContent = '这个设备暂时不能显示 3D 盆栽（需要 WebGL）。'
  console.error(error)
}
