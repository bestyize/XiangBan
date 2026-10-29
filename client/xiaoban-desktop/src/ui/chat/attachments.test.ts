// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest';
import { attachmentCards, readAttachment } from './attachments.ts';

describe('unified attachments', () => {
  it('retains image data URLs and gives documents named payloads', async () => {
    expect(await readAttachment(new File(['pixel'], 'photo.png', {type:'image/png'}))).toMatch(/^data:image\/png;base64,/);
    expect(await readAttachment(new File(['amount\n12'], '预算.csv', {type:'text/csv'}))).toMatchObject({name:'预算.csv',size:9});
  });
  it('rejects unsupported binaries before reading', async () => {
    await expect(readAttachment(new File(['binary'],'program.exe'))).rejects.toThrow('暂不支持');
  });
  it('restores only valid document cards from trusted saved metadata', () => {
    expect(attachmentCards({attachments:[{name:'report.pdf',size:123,mime:'application/pdf'},{name:'image.png',size:2,mime:'image/png'},{name:12}]})).toEqual([{name:'report.pdf',size:123,mime:'application/pdf'}]);
  });
});

