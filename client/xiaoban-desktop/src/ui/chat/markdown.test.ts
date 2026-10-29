import { describe, expect, it } from 'vitest';
import { parseInline, parseMarkdown, safeHref, type Block, type Inline } from './markdown.ts';

const text = (v: string): Inline => ({ t: 'text', v });
const para = (...c: Inline[]): Block => ({ t: 'p', c });
/** Flattens inline nodes back to the words a reader would see. */
const plain = (nodes: Inline[]): string => nodes.map(n => n.t === 'text' || n.t === 'code' ? n.v : n.t === 'br' ? '\n' : plain(n.c)).join('');

describe('inline text', () => {
  it('reads emphasis, strong text, strikethrough and code', () => {
    expect(parseInline('a **b** *c* ~~d~~ `e`')).toEqual([text('a '), { t: 'strong', c: [text('b')] }, text(' '), { t: 'em', c: [text('c')] }, text(' '), { t: 'del', c: [text('d')] }, text(' '), { t: 'code', v: 'e' }]);
    expect(parseInline('***both***')).toEqual([{ t: 'strong', c: [{ t: 'em', c: [text('both')] }] }]);
    expect(parseInline('__bold__ and _it_')).toEqual([{ t: 'strong', c: [text('bold')] }, text(' and '), { t: 'em', c: [text('it')] }]);
    expect(parseInline('**a *b* c**')).toEqual([{ t: 'strong', c: [text('a '), { t: 'em', c: [text('b')] }, text(' c')] }]);
  });

  it('leaves arithmetic, snake_case names and lone marks alone', () => {
    for (const same of ['2 * 3 * 4', 'my_file_name.txt', 'snake_case_and_more', 'a * b', '**', '* ', 'x ** y', '~', 'a ~ b ~ c', '50% * 2', 'what_is_this_']) {
      expect(plain(parseInline(same)), same).toBe(same);
      expect(parseInline(same).every(n => n.t === 'text'), same).toBe(true);
    }
    expect(plain(parseInline('**unclosed bold'))).toBe('**unclosed bold');
  });

  it('keeps code spans literal, including marks inside them, and handles longer fences and unclosed ticks', () => {
    expect(parseInline('`a *b* **c**`')).toEqual([{ t: 'code', v: 'a *b* **c**' }]);
    expect(parseInline('``a ` b``')).toEqual([{ t: 'code', v: 'a ` b' }]);
    expect(parseInline('` pad `')).toEqual([{ t: 'code', v: 'pad' }]);
    expect(plain(parseInline('open `tick'))).toBe('open `tick');
    expect(plain(parseInline('``one` `two``'))).toBe('one` `two');
  });

  it('turns escapes into plain characters and newlines into line breaks', () => {
    expect(plain(parseInline('\\*not em\\* and \\`x\\`'))).toBe('*not em* and `x`');
    expect(parseInline('a\nb')).toEqual([text('a'), { t: 'br' }, text('b')]);
    expect(parseInline('trailing  \nspaces')[0]).toEqual(text('trailing'));
  });
});

describe('links', () => {
  it('keeps web and mail links and nothing else', () => {
    expect(parseInline('[官网](https://example.com/a?b=1 "title")')).toEqual([{ t: 'link', href: 'https://example.com/a?b=1', c: [text('官网')] }]);
    expect(parseInline('[mail](mailto:a@b.co)')[0]).toMatchObject({ t: 'link', href: 'mailto:a@b.co' });
    expect(parseInline('[a](<https://example.com/with space>)')[0].t).toBe('text');
    for (const bad of ['javascript:alert(1)', 'JaVaScRiPt:alert(1)', 'data:text/html,<script>', 'file:///etc/passwd', 'vbscript:x', '//evil.example.com', '/relative', 'example.com', 'tauri://localhost', 'java\tscript:alert(1)', ' javascript:alert(1)']) {
      const nodes = parseInline(`[click me](${bad})`);
      expect(nodes.some(n => n.t === 'link'), bad).toBe(false);
      expect(plain(nodes)).toContain('click me');
    }
  });

  it('shows a link as text when the address is not safe, and never nests links', () => {
    expect(parseInline('[x](javascript:alert(1))')).toEqual([text('x')]);
    const nested = parseInline('[a [b](https://one.example) c](https://two.example)');
    const links = JSON.stringify(nested).match(/"t":"link"/g) ?? [];
    expect(links.length).toBe(1);
  });

  it('does not load images; they become links', () => {
    expect(parseInline('![猫](https://example.com/cat.png)')).toEqual([{ t: 'link', href: 'https://example.com/cat.png', c: [text('猫')] }]);
    expect(parseInline('![](https://example.com/cat.png)')[0]).toMatchObject({ t: 'link', c: [text('图片')] });
    expect(parseInline('![x](javascript:alert(1))').some(n => n.t === 'link')).toBe(false);
  });

  it('links bare addresses without taking the punctuation after them', () => {
    expect(parseInline('see https://example.com/a. Next')).toEqual([text('see '), { t: 'link', href: 'https://example.com/a', c: [text('https://example.com/a')] }, text('. Next')]);
    expect(parseInline('(https://example.com/a)')).toEqual([text('('), { t: 'link', href: 'https://example.com/a', c: [text('https://example.com/a')] }, text(')')]);
    expect(parseInline('https://en.wikipedia.org/wiki/A_(b)')[0]).toMatchObject({ href: 'https://en.wikipedia.org/wiki/A_(b)' });
    expect(parseInline('详见 https://example.com/a。下一句')).toEqual([text('详见 '), { t: 'link', href: 'https://example.com/a', c: [text('https://example.com/a')] }, text('。下一句')]);
    expect(parseInline('（https://example.com/a），还有“https://example.org/b”')).toEqual([
      text('（'), { t: 'link', href: 'https://example.com/a', c: [text('https://example.com/a')] }, text('），还有“'), { t: 'link', href: 'https://example.org/b', c: [text('https://example.org/b')] }, text('”'),
    ]);
    expect(parseInline('<https://example.com>')[0]).toMatchObject({ t: 'link', href: 'https://example.com' });
    expect(plain(parseInline('<javascript:alert(1)>'))).toBe('<javascript:alert(1)>');
    expect(plain(parseInline('xhttps://example.com')).startsWith('xhttps')).toBe(true);
    expect(parseInline('xhttps://example.com').some(n => n.t === 'link')).toBe(false);
  });

  it('decides what is safe in one place', () => {
    expect(safeHref('https://a.example')).toBe('https://a.example');
    expect(safeHref(' http://a.example ')).toBe('http://a.example');
    expect(safeHref('')).toBeNull();
    expect(safeHref('https://a.example/' + 'x'.repeat(2100))).toBeNull();
    expect(safeHref('https://a.example/\u0000')).toBeNull();
  });
});

describe('markup from the model is only ever text', () => {
  it('keeps html, scripts and attributes as the words they are', () => {
    const attacks = ['<script>alert(1)</script>', '<img src=x onerror=alert(1)>', '<a href="javascript:alert(1)">x</a>', '<iframe src="https://evil.example"></iframe>', '<style>*{display:none}</style>', '&lt;b&gt;', '<!-- c -->'];
    for (const attack of attacks) {
      const blocks = parseMarkdown(attack);
      expect(blocks).toHaveLength(1);
      expect(blocks[0].t, attack).toBe('p');
      const first = blocks[0] as { c: Inline[] };
      expect(plain(first.c), `${attack} reads back as the same words`).toBe(attack);
      const hrefs = [...JSON.stringify(blocks).matchAll(/"href":"([^"]*)"/g)].map(m => m[1]);
      expect(hrefs.every(h => /^https?:\/\//.test(h)), `${attack}: ${hrefs}`).toBe(true);
    }
  });
});

describe('blocks', () => {
  it('reads paragraphs, headings, rules and quotes', () => {
    const blocks = parseMarkdown('# 标题\n\n第一段\n第二行\n\n---\n\n> 引用 **重点**\n> 第二行\n\n###### 小标题 ##\n#没有空格');
    expect(blocks[0]).toEqual({ t: 'h', level: 1, c: [text('标题')] });
    expect(blocks[1]).toEqual(para(text('第一段'), { t: 'br' }, text('第二行')));
    expect(blocks[2]).toEqual({ t: 'hr' });
    expect(blocks[3]).toEqual({ t: 'quote', c: [para(text('引用 '), { t: 'strong', c: [text('重点')] }, { t: 'br' }, text('第二行'))] });
    expect(blocks[4]).toEqual({ t: 'h', level: 6, c: [text('小标题')] });
    expect(blocks[5]).toEqual(para(text('#没有空格')));
  });

  it('keeps fenced code exactly, with its language, and treats an unfinished fence as code', () => {
    expect(parseMarkdown('```ts\nconst a = 1;\n\n  indented\n```\nafter')).toEqual([{ t: 'code', lang: 'ts', v: 'const a = 1;\n\n  indented' }, para(text('after'))]);
    expect(parseMarkdown('~~~\n```\ninner\n```\n~~~')).toEqual([{ t: 'code', lang: '', v: '```\ninner\n```' }]);
    expect(parseMarkdown('````\n```\nx\n````')).toEqual([{ t: 'code', lang: '', v: '```\nx' }]);
    expect(parseMarkdown('```py\nprint(1)\nstill typing')).toEqual([{ t: 'code', lang: 'py', v: 'print(1)\nstill typing' }]);
    expect(parseMarkdown('```')).toEqual([{ t: 'code', lang: '', v: '' }]);
    expect(parseMarkdown('```\n<b>*raw*</b>\n```')).toEqual([{ t: 'code', lang: '', v: '<b>*raw*</b>' }]);
  });

  it('reads bullet, numbered and nested lists', () => {
    const list = parseMarkdown('- 一\n- 二\n  - 二点一\n  - 二点二\n- 三\n\n3. 甲\n4. 乙')[0] as Extract<Block, { t: 'list' }>;
    expect(list.ordered).toBe(false);
    expect(list.items).toHaveLength(3);
    expect(list.items[1][0]).toEqual(para(text('二')));
    const inner = list.items[1][1] as Extract<Block, { t: 'list' }>;
    expect(inner.items.map(i => plain((i[0] as { c: Inline[] }).c))).toEqual(['二点一', '二点二']);
    const numbered = parseMarkdown('3. 甲\n4. 乙')[0] as Extract<Block, { t: 'list' }>;
    expect([numbered.ordered, numbered.start, numbered.items.length]).toEqual([true, 3, 2]);
  });

  it('keeps a code block and a second paragraph inside a list item, and a loose list in one list', () => {
    const list = parseMarkdown('1. 装依赖\n\n   ```sh\n   npm ci\n   ```\n\n2. 启动\n\n   然后打开页面')[0] as Extract<Block, { t: 'list' }>;
    expect(list.items).toHaveLength(2);
    expect(list.items[0].map(b => b.t)).toEqual(['p', 'code']);
    expect(list.items[0][1]).toEqual({ t: 'code', lang: 'sh', v: 'npm ci' });
    expect(list.items[1].map(b => b.t)).toEqual(['p', 'p']);
  });

  it('continues a list item onto the next line and ends the list at an unrelated paragraph', () => {
    const blocks = parseMarkdown('- 第一项\n继续这一项\n- 第二项\n\n新的段落');
    expect(blocks.map(b => b.t)).toEqual(['list', 'p']);
    expect(plain(((blocks[0] as Extract<Block, { t: 'list' }>).items[0][0] as { c: Inline[] }).c)).toBe('第一项\n继续这一项');
  });

  it('does not take prices, dashes and numbers in prose for lists', () => {
    expect(parseMarkdown('2024.01 年发布').map(b => b.t)).toEqual(['p']);
    expect(parseMarkdown('-5 度').map(b => b.t)).toEqual(['p']);
    expect(parseMarkdown('1.5 倍').map(b => b.t)).toEqual(['p']);
    expect(parseMarkdown('一二三\n- 列表').map(b => b.t)).toEqual(['p', 'list']);
  });

  it('reads tables with alignment, short rows and escaped bars', () => {
    const table = parseMarkdown('| 名称 | 数量 | 备注 |\n| :-- | --: | :-: |\n| 苹果 | 3 | a\\|b |\n| 梨 | 5 |')[0] as Extract<Block, { t: 'table' }>;
    expect(table.align).toEqual(['left', 'right', 'center']);
    expect(table.head.map(plain)).toEqual(['名称', '数量', '备注']);
    expect(table.rows.map(r => r.map(plain))).toEqual([['苹果', '3', 'a|b'], ['梨', '5', '']]);
    const plainBars = parseMarkdown('a | b\n--- | ---\n1 | 2')[0];
    expect(plainBars.t).toBe('table');
  });

  it('does not take a lone line with bars, or a half-written table, for a table', () => {
    expect(parseMarkdown('a | b').map(b => b.t)).toEqual(['p']);
    expect(parseMarkdown('| a | b |').map(b => b.t)).toEqual(['p']);
    expect(parseMarkdown('| a | b |\n| - |').map(b => b.t)).toEqual(['p']);
    expect(parseMarkdown('x\n---').map(b => b.t)).not.toContain('table');
  });

  it('handles windows line endings, empty input and a lot of nothing', () => {
    expect(parseMarkdown('a\r\n\r\nb')).toEqual([para(text('a')), para(text('b'))]);
    expect(parseMarkdown('')).toEqual([]);
    expect(parseMarkdown('\n\n   \n')).toEqual([]);
  });
});

describe('hostile and unfinished input', () => {
  it('survives very deep nesting without running out of stack', () => {
    const deepQuote = `${'> '.repeat(500)}x`;
    const deepList = Array.from({ length: 300 }, (_, i) => `${' '.repeat(i * 2)}- x`).join('\n');
    const deepEmphasis = `${'*a '.repeat(2000)}`;
    for (const source of [deepQuote, deepList, deepEmphasis, '['.repeat(5000), '`'.repeat(5000), '*'.repeat(5000), '_'.repeat(5000), '(['.repeat(3000) + ']('.repeat(100), '<'.repeat(4000), 'http://'.repeat(1500)]) {
      expect(() => parseMarkdown(source)).not.toThrow();
    }
  });

  it('stays fast on input built to make scanning quadratic', () => {
    const started = performance.now();
    for (const source of ['*a'.repeat(4000), '[a'.repeat(4000), '`a'.repeat(4000), '_a '.repeat(3000), '![a'.repeat(3000) + '](', '~~a'.repeat(3000)]) parseMarkdown(source);
    expect(performance.now() - started).toBeLessThan(2500);
  });

  it('reads every prefix of a rich answer without failing, as it arrives piece by piece', () => {
    const answer = '# 计划\n\n先看 **这里** 和 `code`，再读[文档](https://example.com/a)。\n\n- 一\n  - 二\n\n| a | b |\n| - | - |\n| 1 | 2 |\n\n```js\nconst x = `t`;\n```\n\n> 引用\n\n结束 https://example.com.';
    for (let n = 0; n <= answer.length; n++) expect(() => parseMarkdown(answer.slice(0, n)), `prefix ${n}`).not.toThrow();
    expect(parseMarkdown(answer).map(b => b.t)).toEqual(['h', 'p', 'list', 'table', 'code', 'quote', 'p']);
  });
});
