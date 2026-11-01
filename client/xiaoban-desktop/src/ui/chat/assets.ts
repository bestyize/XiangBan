import type { Summary } from '../design/model.ts';
import type { GeneratedImage } from './generated-images.ts';
import type { ReplyBlock } from './replies.ts';

export interface LocalAsset {
  id: string; name: string; path: string; kind: 'file' | 'folder' | 'image';
  exists: boolean; imported: boolean; bytes: number; updatedMs: number; toolCallId?: string | null;
}
export interface ConversationAsset {
  key: string; id: string; name: string; type: 'design' | 'image' | 'file';
  updatedMs: number; blockId?: string; local?: LocalAsset; generated?: GeneratedImage; design?: Summary;
  /** Descendants retain the root's opaque ID; native code resolves this relative path. */
  relative?: string;
}
export function conversationAssets(designs: Summary[], files: LocalAsset[], blocks: ReplyBlock[]): ConversationAsset[] {
  const assets: ConversationAsset[] = designs.map(design => ({
    key: `design:${design.id}`, id: design.id, name: design.title, type: 'design', updatedMs: design.updatedMs, design,
    blockId: [...blocks].reverse().find(b => b.calls.some(c => c.name.startsWith('design_') && c.arguments.includes(design.id)))?.id,
  }));
  const images = new Map<string, ConversationAsset>();
  for (const block of blocks) for (const call of block.calls) {
    if (call.name !== 'image_generate') continue;
    for (const image of call.generatedImages ?? []) images.set(image.id, {
      key: `generated:${image.id}`, id: image.id, name: image.name, type: 'image', updatedMs: 0, blockId: block.id, generated: image,
    });
  }
  assets.push(...[...images.values()].reverse());
  assets.push(...files.map(local => ({
    key: `local:${local.id}`, id: local.id, name: local.name, type: local.kind === 'image' ? 'image' as const : 'file' as const,
    updatedMs: local.updatedMs, local,
    blockId: [...blocks].reverse().find(b => (local.toolCallId && b.calls.some(c => c.id === local.toolCallId))
      || (!local.imported && b.items.some(i => i.kind === 'assistant' && (i.text.includes(local.path) || i.text.includes(`](${local.name})`)))))?.id,
  })));
  return assets;
}
export function assetBytes(bytes: number): string {
  return bytes >= 1024 * 1024 ? `${(bytes / 1024 / 1024).toFixed(1)} MB` : bytes >= 1024 ? `${Math.ceil(bytes / 1024)} KB` : `${bytes} B`;
}
export function assetMeta(asset: ConversationAsset): string {
  if (asset.design) return asset.design.status === 'generating' ? '生成中' : `${asset.design.artboards} 画板`;
  if (asset.generated) return `${asset.generated.width} × ${asset.generated.height}`;
  return !asset.local?.exists ? '已失效' : asset.local.kind === 'folder' ? '文件夹' : assetBytes(asset.local.bytes);
}
