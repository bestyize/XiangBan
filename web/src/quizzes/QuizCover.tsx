import { useId } from 'react'
import { personaArt } from './art.ts'
import type { QuizId } from './types.ts'

const backgrounds: Record<QuizId, [string, string]> = {
  mbti: ['#416064', '#263d49'],
  mbanti: ['#655778', '#393951'],
  animal: ['#77634f', '#48544e'],
  afterhours: ['#365d72', '#283f59'],
}

function Portrait({ x, y, turn, paper, shirt, hair, glasses = false }: {
  x: number; y: number; turn: number; paper: string; shirt: string; hair: string; glasses?: boolean
}) {
  return <g transform={`translate(${x} ${y}) rotate(${turn} 38 48)`}>
    <rect x="3" y="5" width="76" height="104" rx="16" fill="#142938" opacity=".2" />
    <rect width="76" height="104" rx="16" fill={paper} />
    <path d="M8 99v-6c0-19 13-31 30-31s30 12 30 31v6" fill={shirt} />
    <rect x="30" y="60" width="16" height="15" rx="7" fill="#dfae91" />
    <path d="M15 47V34c0-17 10-24 23-24s24 8 24 25v21H15" fill={hair} />
    <ellipse cx="38" cy="43" rx="22" ry="26" fill="#f0c6a6" />
    <path d="M16 36c0-23 31-30 44-7-17 0-17 8-32 5-3 6-7 10-12 10" fill={hair} />
    <path d="M27 45h3m16 0h3" stroke="#48444c" strokeWidth="3" strokeLinecap="round" />
    <path d="M33 56q5 5 10 0" stroke="#9f655c" strokeWidth="2" strokeLinecap="round" fill="none" />
    <ellipse cx="23" cy="53" rx="4" ry="2" fill="#dd938a" opacity=".5" />
    <ellipse cx="53" cy="53" rx="4" ry="2" fill="#dd938a" opacity=".5" />
    {glasses && <g stroke="#535965" strokeWidth="2" fill="none"><rect x="20" y="39" width="15" height="13" rx="5" /><rect x="41" y="39" width="15" height="13" rx="5" /><path d="M35 43h6" /></g>}
    <path d="m28 72 10 9 10-9" stroke={paper} strokeWidth="3" fill="none" opacity=".7" />
  </g>
}

function PersonalityScene() {
  return <>
    <path d="M21 130C-7 83 35 27 104 47s92-30 146-5 88 97 42 121H30" fill="#a6cdb5" opacity=".13" />
    <path d="M39 131c23-34 69-48 122-38s99-7 121-30" stroke="#c2d9c2" strokeWidth="1.5" strokeDasharray="3 7" fill="none" opacity=".5" />
    <Portrait x={49} y={48} turn={-12} paper="#d7e6db" shirt="#789d8e" hair="#646276" />
    <Portrait x={199} y={43} turn={12} paper="#ead6c4" shirt="#cf9f82" hair="#5b4e48" />
    <Portrait x={125} y={56} turn={3} paper="#e5ddea" shirt="#a695b9" hair="#87695a" glasses />
    <circle cx="38" cy="105" r="13" fill="#e5bc7e" />
    <path d="m38 97 2 5 5 3-5 2-2 6-2-6-5-2 5-3z" fill="#4d6662" />
    <path d="M269 103h19q9 0 9 9v4q0 9-9 9h-8l-9 7v-7h-2q-9 0-9-9v-4q0-9 9-9" fill="#a6cdb5" />
    <circle cx="272" cy="114" r="2" fill="#416064" /><circle cx="279" cy="114" r="2" fill="#416064" /><circle cx="286" cy="114" r="2" fill="#416064" />
    <path d="m182 28 6 0m-3-3v6M29 54h6m-3-3v6" stroke="#dddcc4" strokeWidth="2" strokeLinecap="round" opacity=".7" />
  </>
}

function WorkScene() {
  return <>
    <rect x="38" y="44" width="67" height="56" rx="8" fill="#c7bbd8" opacity=".25" />
    <path d="M48 57h45m-45 11h34m-34 11h39" stroke="#c9c2da" strokeWidth="2" strokeLinecap="round" opacity=".45" />
    <rect x="226" y="47" width="48" height="39" rx="7" fill="#e6c58d" transform="rotate(9 250 65)" />
    <path d="m239 60 4 4 8-8m-10 15h17" stroke="#91766d" strokeWidth="2.5" strokeLinecap="round" fill="none" />
    <ellipse cx="165" cy="155" rx="112" ry="9" fill="#222c43" opacity=".35" />
    <path d="M126 107v-21q0-36 36-36t36 36v21" fill="#eac5a5" />
    <path d="m128 70-2-23 24 16m26 0 23-16-2 25" fill="#eac5a5" />
    <path d="m134 57 2 12 8-5m37 0 10 5 1-12" fill="#d89f8d" />
    <ellipse cx="162" cy="91" rx="39" ry="31" fill="#eac5a5" />
    <path d="m144 85 10 3m17 0 10-3m-24 13 5 3 5-3m-5 3v5" stroke="#6a5756" strokeWidth="2.5" strokeLinecap="round" fill="none" />
    <ellipse cx="137" cy="98" rx="6" ry="3" fill="#d99a8b" /><ellipse cx="187" cy="98" rx="6" ry="3" fill="#d99a8b" />
    <path d="M129 128q6-20 33-20t33 20" fill="#b7a6cd" />
    <rect x="110" y="111" width="101" height="37" rx="6" fill="#c8cbd9" />
    <rect x="116" y="116" width="89" height="26" rx="3" fill="#66617c" />
    <rect x="148" y="123" width="24" height="10" rx="3" stroke="#d8cce5" strokeWidth="1.5" fill="none" /><path d="M175 126v4" stroke="#d8cce5" strokeWidth="2" /><rect x="151" y="126" width="5" height="4" rx="1" fill="#dca799" />
    <path d="M103 146h115l-5 7H107z" fill="#dde0e7" />
    <path d="M51 150h231" stroke="#d6bd9b" strokeWidth="8" strokeLinecap="round" />
    <path d="M72 154v13m188-13v13" stroke="#a49388" strokeWidth="5" strokeLinecap="round" />
    <path d="M236 124h19v17q0 5-6 5h-7q-6 0-6-5z" fill="#e5bc7e" /><path d="M255 128h4q7 0 7 6t-7 6h-4" stroke="#e5bc7e" strokeWidth="3" fill="none" />
    <path d="M243 116q-4-4 0-8m8 8q-4-4 0-8" stroke="#ddd0dc" strokeWidth="2" strokeLinecap="round" fill="none" opacity=".6" />
    <rect x="70" y="127" width="22" height="17" rx="3" fill="#99b5ac" /><path d="M75 133h12m-12 5h8" stroke="#dae1d1" strokeWidth="2" strokeLinecap="round" />
    <path d="m210 34 6-6m-7 1 5 5" stroke="#d6c7e4" strokeWidth="2" strokeLinecap="round" />
  </>
}

function AnimalScene() {
  return <>
    <circle cx="242" cy="54" r="23" fill="#e5bd82" opacity=".7" />
    <path d="M17 151q45-51 96-25t100-4 99 25v33H17" fill="#809583" />
    <path d="M0 164q67-31 133-9t187-6v31H0" fill="#a2ac8c" opacity=".45" />
    <path d="M45 145V98m0 30q-20-3-18-24 20 1 18 24m1-12q20-3 21-24-21 3-21 24" stroke="#aec2a0" strokeWidth="2" fill="#aec2a0" />
    <path d="M282 148v-34m0 18q-17-1-17-18 18 1 17 18m1-9q16-1 18-19-19 1-18 19" stroke="#c2c6a2" strokeWidth="2" fill="#c2c6a2" />
    <ellipse cx="91" cy="144" rx="30" ry="10" fill="#34463b" opacity=".2" /><ellipse cx="162" cy="152" rx="37" ry="10" fill="#34463b" opacity=".2" /><ellipse cx="241" cy="145" rx="31" ry="9" fill="#34463b" opacity=".2" />
    <ellipse cx="91" cy="127" rx="27" ry="22" fill="#eac2a1" /><ellipse cx="162" cy="131" rx="30" ry="26" fill="#d59a6b" /><ellipse cx="242" cy="127" rx="29" ry="21" fill="#d5b795" />
    <g transform="translate(31 39) scale(.5)" dangerouslySetInnerHTML={{ __html: personaArt('animal', 'CAT') }} />
    <g transform="translate(95 22) scale(.56)" dangerouslySetInnerHTML={{ __html: personaArt('animal', 'FOX') }} />
    <g transform="translate(185 45) scale(.48)" dangerouslySetInnerHTML={{ __html: personaArt('animal', 'CAPY') }} />
    <path d="M81 143h7m9 0h7m47 9h8m11 0h8m54-8h8m10 0h8" stroke="#f1dcc0" strokeWidth="5" strokeLinecap="round" />
    <path d="m120 38 2 4 5 1-5 2-2 5-2-5-5-2 5-1zm154 45 2 4 4 1-4 2-2 4-2-4-4-2 4-1z" fill="#e6c99b" />
    <circle cx="72" cy="55" r="3" fill="#cbd2a9" /><circle cx="206" cy="88" r="2" fill="#cbd2a9" />
  </>
}

function EveningScene() {
  return <>
    <rect x="189" y="31" width="75" height="74" rx="24" fill="#1e314c" stroke="#6b8b9c" strokeWidth="3" />
    <path d="M226 32v72m-35-34h72" stroke="#6b8b9c" strokeWidth="2" />
    <path d="M240 43a11 11 0 1 0 12 16 10 10 0 0 1-12-16" fill="#e8c285" />
    <circle cx="206" cy="50" r="1.5" fill="#bcd5dc" /><circle cx="250" cy="84" r="1.5" fill="#bcd5dc" />
    <path d="M198 99V86h11v13m9 0V80h12v19m8 0V88h13v11" fill="#36536f" />
    <path d="M48 132V74m-6 59h15" stroke="#b9a98e" strokeWidth="3" strokeLinecap="round" />
    <path d="m31 74 10-32h17l10 32z" fill="#e8c285" /><path d="M32 75h35l31 67H13z" fill="#e8c285" opacity=".06" />
    <rect x="78" y="95" width="97" height="45" rx="15" fill="#8caeb8" />
    <rect x="70" y="114" width="113" height="36" rx="12" fill="#7397a5" />
    <rect x="74" y="108" width="14" height="33" rx="7" fill="#a7c3c6" /><rect x="165" y="108" width="14" height="33" rx="7" fill="#a7c3c6" />
    <path d="m102 99 1-16 15 10m14 0 15-10 1 16" fill="#eac2a1" />
    <ellipse cx="125" cy="109" rx="28" ry="22" fill="#eac2a1" />
    <path d="M112 105h6m15 0h6m-19 12q5 4 10 0" stroke="#6b5954" strokeWidth="2.5" strokeLinecap="round" fill="none" />
    <path d="M92 125q34-17 65 0v14H92z" fill="#c7b8d6" /><path d="M105 126v11m16-15v15m17-11v11" stroke="#e4d9e6" strokeWidth="2" opacity=".7" />
    <path d="M86 149v8m83-8v8" stroke="#ac9d8b" strokeWidth="4" strokeLinecap="round" />
    <ellipse cx="216" cy="137" rx="23" ry="7" fill="#c4ad8d" /><path d="M216 141v18" stroke="#a89780" strokeWidth="4" strokeLinecap="round" />
    <path d="M209 120h13v12h-13zm13 3h4q4 0 4 4t-4 4h-4" fill="#dfc18f" />
    <path d="M270 148v-34m0 14q-14-1-17-17 18 1 17 17m0-6q16-4 19-23-19 5-19 23" stroke="#a1bca4" strokeWidth="2" fill="#a1bca4" />
    <path d="M258 142h25l-4 18h-17z" fill="#d2ac8b" />
    <ellipse cx="141" cy="167" rx="90" ry="6" fill="#5c8498" opacity=".25" />
    <path d="M94 44v13m0-13 9-2v12" stroke="#b8ccd7" strokeWidth="2" strokeLinecap="round" /><ellipse cx="91" cy="57" rx="4" ry="3" fill="#b8ccd7" /><ellipse cx="100" cy="54" rx="4" ry="3" fill="#b8ccd7" />
  </>
}

/** Decorative cover scenes stay separate from the result-specific report illustrations. */
export function QuizCover({ kind }: { kind: QuizId }) {
  const id = useId()
  const [top, bottom] = backgrounds[kind]
  return <svg viewBox="0 0 320 180" fill="none" aria-hidden="true" focusable="false">
    <defs><linearGradient id={id} x1="0" y1="0" x2="320" y2="180" gradientUnits="userSpaceOnUse"><stop stopColor={top} /><stop offset="1" stopColor={bottom} /></linearGradient></defs>
    <rect width="320" height="180" rx="18" fill={`url(#${id})`} />
    <circle cx="294" cy="-10" r="84" fill="#e8ddbf" opacity=".035" />
    {kind === 'mbti' ? <PersonalityScene /> : kind === 'mbanti' ? <WorkScene /> : kind === 'animal' ? <AnimalScene /> : <EveningScene />}
  </svg>
}
