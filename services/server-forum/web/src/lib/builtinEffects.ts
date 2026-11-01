import lanterns from './builtinEffectsData/lanterns.json'
import maple from './builtinEffectsData/maple.json'
import galaxy from './builtinEffectsData/galaxy.json'
import butterfly from './builtinEffectsData/butterfly-dream.json'
import fireworks from './builtinEffectsData/fireworks.json'
import snow from './builtinEffectsData/snow.json'
import rain from './builtinEffectsData/rain.json'
import fireflies from './builtinEffectsData/fireflies.json'
import petals from './builtinEffectsData/petals.json'
import meteors from './builtinEffectsData/meteors.json'
import stars from './builtinEffectsData/stars.json'
import aurora from './builtinEffectsData/aurora.json'
import { parsePack } from './builtinEffectsEngine/parse'
import type { EffectPack } from './builtinEffectsEngine/types'

const official = [lanterns, maple, galaxy, butterfly, fireworks, snow, rain, fireflies, petals, meteors, stars, aurora]
export const BUILTIN_EFFECTS = official.map(({id,name,description}) => ({id,name,description,kind:'effect' as const}))
const packs = new Map<string, EffectPack>()
for (const raw of official) {
  const parsed = parsePack(raw)
  if (!parsed.ok) throw new Error(`官方特效 ${raw.id} 无法读取：${parsed.error}`)
  packs.set(raw.id, parsed.pack)
}
export function builtinEffectPack(id:string): EffectPack | undefined { return packs.get(id) }
