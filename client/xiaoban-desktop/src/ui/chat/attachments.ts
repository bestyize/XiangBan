/** Images retain their existing data-URL representation; documents carry their display metadata. */
export interface FileAttachment { name: string; size: number; mime: string; dataUrl: string }
export interface AttachmentCard { id?: string; name: string; size: number; mime?: string }
export type PendingAttachment = string | FileAttachment;
export const MAX_ATTACHMENTS = 8;
export const MAX_TOTAL_BYTES = 30 * 1024 * 1024;
const IMAGE_TYPES = ['image/png', 'image/jpeg', 'image/gif', 'image/webp'];
const DOCUMENT_EXTENSIONS = new Set('pdf docx xlsx pptx txt md csv tsv json jsonl log xml yaml yml toml ini html css js ts tsx jsx py rs java kt swift c cpp h sh sql'.split(' '));
export const ATTACHMENT_ACCEPT = IMAGE_TYPES.join(',') + ',' + [...DOCUMENT_EXTENSIONS].map(ext => '.' + ext).join(',');
export const attachmentName = (item: PendingAttachment) => typeof item === 'string' ? '图片' : item.name;
export const attachmentImage = (item: PendingAttachment): string | null => typeof item === 'string' ? item : null;
export const attachmentBytes = (item: PendingAttachment) => typeof item === 'string' ? Math.ceil((item.split(',')[1]?.length ?? 0) * 3 / 4) : item.size;
export function attachmentCards(value: unknown): AttachmentCard[] {
  if (!value || typeof value !== 'object' || !('attachments' in value) || !Array.isArray(value.attachments)) return [];
  return value.attachments.filter((item): item is AttachmentCard => !!item && typeof item === 'object' && typeof item.name === 'string' && typeof item.size === 'number' && !String(item.mime ?? '').startsWith('image/'));
}
export function readAttachment(file: File): Promise<PendingAttachment> {
  const ext = file.name.split('.').pop()?.toLowerCase() ?? '';
  const fallbackMime: Record<string, string> = {png:'image/png',jpg:'image/jpeg',jpeg:'image/jpeg',gif:'image/gif',webp:'image/webp'};
  const imageMime = IMAGE_TYPES.includes(file.type) ? file.type : fallbackMime[ext];
  const image = !!imageMime;
  if (!image && !DOCUMENT_EXTENSIONS.has(ext)) return Promise.reject(Error('暂不支持这种文件，请选择图片、PDF、Office 文档或文本文件'));
  if (file.size > 10 * 1024 * 1024) return Promise.reject(Error(image ? '这张图片太大了，请换一张小于 10MB 的' : '单个文件不能超过 10MB'));
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = reader.onabort = () => reject(Error('文件读取失败，请重试'));
    reader.onload = () => {
      if (typeof reader.result !== 'string') { reject(Error('文件读取失败，请重试')); return; }
      resolve(image ? reader.result.replace(/^data:[^;]*;/, `data:${imageMime};`) : { name: file.name, size: file.size, mime: file.type, dataUrl: reader.result });
    };
    reader.readAsDataURL(file);
  });
}

