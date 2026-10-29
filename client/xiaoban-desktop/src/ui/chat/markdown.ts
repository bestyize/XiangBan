/**
 * A small Markdown reader for chat answers.
 *
 * It produces a tree of plain nodes, never HTML: the renderer puts every piece of text into the page
 * as text, so nothing the model (or a web page it read) writes can add markup. Links are kept only
 * for http, https and mailto; images are shown as links and never loaded. Input that is still being
 * typed (an open code fence, a half-written table) is read in the most natural way instead of failing.
 */

export type Inline =
  | { t: 'text'; v: string }
  | { t: 'code'; v: string }
  | { t: 'br' }
  | { t: 'strong' | 'em' | 'del'; c: Inline[] }
  | { t: 'link'; href: string; c: Inline[] };

export type Align = null | 'left' | 'center' | 'right';
export type Block =
  | { t: 'p'; c: Inline[] }
  | { t: 'h'; level: number; c: Inline[] }
  | { t: 'code'; lang: string; v: string }
  | { t: 'list'; ordered: boolean; start: number; items: Block[][] }
  | { t: 'quote'; c: Block[] }
  | { t: 'hr' }
  | { t: 'table'; align: Align[]; head: Inline[][]; rows: Inline[][][] };

const MAX_DEPTH = 8;
const EMPHASIS_WINDOW = 1500;
const BRACKET_WINDOW = 1000;
const CODE_WINDOW = 4000;
const SAFE_SCHEMES = new Set(['http:', 'https:', 'mailto:']);

/** The address if it is safe to link to, otherwise null. Relative and scheme-less addresses are not links. */
export function safeHref(raw: string): string | null {
  const url = raw.trim();
  // eslint-disable-next-line no-control-regex
  if (!url || url.length > 2048 || /[\s\u0000-\u001f\u007f]/.test(url)) return null;
  try {
    return SAFE_SCHEMES.has(new URL(url).protocol) ? url : null;
  } catch { return null; }
}

const isWord = (c: string | undefined) => !!c && /[\p{L}\p{N}]/u.test(c);
const isSpace = (c: string | undefined) => !c || /\s/.test(c);

function tryCodeSpan(src: string, i: number): { node: Inline; end: number } | { literal: string; end: number } {
  let n = 0;
  while (src[i + n] === '`') n++;
  const limit = Math.min(src.length, i + n + CODE_WINDOW);
  let j = i + n;
  while (j < limit) {
    const k = src.indexOf('`', j);
    if (k < 0 || k >= limit) break;
    let m = 0;
    while (src[k + m] === '`') m++;
    if (m === n) {
      let v = src.slice(i + n, k).replace(/\n/g, ' ');
      if (v.length > 2 && v.startsWith(' ') && v.endsWith(' ') && v.trim()) v = v.slice(1, -1);
      return { node: { t: 'code', v }, end: k + n };
    }
    j = k + m;
  }
  return { literal: '`'.repeat(n), end: i + n };
}

function tryLink(src: string, open: number): { text: string; url: string; end: number } | null {
  let depth = 0;
  let j = open;
  const limit = Math.min(src.length, open + BRACKET_WINDOW);
  for (; j < limit; j++) {
    const c = src[j];
    if (c === '\\') { j++; continue; }
    if (c === '[') depth++;
    else if (c === ']' && --depth === 0) break;
  }
  if (j >= limit || src[j] !== ']' || src[j + 1] !== '(') return null;
  let k = j + 2;
  while (src[k] === ' ') k++;
  let url = '';
  if (src[k] === '<') {
    const end = src.indexOf('>', k);
    if (end < 0 || end - k > 2100) return null;
    url = src.slice(k + 1, end);
    k = end + 1;
  } else {
    const start = k;
    let nest = 0;
    while (k < src.length && k - start < 2100) {
      const c = src[k];
      if (c === '\\') { k += 2; continue; }
      if (/\s/.test(c)) break;
      if (c === '(') nest++;
      else if (c === ')') { if (nest === 0) break; nest--; }
      k++;
    }
    url = src.slice(start, k);
  }
  while (src[k] === ' ') k++;
  if (src[k] === '"' || src[k] === "'") {
    const end = src.indexOf(src[k], k + 1);
    if (end < 0) return null;
    k = end + 1;
    while (src[k] === ' ') k++;
  }
  return src[k] === ')' ? { text: src.slice(open + 1, j), url, end: k + 1 } : null;
}

function tryEmphasis(src: string, i: number, depth: number, inLink: boolean): { node: Inline; end: number } | null {
  const delim = src.startsWith('***', i) ? '***' : src.startsWith('**', i) ? '**' : src.startsWith('__', i) ? '__' : src.startsWith('~~', i) ? '~~' : src[i];
  if (isSpace(src[i + delim.length])) return null;
  if (delim[0] === '_' && isWord(src[i - 1])) return null;
  const limit = Math.min(src.length, i + delim.length + EMPHASIS_WINDOW);
  let j = i + delim.length;
  while (j < limit) {
    const k = src.indexOf(delim, j);
    if (k < 0 || k >= limit) return null;
    const after = src[k + delim.length];
    const doubled = delim.length === 1 && (after === delim || src[k - 1] === delim);
    const closes = k > i + delim.length && !isSpace(src[k - 1]) && src[k - 1] !== '\\' && !doubled && !(delim[0] === '_' && isWord(after));
    if (closes) {
      const c = parseInline(src.slice(i + delim.length, k), depth + 1, inLink);
      const t = delim === '~~' ? 'del' : delim.length === 1 ? 'em' : 'strong';
      const node: Inline = delim === '***' ? { t: 'strong', c: [{ t: 'em', c }] } : { t, c };
      return { node, end: k + delim.length };
    }
    j = k + delim.length;
  }
  return null;
}

function trimUrl(url: string): string {
  let out = url;
  for (;;) {
    const last = out[out.length - 1];
    if (last && '.,;:!?\'"'.includes(last)) out = out.slice(0, -1);
    else if (last === ')' && (out.match(/\)/g)?.length ?? 0) > (out.match(/\(/g)?.length ?? 0)) out = out.slice(0, -1);
    else return out;
  }
}

export function parseInline(src: string, depth = 0, inLink = false): Inline[] {
  const out: Inline[] = [];
  let text = '';
  const flush = () => { if (text) { out.push({ t: 'text', v: text }); text = ''; } };
  let i = 0;
  while (i < src.length) {
    const ch = src[i];
    if (ch === '\\' && i + 1 < src.length && /[\\`*_{}[\]()#+\-.!|~<>]/.test(src[i + 1])) { text += src[i + 1]; i += 2; continue; }
    if (ch === '\n') { text = text.replace(/ +$/, ''); flush(); out.push({ t: 'br' }); i++; continue; }
    if (ch === '`') {
      const found = tryCodeSpan(src, i);
      if ('node' in found) { flush(); out.push(found.node); } else text += found.literal;
      i = found.end;
      continue;
    }
    if (ch === '[' || (ch === '!' && src[i + 1] === '[')) {
      const image = ch === '!';
      const link = !inLink || image ? tryLink(src, image ? i + 1 : i) : null;
      if (link) {
        const href = safeHref(link.url);
        const label = parseInline(link.text, depth + 1, true);
        flush();
        if (!href) out.push(...(label.length ? label : [{ t: 'text' as const, v: link.url }]));
        else out.push({ t: 'link', href, c: label.length ? label : [{ t: 'text', v: image ? '图片' : href }] });
        i = link.end;
        continue;
      }
    }
    if (ch === '<' && !inLink) {
      const auto = /^<((?:https?:\/\/|mailto:)[^\s<>]+)>/.exec(src.slice(i, i + 2200));
      const href = auto && safeHref(auto[1]);
      if (auto && href) { flush(); out.push({ t: 'link', href, c: [{ t: 'text', v: auto[1] }] }); i += auto[0].length; continue; }
    }
    if ((ch === '*' || ch === '_' || ch === '~') && depth < MAX_DEPTH && (ch !== '~' || src[i + 1] === '~')) {
      const found = tryEmphasis(src, i, depth, inLink);
      if (found) { flush(); out.push(found.node); i = found.end; continue; }
    }
    if (ch === 'h' && !inLink && !isWord(src[i - 1]) && /^https?:\/\//.test(src.slice(i, i + 8))) {
      // Stops at CJK and full-width punctuation too: in Chinese text it follows an address with no space.
      const match = /^https?:\/\/[^\s<>"`\u2018-\u201f\u3000-\u303f\uff00-\uffef]+/.exec(src.slice(i, i + 2100));
      const url = match && trimUrl(match[0]);
      const href = url && safeHref(url);
      if (url && href) { flush(); out.push({ t: 'link', href, c: [{ t: 'text', v: url }] }); i += url.length; continue; }
    }
    text += ch;
    i++;
  }
  flush();
  return out;
}

const FENCE = /^( {0,3})(`{3,}|~{3,})\s*([^\s`]*)[^`]*$/;
const HEADING = /^ {0,3}(#{1,6})(?:[ \t]+(.*?))?[ \t]*$/;
const HR = /^ {0,3}([-*_])(?:[ \t]*\1){2,}[ \t]*$/;
const QUOTE = /^ {0,3}>/;
const ITEM = /^( {0,3})([-*+]|\d{1,9}[.)])([ \t]+)(.*)$/;
const SEPARATOR = /^[ \t]*\|?[ \t]*:?-+:?[ \t]*(\|[ \t]*:?-+:?[ \t]*)*\|?[ \t]*$/;
const blank = (line: string) => line.trim() === '';
const indentOf = (line: string) => line.length - line.trimStart().length;

/** Splits a table row on `|`, keeping `\|` as a literal bar. (No regex look-behind: older WebKit lacks it.) */
function cells(line: string): string[] {
  let body = line.trim();
  if (body.startsWith('|')) body = body.slice(1);
  if (body.endsWith('|') && !body.endsWith('\\|')) body = body.slice(0, -1);
  const out: string[] = [];
  let cell = '';
  for (let k = 0; k < body.length; k++) {
    if (body[k] === '\\' && body[k + 1] === '|') { cell += '|'; k++; }
    else if (body[k] === '|') { out.push(cell.trim()); cell = ''; }
    else cell += body[k];
  }
  out.push(cell.trim());
  return out;
}

/** A header row with bars, then a delimiter row with bars and the same number of columns. */
function tableAt(lines: string[], i: number): boolean {
  return i + 1 < lines.length && lines[i].includes('|') && lines[i + 1].includes('|') && SEPARATOR.test(lines[i + 1]) && cells(lines[i]).length === cells(lines[i + 1]).length;
}

/** Whether the line starts a new block, and so ends a paragraph. Lists and quotes only count where they are read. */
function startsBlock(lines: string[], i: number, nested: boolean): boolean {
  const line = lines[i];
  return FENCE.test(line) || HEADING.test(line) || HR.test(line) || tableAt(lines, i) || (nested && (QUOTE.test(line) || ITEM.test(line)));
}

function parseTable(lines: string[], start: number): [Block, number] {
  const head = cells(lines[start]);
  const align: Align[] = cells(lines[start + 1]).map(spec => (spec.startsWith(':') && spec.endsWith(':') ? 'center' : spec.endsWith(':') ? 'right' : spec.startsWith(':') ? 'left' : null));
  const rows: Inline[][][] = [];
  let i = start + 2;
  while (i < lines.length && !blank(lines[i]) && lines[i].includes('|') && rows.length < 500) {
    const row = cells(lines[i]);
    rows.push(head.map((_, n) => parseInline(row[n] ?? '')));
    i++;
  }
  return [{ t: 'table', align, head: head.map(cell => parseInline(cell)), rows }, i];
}

function parseList(lines: string[], start: number, depth: number): [Block, number] {
  const first = ITEM.exec(lines[start])!;
  const ordered = /\d/.test(first[2][0]);
  const items: Block[][] = [];
  let i = start;
  while (i < lines.length) {
    const m = ITEM.exec(lines[i]);
    if (!m || /\d/.test(m[2][0]) !== ordered) break;
    const markerIndent = m[1].length;
    const spaces = m[3].replace(/\t/g, '    ');
    // Five or more spaces after the marker make the content start with an indented block instead.
    const contentIndent = markerIndent + m[2].length + (spaces.length > 4 ? 1 : spaces.length);
    const own: string[] = [(spaces.length > 4 ? spaces.slice(1) : '') + m[4]];
    i++;
    while (i < lines.length) {
      const line = lines[i];
      if (blank(line)) {
        let j = i;
        while (j < lines.length && blank(lines[j])) j++;
        const indent = j < lines.length ? indentOf(lines[j]) : -1;
        if (j < lines.length && (indent >= contentIndent || (ITEM.test(lines[j]) && indent > markerIndent))) { own.push(''); i++; continue; }
        break;
      }
      const indent = indentOf(line);
      if (indent >= contentIndent || (indent > markerIndent && ITEM.test(line))) { own.push(line.slice(Math.min(indent, contentIndent))); i++; continue; }
      // A line that is not a new block continues the paragraph above, even without indentation.
      if (!startsBlock(lines, i, true) && !blank(own[own.length - 1])) { own.push(line.trim()); i++; continue; }
      break;
    }
    items.push(parseBlocks(own, depth + 1));
    let j = i;
    while (j < lines.length && blank(lines[j])) j++;
    const next = j < lines.length ? ITEM.exec(lines[j]) : null;
    if (next && /\d/.test(next[2][0]) === ordered && next[1].length <= markerIndent + 3) i = j;
    else break;
  }
  return [{ t: 'list', ordered, start: ordered ? parseInt(first[2], 10) : 1, items }, i];
}

export function parseBlocks(lines: string[], depth = 0): Block[] {
  const blocks: Block[] = [];
  const nested = depth < MAX_DEPTH;
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    if (blank(line)) { i++; continue; }
    const fence = FENCE.exec(line);
    if (fence) {
      const [, indent, marks, lang] = fence;
      const body: string[] = [];
      i++;
      while (i < lines.length) {
        const close = /^ {0,3}(`{3,}|~{3,})\s*$/.exec(lines[i]);
        if (close && close[1][0] === marks[0] && close[1].length >= marks.length) { i++; break; }
        body.push(lines[i].replace(new RegExp(`^ {0,${indent.length}}`), ''));
        i++;
      }
      blocks.push({ t: 'code', lang, v: body.join('\n') });
      continue;
    }
    const heading = HEADING.exec(line);
    if (heading) {
      blocks.push({ t: 'h', level: heading[1].length, c: parseInline((heading[2] ?? '').replace(/[ \t]+#+[ \t]*$/, '').replace(/^#+$/, '')) });
      i++;
      continue;
    }
    if (HR.test(line)) { blocks.push({ t: 'hr' }); i++; continue; }
    if (nested && QUOTE.test(line)) {
      const inner: string[] = [];
      while (i < lines.length && QUOTE.test(lines[i])) { inner.push(lines[i].replace(/^ {0,3}> ?/, '')); i++; }
      blocks.push({ t: 'quote', c: parseBlocks(inner, depth + 1) });
      continue;
    }
    if (nested && ITEM.test(line)) {
      const [block, next] = parseList(lines, i, depth);
      blocks.push(block);
      i = next;
      continue;
    }
    if (tableAt(lines, i)) {
      const [block, next] = parseTable(lines, i);
      blocks.push(block);
      i = next;
      continue;
    }
    const paragraph = [line.trim()];
    i++;
    while (i < lines.length && !blank(lines[i]) && !startsBlock(lines, i, nested)) { paragraph.push(lines[i].trim()); i++; }
    blocks.push({ t: 'p', c: parseInline(paragraph.join('\n')) });
  }
  return blocks;
}

export function parseMarkdown(source: string): Block[] {
  return parseBlocks(source.replace(/\r\n?/g, '\n').split('\n'));
}
