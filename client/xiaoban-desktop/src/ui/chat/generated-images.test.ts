import { describe, expect, it } from 'vitest';
import { generatedImages } from './generated-images.ts';
import { applyEvent, itemsFromHistory, newView } from './view.ts';
import { replyBlocks } from './replies.ts';
import { withBuiltinTool } from './builtin-tools.ts';

const image = { id: 'a'.repeat(64) + '.png', name: '小伴生图.png', mime: 'image/png', width: 1024, height: 1024 };
const metadata = { generated_images: [image] };
describe('native generated image references', () => {
  it('accepts native GIF references and rejects remote URLs masquerading as images', () => {
    const gif = {...image, id:'b'.repeat(64) + '.gif', mime:'image/gif'};
    expect(generatedImages({generated_images:[gif]})).toEqual([gif]);
    expect(generatedImages({generated_images:[{...gif,id:'https://example.com/a.gif'}]})).toEqual([]);
  });
  it('keeps the same gallery after live events and a history reload', () => {
    const view = newView('s1');
    applyEvent(view, { type: 'tool_start', id: 'g1', name: 'image_generate', arguments: '{"prompt":"cat"}' });
    applyEvent(view, { type: 'tool_end', id: 'g1', output: '已生成', is_error: false, structured_content: metadata });
    const history = itemsFromHistory(newView('s1'), [
      { role: 'assistant', tool_calls: [{ id: 'g1', name: 'image_generate', arguments: '{"prompt":"cat"}' }] },
      { role: 'tool', tool_call_id: 'g1', parts: [{ type: 'text', text: '已生成' }], structured_content: metadata },
    ]);
    expect(view.items).toEqual(history);
    expect(replyBlocks(history)[0]?.calls[0]?.generatedImages).toEqual([image]);
  });
  it('rejects malformed references and never displays third-party or failed tool payloads as native images', () => {
    for (const invalid of [null, {}, { generated_images: null }, { generated_images: [{ ...image, id: '../secret.png' }] }, { generated_images: [{ ...image, width: 999999 }] }, { generated_images: [{ ...image, mime: 'image/svg+xml' }] }]) expect(generatedImages(invalid)).toEqual([]);
    for (const [name, error] of [['mcp_image', false], ['image_generate', true]] as const) {
      const view = newView('s');
      applyEvent(view, { type: 'tool_start', id: 'g', name, arguments: '{}' });
      applyEvent(view, { type: 'tool_end', id: 'g', output: '', is_error: error, structured_content: metadata });
      expect(view.items[0]).not.toHaveProperty('generatedImages');
    }
  });
  it('persists the user-selected image tool in the submitted text', () => {
    expect(withBuiltinTool('一只猫', 'image')).toBe('请使用 image_generate 生成图片：\n一只猫');
  });
});
