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
/** Read visible prose and link labels; omit code blocks, URL destinations and Markdown syntax. */
export const speechText = (source: string) => blocks(parseMarkdown(source)).replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/g, '').trim();
