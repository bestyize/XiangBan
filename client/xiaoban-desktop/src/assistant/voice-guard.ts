import { speechText } from '../ui/speech/text.ts';
import { isStray } from '../voice/stray.ts';

function words(text: string): string {
  return speechText(text.replace(/<\|[^>]*\|>/g, '')).normalize('NFKC').toLowerCase().replace(/[^\p{L}\p{N}]/gu, '');
}

/** Reject non-speech labels and punctuation without banning ordinary short replies. */
export function isSpeechText(text: string): boolean {
  return !isStray(text) && Boolean(words(text.replace(/<\|[^>]*\|>/g, '').replace(/^\s*[（(\[]\s*(?:noise|music|silence|杂音|噪音|音乐|静音)\s*[）)\]]\s*$/i, '')));
}

/** Compare against the actual read-out text, including small ASR substitutions.
 * Matching any substring also handles echo from the middle of a long answer.
 */
export function isPlaybackEcho(text: string, playback: string): boolean {
  const query = words(text), source = words(playback);
  if (!query) return true;
  if (!source) return false;
  if (source.includes(query)) return true;
  if (query.length < 5 || query.length > 256) return false;
  const allowance = Math.min(10, Math.floor(query.length * .2));
  let previous = new Uint16Array(source.length + 1);
  let current = new Uint16Array(source.length + 1);
  for (let i = 1; i <= query.length; i++) {
    current[0] = i;
    for (let j = 1; j <= source.length; j++) {
      current[j] = Math.min(previous[j] + 1, current[j - 1] + 1, previous[j - 1] + (query[i - 1] === source[j - 1] ? 0 : 1));
    }
    [previous, current] = [current, previous];
  }
  return previous.some(distance => distance <= allowance);
}

/** A sound of listening rather than a sentence ("嗯", "哦", a laugh): it must neither interrupt the assistant nor be sent as a question. */
const FILLER = /^(?:[嗯啊哦噢喔哎唉呃额欸诶嘿哈呵]+|嗯哼|哈哈+|呵呵+)$/;
export function isFiller(text: string): boolean {
  return FILLER.test(words(text));
}
