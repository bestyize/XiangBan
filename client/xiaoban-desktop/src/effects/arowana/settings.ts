import type { AquariumId } from '../types.ts';

export const aquariumModes = [
  { value: 'arowana', label: '单条龙鱼' },
  { value: 'guppy', label: '单条孔雀鱼' },
  { value: 'school', label: '一群孔雀鱼' },
  { value: 'mixed', label: '龙鱼和孔雀鱼群' },
] as const;
export type AquariumMode = typeof aquariumModes[number]['value'] | 'koi';

export const aquariumLightControls = [
  { key: 'main', label: '主光', min: 0, max: 300, step: 5 },
  { key: 'azimuth', label: '主光方向', min: -150, max: 150, step: 1 },
  { key: 'elevation', label: '主光高度', min: 0, max: 85, step: 1 },
  { key: 'warmth', label: '色温', min: -100, max: 100, step: 5 },
  { key: 'fill', label: '补光', min: 0, max: 300, step: 5 },
  { key: 'rim', label: '轮廓光', min: 0, max: 300, step: 5 },
  { key: 'environment', label: '环境反射', min: 0, max: 300, step: 5 },
] as const;
export type AquariumLight = typeof aquariumLightControls[number]['key'];
export interface AquariumSettings {
  mode: AquariumMode;
  aquascape: boolean;
  lighting: Record<AquariumLight, number>;
}

export const defaultAquarium: AquariumSettings = {
  mode: 'arowana',
  aquascape: false,
  lighting: { main: 100, azimuth: -31, elevation: 49, warmth: 0, fill: 100, rim: 100, environment: 100 },
};

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === 'object' && !Array.isArray(value) ? value as Record<string, unknown> : {};
}

export function normalizeAquarium(value: unknown, id: AquariumId = 'arowana'): AquariumSettings {
  const raw = asRecord(value);
  const lighting = asRecord(raw.lighting);
  const result = { ...defaultAquarium.lighting };
  for (const { key, min, max } of aquariumLightControls) {
    const number = lighting[key];
    if (typeof number === 'number' && Number.isFinite(number)) result[key] = Math.min(max, Math.max(min, number));
  }
  return {
    mode: id === 'koi' ? 'koi' : aquariumModes.find(({ value }) => value === raw.mode)?.value ?? defaultAquarium.mode,
    aquascape: typeof raw.aquascape === 'boolean' ? raw.aquascape : defaultAquarium.aquascape,
    lighting: result,
  };
}

export function lightValue(key: AquariumLight, number: number): string {
  if (key === 'azimuth') return number === 0 ? '正前' : `${number < 0 ? '左' : '右'} ${Math.abs(number)}°`;
  if (key === 'elevation') return `${number}°`;
  if (key === 'warmth') return number === 0 ? '原色' : `${number < 0 ? '偏冷' : '偏暖'} ${Math.abs(number)}`;
  return `${number}%`;
}
