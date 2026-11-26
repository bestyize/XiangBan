import type { PetDefinition } from '../model.ts';

// Already-imported packs store the two old snapshots as data URLs. Match their
// published bytes before replacing the pair, so edited artwork stays intact.
const previous = {
  standing: [62319, '8168e77b9b43ca54498479f814cc09ff7045e1129d8222cc091695ef00ff1c88'],
  blink: [60431, '9a0c9b06183da34f8d71e79d1c76b794047da97685fbe1f23c858dea606bf38a'],
} as const;
const names = ['standing', 'blink'] as const;
const images = [
  new URL('../../../../../assets/pet-packs/mingyue/assets/standing.webp', import.meta.url),
  new URL('../../../../../assets/pet-packs/mingyue/assets/blink.webp', import.meta.url),
];
let corrected: Promise<string[]> | undefined;

async function loadSprite(url: URL): Promise<string> {
  const response = await fetch(url);
  if (!response.ok) throw new Error('无法读取溟月备用姿态');
  const image = new Blob([await response.arrayBuffer()], { type: 'image/webp' });
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(image);
  });
}

export async function correctedMingyueSprites(pet: PetDefinition): Promise<PetDefinition> {
  const sources = names.map(id => pet.parts.find(part => part.id === id)?.shapes[0]?.src ?? '');
  if (!crypto.subtle || sources.some((src, i) => src.length !== previous[names[i]][0])) return pet;
  const matches = await Promise.all(sources.map(async (src, i) => {
    const bytes = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(src));
    const digest = Array.from(new Uint8Array(bytes), byte => byte.toString(16).padStart(2, '0')).join('');
    return digest === previous[names[i]][1];
  }));
  if (!matches.every(Boolean)) return pet;
  corrected ??= Promise.all(images.map(loadSprite)).catch(error => { corrected = undefined; throw error; });
  const [standing, blink] = await corrected;
  return { ...pet, parts: pet.parts.map(part => {
    if (part.id !== 'standing' && part.id !== 'blink') return part;
    const src = part.id === 'standing' ? standing : blink;
    const [image, ...rest] = part.shapes;
    return { ...part, shapes: [{ ...image, src }, ...rest] };
  }) };
}
