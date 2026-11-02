import { shallowRef } from 'vue';
import { invoke } from '../../bridge.ts';
import springRiver from '../application-skins/assets/spring-river.webp?inline';
import lantern from '../application-skins/assets/lantern.webp?inline';
import jiangnan from '../application-skins/assets/jiangnan.webp?inline';
import seasideTrain from '../application-skins/assets/seaside-train.webp?inline';

export interface SkinLayer { id: string; image: string; depth: number; anchor: string; interactive?: boolean }
export interface SkinInteraction { target: string; trigger: string; action: string; label: string; repeat?: string }
export interface Skin { id: string; name: string; version: string; author: string; preview: string; scene: { layers: SkinLayer[]; interactions: SkinInteraction[] } }
interface SkinList { skins: Skin[]; activeId: string | null }
export const builtinSkin: Skin = { id: 'builtin:spring-river', name: '春江花月夜', version: '1.0.0', author: '小伴', preview: springRiver, scene: { layers: [{ id: 'river', image: springRiver, depth: 0, anchor: 'center' }, { id: 'lantern', image: lantern, depth: 2, anchor: 'bottom-right', interactive: true }], interactions: [{ target: 'lantern', trigger: 'click', action: 'glow', label: '点亮花灯', repeat: 'restart' }] } };
export const builtinSkins: readonly Skin[] = [
  builtinSkin,
  { id: 'builtin:jiangnan', name: '江南烟雨', version: '1.0.0', author: '小伴', preview: jiangnan, scene: { layers: [{ id: 'background', image: jiangnan, depth: 0, anchor: 'center' }], interactions: [] } },
  { id: 'builtin:seaside-train', name: '海边列车', version: '1.0.0', author: '小伴', preview: seasideTrain, scene: { layers: [{ id: 'background', image: seasideTrain, depth: 0, anchor: 'center' }], interactions: [] } },
];
export function isBuiltinSkin(id: string) { return builtinSkins.some(skin => skin.id === id); }
export const skins = shallowRef<Skin[]>([...builtinSkins]);
export const activeSkin = shallowRef<Skin | null>(null);
export const activeSkinId = shallowRef<string | null>(null);
function accept(result: SkinList) {
  skins.value = [...builtinSkins, ...result.skins.filter(s => !isBuiltinSkin(s.id))];
  activeSkinId.value = result.activeId;
  activeSkin.value = skins.value.find(s => s.id === result.activeId) ?? null;
}
let revision = 0;
async function update(command: string, args?: Record<string, unknown>) {
  const request = ++revision;
  const result = await invoke<SkinList>(command, args);
  if (request === revision) accept(result);
}
export async function refreshSkins() { await update('skin_list'); }
export async function applySkin(id: string | null) { await update('skin_apply', { id }); }
export async function importSkin() { const imported = await invoke<Skin | null>('skin_import'); if (imported) await refreshSkins(); return imported; }
export async function deleteSkin(id: string) { await update('skin_delete', { id }); }
export async function exportSkin(id: string) { return await invoke<boolean>('skin_export', { id }); }
