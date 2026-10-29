import type { GameManifest } from './protocol.ts';

type CoverGame = Pick<GameManifest, 'id' | 'name' | 'description'>;
const palettes = [
  ['#123c39', '#26685b', '#b6f5bf'],
  ['#24335c', '#475e96', '#ffd58b'],
  ['#483147', '#80546b', '#ffcfac'],
  ['#31335c', '#6861a3', '#d9cdff'],
] as const;
const escapeXml = (text: string) => text.replace(/[&<>"']/g, char => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&apos;' })[char]!);

function titleLines(name: string): string[] {
  const lines = [''];
  let width = 0;
  for (const char of name.replace(/[\u0000-\u001f\u007f]/g, ' ').trim()) {
    const size = /[\u0020-\u007e]/.test(char) ? 0.55 : 1;
    if (width + size > 9) {
      if (lines.length === 2) { lines[1] += '…'; break; }
      lines.push(''); width = 0;
    }
    lines[lines.length - 1] += char;
    width += size;
  }
  return lines;
}

/** Offline artwork for older packs and games whose custom cover cannot be loaded. */
export function generatedCover(game: CoverGame): string {
  let hash = 2166136261;
  for (const char of game.id) hash = Math.imul(hash ^ char.charCodeAt(0), 16777619);
  const [background, glow, accent] = palettes[(hash >>> 0) % palettes.length];
  const subject = `${game.id} ${game.name} ${game.description}`;
  let artwork: string;
  if (/snake|贪吃蛇|小蛇/i.test(subject)) {
    artwork = `<path d="M402 250h94a26 26 0 0 0 0-52h-47a26 26 0 0 1 0-52h64" fill="none" stroke="${accent}" stroke-width="28" stroke-linecap="round"/><circle cx="519" cy="140" r="3" fill="${background}"/><circle cx="519" cy="152" r="3" fill="${background}"/><circle cx="559" cy="104" r="12" fill="#ff9b89"/><path d="M559 89q0-12 10-12" fill="none" stroke="${accent}" stroke-width="4"/>`;
  } else if (/mario|platform|玛丽|马里奥|平台|跳跃|冒险/i.test(subject)) {
    artwork = `<g fill="${accent}"><rect x="365" y="260" width="80" height="22" rx="6"/><rect x="474" y="218" width="76" height="22" rx="6"/><rect x="550" y="154" width="54" height="22" rx="6"/></g><path d="M399 228q48-111 100-52" fill="none" stroke="#fff" stroke-opacity=".4" stroke-width="3" stroke-dasharray="6 9"/><rect x="465" y="154" width="30" height="36" rx="12" fill="#fff"/><circle cx="485" cy="165" r="3" fill="${background}"/><path d="m567 109 5-10 5 10 11 2-8 8 2 11-10-5-10 5 2-11-8-8z" fill="${accent}"/>`;
  } else if (/2048|tetris|block|方块|消除|拼图|数独/i.test(subject)) {
    artwork = `<g transform="translate(385 104) rotate(-9 90 90)" fill="${accent}"><rect width="66" height="66" rx="14" opacity=".35"/><rect x="78" width="66" height="66" rx="14"/><rect y="78" width="66" height="66" rx="14"/><rect x="78" y="78" width="66" height="66" rx="14" opacity=".65"/><rect x="156" y="78" width="42" height="66" rx="14" opacity=".3"/></g><path d="m555 81 5-12 5 12 12 5-12 5-5 12-5-12-12-5z" fill="#fff"/>`;
  } else {
    artwork = `<g transform="translate(377 133) rotate(-10 100 60)"><path d="M52 0h96c24 0 36 20 43 48l10 41c5 24-20 36-35 18l-24-27H58l-24 27C19 125-6 113-1 89L9 48C16 20 28 0 52 0Z" fill="${accent}"/><path d="M36 40h30M51 25v30" stroke="${background}" stroke-width="8" stroke-linecap="round"/><circle cx="143" cy="32" r="7" fill="${background}"/><circle cx="162" cy="51" r="7" fill="${background}"/></g><path d="m533 81 5-12 5 12 12 5-12 5-5 12-5-12-12-5z" fill="${accent}"/>`;
  }
  const titles = titleLines(game.name).map((line, index) => `<tspan x="38" y="${155 + index * 42}">${escapeXml(line)}</tspan>`).join('');
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="640" height="360" viewBox="0 0 640 360"><defs><radialGradient id="glow"><stop stop-color="${glow}"/><stop offset="1" stop-color="${background}"/></radialGradient><pattern id="dots" width="24" height="24" patternUnits="userSpaceOnUse"><circle cx="1" cy="1" r="1" fill="#fff" opacity=".07"/></pattern></defs><rect width="640" height="360" fill="${background}"/><circle cx="490" cy="175" r="270" fill="url(#glow)"/><rect width="640" height="360" fill="url(#dots)"/><circle cx="485" cy="180" r="124" fill="none" stroke="#fff" stroke-opacity=".08"/><text x="38" y="49" fill="${accent}" font-family="sans-serif" font-size="12" font-weight="600" letter-spacing="3">MY ARCADE</text><text fill="#fff" font-family="'Microsoft YaHei','PingFang SC',sans-serif" font-size="30" font-weight="700">${titles}</text><path d="M38 245h32" stroke="${accent}" stroke-width="3" stroke-linecap="round"/><text x="38" y="287" fill="#fff" opacity=".6" font-family="'Microsoft YaHei','PingFang SC',sans-serif" font-size="13" letter-spacing="2">我的游戏 · 随时开玩</text>${artwork}</svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}
