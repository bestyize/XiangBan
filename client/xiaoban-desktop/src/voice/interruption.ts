/**
 * Notes for the model that travel at the front of the user's next message and are cut off again for display, so
 * only the model reads them:
 * - when the user talks over an answer, how much of it was actually heard (it wrote all of it, the user heard the start);
 * - when the assistant has already said something aloud before answering ("你好呀，我在"), what that was, so the answer
 *   does not say it again.
 */
export const INTERRUPTION_MARK = '【语音打断】';
export const SPOKEN_MARK = '【语音接话】';
const NOTES = /^(?:【语音(?:打断|接话)】[^\n]*\n)+/;
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

/** The assistant has already said `phrase` aloud as the opening of this answer. */
export function spokenNote(phrase: string): string {
  return `${SPOKEN_MARK}你已经先对用户说出了「${phrase}」，不要重复这句话，直接接着往下回应。\n`;
}

/** The message as the user wrote it. */
export function withoutVoiceNotes(text: string): string {
  return text.replace(NOTES, '');
}
