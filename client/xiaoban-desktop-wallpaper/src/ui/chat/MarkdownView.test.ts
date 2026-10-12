// @vitest-environment happy-dom
import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import MarkdownView from './MarkdownView.ts';

const render = (source: string) => mount(MarkdownView, { props: { source } });

describe('drawing an answer', () => {
  it('draws structure as elements', () => {
    const w = render('# 标题\n\n一段 **粗** 和 `码`\n\n- 甲\n- 乙\n\n3. 丙\n\n> 引\n\n---\n\n| a | b |\n| :- | -: |\n| 1 | 2 |');
    const html = w.html();
    expect(w.find('h3').text()).toBe('标题');
    expect(w.find('strong').text()).toBe('粗');
    expect(w.find('code.md-code').text()).toBe('码');
    expect(w.findAll('ul li').map(li => li.text())).toEqual(['甲', '乙']);
    expect(w.find('ol').attributes('start')).toBe('3');
    expect(w.find('blockquote').text()).toBe('引');
    expect(w.find('hr').exists()).toBe(true);
    expect(w.findAll('th').map(th => th.attributes('style'))).toEqual(['text-align: left;', 'text-align: right;']);
    expect(w.findAll('td').map(td => td.text())).toEqual(['1', '2']);
    expect(html).not.toContain('<script');
  });

  it('keeps list items that hold more than a line as blocks', () => {
    const w = render('1. 装依赖\n\n   ```sh\n   npm ci\n   ```');
    expect(w.find('li p').text()).toBe('装依赖');
    expect(w.find('li pre code').text()).toBe('npm ci');
    expect(w.find('.code-lang').text()).toBe('sh');
  });

  it('turns every kind of markup in the answer into harmless text', () => {
    const w = render('<script>alert(1)</script> <img src=x onerror=alert(1)> <b>粗</b> &lt;i&gt;\n\n```html\n<iframe src="https://evil.example"></iframe>\n```');
    expect(w.find('script').exists() || w.find('img').exists() || w.find('b').exists() || w.find('iframe').exists()).toBe(false);
    expect(w.text()).toContain('<script>alert(1)</script>');
    expect(w.text()).toContain('<img src=x onerror=alert(1)>');
    expect(w.text()).toContain('&lt;i&gt;');
    expect(w.find('pre code').text()).toBe('<iframe src="https://evil.example"></iframe>');
  });

  it('draws a link without an address the browser could follow, and asks the page to open it', async () => {
    const w = render('看[这里](https://example.com/a)，别点[坏的](javascript:alert(1))，还有 https://example.org/x。');
    const links = w.findAll('a');
    expect(links).toHaveLength(2);
    for (const link of links) {
      expect(link.attributes('href')).toBeUndefined();
      expect(link.attributes('target')).toBeUndefined();
      expect(link.attributes('role')).toBe('link');
      expect(link.attributes('tabindex')).toBe('0');
    }
    expect(links[0].attributes('title')).toBe('https://example.com/a');
    expect(w.text()).toContain('别点坏的');
    await links[0].trigger('click');
    await links[1].trigger('keydown', { key: 'Enter' });
    await links[1].trigger('keydown', { key: 'a' });
    expect(w.emitted('open')).toEqual([['https://example.com/a'], ['https://example.org/x']]);
    expect(w.html()).not.toContain('javascript:');
  });

  it('follows an answer as it is typed, even when it is half way through a code block', async () => {
    const w = render('先写');
    for (const part of ['先写 **重', '先写 **重点** 再看\n\n```py\nprint(', '先写 **重点** 再看\n\n```py\nprint(1)\n```\n完']) {
      await w.setProps({ source: part });
      expect(w.find('.markdown').exists()).toBe(true);
    }
    expect(w.find('pre code').text()).toBe('print(1)');
    expect(w.text()).toContain('完');
  });

  it('draws nothing for nothing', () => {
    expect(render('').find('.markdown').element.children).toHaveLength(0);
  });
});
