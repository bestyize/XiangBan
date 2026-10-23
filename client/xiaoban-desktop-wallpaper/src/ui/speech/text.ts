import { parseMarkdown, type Block, type Inline } from '../chat/markdown.ts';
const inline = (nodes: Inline[]): string => nodes.map(n => n.t === 'text' || n.t === 'code' ? n.v : n.t === 'br' ? '\n' : inline(n.c)).join('');
function blocks(nodes: Block[]): string {
  return nodes.map(n => {
    switch (n.t) {
      case 'p': case 'h': return inline(n.c);
      case 'quote': return blocks(n.c);
      case 'list': return n.items.map(blocks).join('\n');
      case 'table': return [n.head, ...n.rows].map(row => row.map(inline).join('，')).join('\n');
      case 'code': case 'hr': return '';
    }
  }).filter(Boolean).join('\n');
}
/** Emoji and their glue (skin tones, flags, zero-width joiners, variation selectors): the voice would read each one by its name. */
const EMOJI = /[\p{Extended_Pictographic}\u{1F1E6}-\u{1F1FF}\u{1F3FB}-\u{1F3FF}\u{E0020}-\u{E007F}\uFE0E\uFE0F\u200D\u20E3]/gu;
/** "1️⃣" is the digit with a frame around it: say the digit. */
const KEYCAP = /([0-9#*])\uFE0F?\u20E3/g;
/** Read visible prose and link labels; omit code blocks, URL destinations, Markdown syntax and emoji. */
export const speechText = (source: string) => blocks(parseMarkdown(source))
  .replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/g, '')
  .replace(KEYCAP, '$1').replace(EMOJI, '').replace(/[ \t]{2,}/g, ' ').trim();
