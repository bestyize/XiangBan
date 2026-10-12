import { computed, defineComponent, h, type VNodeArrayChildren, type VNodeChild } from 'vue';
import CodeBlock from './CodeBlock.vue';
import { parseMarkdown, type Block, type Inline } from './markdown.ts';

/**
 * Draws an answer. Everything goes in as Vue nodes and text, never as HTML, and links carry no
 * `href`: a real address would let a middle click or ctrl-click take the app's own window away.
 * Clicking (or Enter on) a link asks the page to open it.
 */
export default defineComponent({
  name: 'MarkdownView',
  props: { source: { type: String, required: true } },
  emits: { open: (href: string) => typeof href === 'string' },
  setup(props, { emit }) {
    const blocks = computed(() => parseMarkdown(props.source));

    const open = (href: string) => emit('open', href);
    const inline = (nodes: Inline[]): VNodeArrayChildren => nodes.map((node): VNodeChild => {
      switch (node.t) {
        case 'text': return node.v;
        case 'code': return h('code', { class: 'md-code' }, node.v);
        case 'br': return h('br');
        case 'strong': return h('strong', inline(node.c));
        case 'em': return h('em', inline(node.c));
        case 'del': return h('del', inline(node.c));
        case 'link': return h('a', {
          class: 'md-link', role: 'link', tabindex: 0, title: node.href,
          onClick: (event: MouseEvent) => { event.preventDefault(); open(node.href); },
          onKeydown: (event: KeyboardEvent) => { if (event.key === 'Enter') { event.preventDefault(); open(node.href); } },
        }, inline(node.c));
      }
    });

    const block = (node: Block): VNodeChild => {
      switch (node.t) {
        case 'p': return h('p', inline(node.c));
        case 'h': return h(`h${Math.min(6, node.level + 2)}`, { class: 'md-heading' }, inline(node.c));
        case 'hr': return h('hr');
        case 'code': return h(CodeBlock, { lang: node.lang, code: node.v });
        case 'quote': return h('blockquote', node.c.map(block));
        case 'list': {
          const items = node.items.map(item => h('li', item.length === 1 && item[0].t === 'p' ? inline(item[0].c) : item.map(block)));
          return node.ordered ? h('ol', node.start !== 1 ? { start: node.start } : {}, items) : h('ul', items);
        }
        case 'table': {
          const style = (n: number) => (node.align[n] ? { textAlign: node.align[n] as string } : undefined);
          return h('div', { class: 'md-table' }, h('table', [
            h('thead', h('tr', node.head.map((cell, n) => h('th', { style: style(n) }, inline(cell))))),
            h('tbody', node.rows.map(row => h('tr', row.map((cell, n) => h('td', { style: style(n) }, inline(cell)))))),
          ]));
        }
      }
    };

    return () => h('div', { class: 'markdown' }, blocks.value.map(block));
  },
});
