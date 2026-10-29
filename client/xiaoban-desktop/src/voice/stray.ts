/**
 * The recogniser guesses the language, and on noise, a breath, a clipped syllable or the assistant's own voice
 * coming back through the speakers it tends to answer with a stray English word, most often "The". Those are not
 * something the user said: they must neither interrupt a reading nor be sent as a question.
 */
const STRAY = new Set([
  'the', 'a', 'an', 'i', 'it', 'is', 'and', 'so', 'to', 'of', 'in', 'on', 'at', 'he', 'she', 'we', 'they', 'you', 'that', 'this',
  'but', 'or', 'as', 'be', 'if', 'uh', 'um', 'er', 'ah', 'oh', 'hmm', 'huh', 'mm', 'yeah', 'yes', 'no', 'well', 'what', 'there',
]);
const CJK = /[㐀-鿿぀-ヿ가-힯]/;

/** Nothing but a few filler or function words of another language, and none of the user's own. */
export function isStray(text: string): boolean {
  const plain = text.replace(/<\|[^>]*\|>/g, '').normalize('NFKC').toLowerCase();
  if (CJK.test(plain)) return false;
  const words = plain.split(/[^\p{L}\p{N}']+/u).filter(Boolean);
  return words.length > 0 && words.length <= 4 && words.every(word => STRAY.has(word));
}
