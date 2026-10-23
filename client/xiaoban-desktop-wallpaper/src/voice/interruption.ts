/**
 * When the user talks over an answer, the model should know how much of it was actually heard: it wrote the whole
 * answer, the user heard the first part. The note travels at the front of the user's next message, and the
 * conversation view cuts it off again, so it is read by the model only.
 */
export const INTERRUPTION_MARK = '【语音打断】';
const NOTE = /^【语音打断】[^\n]*\n/;
const HEAD = 40, TAIL = 100;

/** `spoken` is what had started playing, or null when it is not known how far the reading got. */
export function interruptionNote(spoken: string | null): string {
  const heard = (spoken ?? '').replace(/\s+/g, ' ').trim();
  const excerpt = heard.length > HEAD + TAIL ? `${heard.slice(0, HEAD)}……${heard.slice(-TAIL)}` : heard;
  const body = spoken === null
    ? '具体听到哪里不详。'
    : excerpt ? `用户听到的是：「${excerpt}」，最后一句可能没听完，后面的内容没有听到。` : '用户还没听到任何内容就打断了。';
  return `${INTERRUPTION_MARK}你上一条回答被用户开口打断了。${body}请直接回应用户这次说的话；没听到的部分如果仍然重要，再简短补上。\n`;
}

/** The message as the user wrote it. */
export function withoutInterruptionNote(text: string): string {
  return text.replace(NOTE, '');
}
