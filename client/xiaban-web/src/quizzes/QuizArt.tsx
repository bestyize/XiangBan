import type { QuizId } from './types.ts'
import { personaArt } from './art.ts'

export function QuizArt({ kind, code }: { kind: QuizId; code?: string }) {
  return <svg className="quiz-art" viewBox="0 0 240 200" fill="none" aria-hidden="true">
    <circle cx="125" cy="105" r="76" fill="currentColor" opacity=".08" />
    <circle cx="125" cy="105" r="92" stroke="currentColor" strokeDasharray="2 8" opacity=".3" />
    {kind === 'animal' || kind === 'afterhours' ? <g dangerouslySetInnerHTML={{ __html: personaArt(kind, code) }} /> : kind === 'mbti' ? <>
      <path d="m126 35 22 48 53 22-53 22-22 48-22-48-53-22 53-22z" fill="currentColor" opacity=".9" />
      <path d="m126 35 1 70 74 0-53 22-22 48-1-70-74 0 53-22z" fill="#182338" opacity=".18" />
      <circle cx="126" cy="105" r="22" fill="#f3eee4" />
      <path d="m116 105 7 7 14-15" stroke="#2b3b52" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" />
      <circle cx="51" cy="50" r="16" fill="#edbd76" />
      <path d="m46 50 10 0m-5-5v10" stroke="#182338" strokeWidth="2" strokeLinecap="round" />
      <circle cx="198" cy="153" r="13" fill="#e7b0a1" />
      <path d="M195 154h6m-6-4h6" stroke="#182338" strokeWidth="2" strokeLinecap="round" />
    </> : <>
      <rect x="58" y="42" width="125" height="119" rx="22" fill="currentColor" transform="rotate(-7 120 100)" />
      <rect x="80" y="73" width="72" height="32" rx="7" stroke="#182338" strokeWidth="4" />
      <path d="M156 83v12" stroke="#182338" strokeWidth="4" strokeLinecap="round" />
      <rect x="87" y="80" width="17" height="18" rx="3" fill="#182338" />
      <rect x="109" y="80" width="17" height="18" rx="3" fill="#182338" opacity=".35" />
      <path d="M85 128h48m-48 10h31" stroke="#182338" strokeWidth="3" strokeLinecap="round" opacity=".65" />
      <rect x="154" y="124" width="49" height="45" rx="12" fill="#edbd76" />
      <path d="M203 134h5a11 11 0 0 1 0 22h-5M169 115v-9m13 9v-15" stroke="#edbd76" strokeWidth="4" strokeLinecap="round" />
      <path d="m35 56 5-10m-9 1 13 4m152-15 8 8m-8 0 8-8" stroke="currentColor" strokeWidth="3" strokeLinecap="round" />
    </>}
    <circle cx="31" cy="133" r="4" fill="currentColor" opacity=".5" />
    <path d="m197 55 9 0m-4-4v8" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
  </svg>
}
