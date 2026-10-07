import { useMemo, useRef, useState } from 'react'
import { Robot, type RobotHandle } from '../../components/Robot.tsx'
import { EXPRESSION_NAMES, EXPRESSIONS, resolve, type ExpressionName } from '../../robot/expressions.ts'
import type { DeepPartial, Face, SymbolName } from '../../robot/params.ts'
import { SYMBOL_NAMES } from '../../robot/symbols.ts'

type Slider = { key: string; label: string; min: number; max: number; step: number; get: (f: Face) => number; set: (v: number) => DeepPartial<Face> }

const eye = (key: keyof Face['eyeL'], label: string, min: number, max: number, step = 0.01): Slider => ({
  key: `eye.${key}`, label, min, max, step,
  get: (f) => f.eyeL[key],
  set: (v) => ({ eyeL: { [key]: v }, eyeR: { [key]: v } }),
})
const mouth = (key: keyof Face['mouth'], label: string, min: number, max: number, step = 0.1): Slider => ({
  key: `mouth.${key}`, label, min, max, step, get: (f) => f.mouth[key], set: (v) => ({ mouth: { [key]: v } }),
})
const top = (key: 'blush' | 'blushLines' | 'tears' | 'tilt' | 'screen' | 'earGlow' | 'gazeX' | 'gazeY' | 'earL' | 'earR', label: string, min: number, max: number, step = 0.01): Slider => ({
  key, label, min, max, step, get: (f) => f[key], set: (v) => ({ [key]: v }),
})

const SLIDERS: Slider[] = [
  eye('w', '眼宽', 4, 32, 0.5), eye('h', '眼高', 0, 32, 0.5), eye('thick', '环厚 → 实心', 0, 1), eye('span', '弧长', 0, 1),
  eye('arcMid', '弧朝向', -180, 180, 1), eye('lidTop', '上眼睑', -1.4, 1), eye('lidTilt', '眼睑倾斜', -1, 1), eye('lidBottom', '下眼睑', -0.5, 1.4),
  eye('pupil', '瞳孔', 0, 0.9), eye('pupilX', '瞳孔 X', -0.8, 0.8), eye('pupilY', '瞳孔 Y', -0.8, 0.8), eye('dash', '虚线', 0, 1),
  mouth('w', '嘴宽', 1, 16), mouth('curve', '嘴弯', -6, 6), mouth('cat', 'ω', -2, 7), mouth('open', '张嘴', 0, 16), mouth('tongue', '舌头', 0, 1, 0.01),
  top('blush', '腮红', 0, 1), top('blushLines', '害羞线', 0, 1), top('tears', '眼泪', 0, 1),
  top('gazeX', '视线 X', -14, 14, 0.5), top('gazeY', '视线 Y', -8, 8, 0.5), top('tilt', '歪头', -15, 15, 0.5),
  top('earL', '左耳', -20, 20, 0.5), top('earR', '右耳', -20, 20, 0.5), top('earGlow', '耳灯', 0, 1), top('screen', '屏幕亮度', 0, 1),
]

function merge(a: DeepPartial<Face>, b: DeepPartial<Face>): DeepPartial<Face> {
  return {
    ...a, ...b,
    eyeL: { ...a.eyeL, ...b.eyeL }, eyeR: { ...a.eyeR, ...b.eyeR }, mouth: { ...a.mouth, ...b.mouth },
  }
}

const DEMO: { expr: ExpressionName; hold: number }[] = [
  { expr: 'sleeping', hold: 1.6 }, { expr: 'surprised', hold: 0.6 }, { expr: 'curious', hold: 1.2 },
  { expr: 'greeting', hold: 1.4 }, { expr: 'speaking', hold: 1.6 }, { expr: 'thinking', hold: 1.3 },
  { expr: 'excited', hold: 1.1 }, { expr: 'laughing', hold: 1.2 }, { expr: 'shy', hold: 1.2 },
  { expr: 'worried', hold: 1.1 }, { expr: 'crying', hold: 1.4 }, { expr: 'love', hold: 1.3 }, { expr: 'charging', hold: 1.3 },
]

export default function RobotLab() {
  const robot = useRef<RobotHandle>(null)
  const [expr, setExpr] = useState<ExpressionName>('idle')
  const [symbol, setSymbol] = useState<SymbolName | null | undefined>(undefined)
  const [overrides, setOverrides] = useState<DeepPartial<Face>>({})
  const [flags, setFlags] = useState({ speaking: false, thinking: false, listening: false })
  const [dark, setDark] = useState(false)
  const [tilesAlive, setTilesAlive] = useState(true)

  const spec = useMemo(() => ({ expression: expr, symbol, face: overrides }), [expr, symbol, overrides])
  const resolved = useMemo(() => resolve(spec).face, [spec])

  const bg = dark ? 'bg-[#182338] text-[#f3eee4]' : 'bg-[#f4eee6] text-[#2b2622]'
  const card = dark ? 'bg-[#223149]' : 'bg-[#ebe3d8]'

  return (
    <main className={`min-h-dvh ${bg} px-6 py-8`}>
      <header className="mx-auto flex max-w-6xl flex-wrap items-end justify-between gap-4">
        <div>
          <h1 className="text-3xl font-black tracking-tight">伴侣机器人表情系统 <span className="font-medium opacity-80">/ Companion Robot Expression System</span></h1>
          <p className="mt-1 text-xs tracking-[0.2em] opacity-60">参数化 · 状态流转 · 点它、晃鼠标、让它说话</p>
        </div>
        <div className="flex gap-2 text-sm">
          <button type="button" className={`rounded-full px-4 py-2 ${card}`} onClick={() => setDark(!dark)}>{dark ? '浅色背景' : '深色背景'}</button>
          <button type="button" className={`rounded-full px-4 py-2 ${card}`} onClick={() => setTilesAlive(!tilesAlive)}>{tilesAlive ? '格子静止' : '格子动起来'}</button>
        </div>
      </header>

      <section className="mx-auto mt-6 grid max-w-6xl gap-6 lg:grid-cols-[420px_1fr]">
        <div className={`flex flex-col items-center rounded-3xl ${card} p-6`}>
          <Robot ref={robot} spec={spec} label={`机器人：${EXPRESSIONS[expr].label}`} size={360} interactive autoSleep {...flags} />
          <p className="mt-3 text-sm opacity-70">{EXPRESSIONS[expr].label} · {EXPRESSIONS[expr].en}</p>
          <div className="mt-4 flex flex-wrap justify-center gap-2 text-sm">
            {(['speaking', 'thinking', 'listening'] as const).map((k) => (
              <label key={k} className="flex items-center gap-1.5">
                <input type="checkbox" checked={flags[k]} onChange={(e) => setFlags({ ...flags, [k]: e.target.checked })} />
                {{ speaking: '说话', thinking: '思考中', listening: '在听' }[k]}
              </label>
            ))}
          </div>
          <div className="mt-3 flex flex-wrap justify-center gap-2 text-sm">
            <button type="button" className="rounded-full bg-[#edbd76] px-4 py-2 font-bold text-[#202a39]" onClick={() => robot.current?.poke()}>戳一下</button>
            <button type="button" className="rounded-full bg-[#edbd76] px-4 py-2 font-bold text-[#202a39]" onClick={() => robot.current?.play(DEMO)}>播放演示</button>
            <button type="button" className={`rounded-full px-4 py-2 ${dark ? 'bg-[#182338]' : 'bg-[#f4eee6]'}`} onClick={() => setOverrides({})}>重置参数</button>
          </div>
          <label className="mt-3 flex items-center gap-2 text-sm">
            屏幕符号
            <select className="rounded-lg bg-transparent px-2 py-1 ring-1 ring-current/20" value={symbol === undefined ? '__default' : symbol ?? '__none'}
              onChange={(e) => setSymbol(e.target.value === '__default' ? undefined : e.target.value === '__none' ? null : e.target.value as SymbolName)}>
              <option value="__default">跟随表情</option>
              <option value="__none">无</option>
              {SYMBOL_NAMES.filter((s) => s !== 'badge').map((s) => <option key={s} value={s}>{s}</option>)}
            </select>
          </label>
        </div>

        <div className={`rounded-3xl ${card} p-5`}>
          <div className="flex flex-wrap gap-1.5">
            {EXPRESSION_NAMES.map((name, i) => (
              <button key={name} type="button" onClick={() => { setExpr(name); setOverrides({}) }}
                className={`rounded-full px-3 py-1.5 text-sm ${name === expr ? 'bg-[#edbd76] font-bold text-[#202a39]' : dark ? 'bg-[#182338]' : 'bg-[#f4eee6]'}`}>
                {String(i + 1).padStart(2, '0')} {EXPRESSIONS[name].label}
              </button>
            ))}
          </div>
          <div className="mt-5 grid gap-x-8 gap-y-2 md:grid-cols-2">
            {SLIDERS.map((s) => (
              <label key={s.key} className="grid grid-cols-[6rem_minmax(0,1fr)_3rem] items-center gap-2 text-xs">
                <span className="opacity-75">{s.label}</span>
                <input className="w-full min-w-0" type="range" min={s.min} max={s.max} step={s.step} value={s.get(resolved)}
                  onChange={(e) => setOverrides(merge(overrides, s.set(Number(e.target.value))))} />
                <span className="text-right tabular-nums opacity-60">{s.get(resolved).toFixed(s.step >= 1 ? 0 : 2)}</span>
              </label>
            ))}
          </div>
        </div>
      </section>

      <section className="mx-auto mt-8 grid max-w-6xl grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-5">
        {EXPRESSION_NAMES.map((name, i) => (
          <button key={name} type="button" onClick={() => { setExpr(name); setOverrides({}); window.scrollTo({ top: 0, behavior: 'smooth' }) }}
            className="flex flex-col items-center text-left">
            <span className={`grid w-full place-items-center rounded-2xl ${card} py-3`}>
              <Robot spec={name} label={EXPRESSIONS[name].label} size={170} alive={tilesAlive} fps={30} />
            </span>
            <span className="mt-2 text-sm"><b className="mr-2 tabular-nums">{String(i + 1).padStart(2, '0')}</b><b>{EXPRESSIONS[name].label}</b> <span className="opacity-70">{EXPRESSIONS[name].en}</span></span>
          </button>
        ))}
      </section>
    </main>
  )
}
