// 内置已知高质量软件的专属矢量高质感图标（零外部网络依赖，0ms 加载，绝不失效，绝不闪烁）
const KNOWN_ICONS: Record<string, { bg: string; svg: string }> = {
  'claude-desktop': {
    bg: '#d97757',
    svg: `<path d="M32 16 L34 27 L43 21 L37 30 L48 32 L37 34 L43 43 L34 37 L32 48 L30 37 L21 43 L27 34 L16 32 L27 30 L21 21 L30 27 Z" fill="white"/>`,
  },
  'codex-editor': {
    bg: '#007acc',
    svg: `<path d="M44 19 L33 28 L44 37 L41 42 L26 29 L26 27 L41 14 Z" fill="white"/><path d="M20 19 L31 28 L20 37 L23 42 L38 29 L38 27 L23 14 Z" fill="white" opacity="0.6"/>`,
  },
  'ccswitch': {
    bg: '#4f46e5',
    svg: `<rect x="16" y="24" width="32" height="16" rx="8" fill="none" stroke="white" stroke-width="4"/><circle cx="24" cy="32" r="5" fill="white"/>`,
  },
  'ollama': {
    bg: '#111827',
    svg: `<circle cx="32" cy="32" r="14" fill="none" stroke="white" stroke-width="4"/><circle cx="27" cy="30" r="2.5" fill="white"/><circle cx="37" cy="30" r="2.5" fill="white"/><path d="M28 36 Q32 39 36 36" stroke="white" stroke-width="3" fill="none" stroke-linecap="round"/>`,
  },
  'photocraft': {
    bg: '#001e36',
    svg: `<text x="24" y="42" font-family="-apple-system, sans-serif" font-size="28" font-weight="800" fill="#31a8ff">P</text><text x="40" y="42" font-family="-apple-system, sans-serif" font-size="22" font-weight="700" fill="#31a8ff">s</text>`,
  },
  'patchy': {
    bg: '#059669',
    svg: `<path d="M18 36 L18 28 M24 44 L24 20 M32 48 L32 16 M40 42 L40 22 M46 36 L46 28" stroke="white" stroke-width="4" stroke-linecap="round"/>`,
  },
  'typora': {
    bg: '#27272a',
    svg: `<path d="M20 22 L44 22 M32 22 L32 46" stroke="white" stroke-width="5" stroke-linecap="round"/>`,
  },
  'iina': {
    bg: '#1d4ed8',
    svg: `<polygon points="26,20 44,32 26,44" fill="white"/>`,
  },
  'maccy': {
    bg: '#ea580c',
    svg: `<rect x="20" y="20" width="24" height="28" rx="4" fill="none" stroke="white" stroke-width="4"/><path d="M26 16 L38 16" stroke="white" stroke-width="4" stroke-linecap="round"/><line x1="26" y1="28" x2="38" y2="28" stroke="white" stroke-width="3"/><line x1="26" y1="36" x2="34" y2="36" stroke="white" stroke-width="3"/>`,
  },
  'localsend': {
    bg: '#0d9488',
    svg: `<circle cx="32" cy="32" r="16" fill="none" stroke="white" stroke-width="3"/><polygon points="26,22 42,32 26,42 30,32" fill="white"/>`,
  },
  'raycast': {
    bg: '#ff6363',
    svg: `<path d="M32 14 L36 28 L50 32 L36 36 L32 50 L28 36 L14 32 L28 28 Z" fill="white"/>`,
  },
  'lemon-cleaner': {
    bg: '#f59e0b',
    svg: `<circle cx="32" cy="32" r="16" fill="none" stroke="white" stroke-width="4"/><line x1="32" y1="16" x2="32" y2="32" stroke="white" stroke-width="3"/><line x1="32" y1="32" x2="43" y2="43" stroke="white" stroke-width="3"/><line x1="32" y1="32" x2="21" y2="43" stroke="white" stroke-width="3"/>`,
  },
  'shottr': {
    bg: '#2563eb',
    svg: `<circle cx="32" cy="32" r="15" fill="none" stroke="white" stroke-width="4"/><circle cx="32" cy="32" r="6" fill="white"/><line x1="32" y1="10" x2="32" y2="15" stroke="white" stroke-width="3"/><line x1="32" y1="49" x2="32" y2="54" stroke="white" stroke-width="3"/><line x1="10" y1="32" x2="15" y2="32" stroke="white" stroke-width="3"/><line x1="49" y1="32" x2="54" y2="32" stroke="white" stroke-width="3"/>`,
  },
  'ghostty': {
    bg: '#18181b',
    svg: `<path d="M20 44 L20 28 C20 21.4 25.4 16 32 16 C38.6 16 44 21.4 44 28 L44 44 L39 40 L34 44 L29 40 L25 44 Z" fill="white"/><circle cx="28" cy="27" r="2.5" fill="#18181b"/><circle cx="36" cy="27" r="2.5" fill="#18181b"/>`,
  },
  'orbstack': {
    bg: '#0284c7',
    svg: `<circle cx="32" cy="32" r="8" fill="white"/><ellipse cx="32" cy="32" rx="20" ry="8" fill="none" stroke="white" stroke-width="3" transform="rotate(-30 32 32)"/>`,
  },
  'ice': {
    bg: '#38bdf8',
    svg: `<path d="M32 16 L32 48 M16 32 L48 32 M21 21 L43 43 M21 43 L43 21" stroke="white" stroke-width="4" stroke-linecap="round"/>`,
  },
  'obsidian': {
    bg: '#7c3aed',
    svg: `<polygon points="32,14 46,24 38,48 26,48 18,24" fill="none" stroke="white" stroke-width="3.5"/><polygon points="32,14 38,48 26,48" fill="white" opacity="0.4"/>`,
  },
  'xiaoban-desktop': {
    bg: '#2563eb',
    svg: `<rect x="18" y="20" width="28" height="20" rx="3" fill="none" stroke="white" stroke-width="3"/><path d="M22 36 L27 29 L33 34 L38 27 L42 36 Z" fill="white"/><line x1="28" y1="44" x2="36" y2="44" stroke="white" stroke-width="3"/>`,
  },
};

const PALETTES = [
  '#2563eb',
  '#0f766e',
  '#4f46e5',
  '#059669',
  '#7c3aed',
  '#0284c7',
  '#334155',
  '#dc2626',
  '#c026d3',
];

function hashString(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = (hash << 5) - hash + str.charCodeAt(i);
    hash |= 0;
  }
  return Math.abs(hash);
}

export function getAppIcon(slug?: string, name?: string): string {
  if (slug && KNOWN_ICONS[slug]) {
    const item = KNOWN_ICONS[slug];
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="64" height="64"><rect width="64" height="64" rx="14" fill="${item.bg}"/>${item.svg}</svg>`;
    return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
  }

  // 首字母通用优雅兜底
  const cleanName = (name || 'App').trim();
  let initials = cleanName.slice(0, 1);
  if (/^[a-zA-Z]/.test(cleanName)) {
    initials = cleanName.slice(0, Math.min(cleanName.length, 2)).toUpperCase();
  }

  const color = PALETTES[hashString(cleanName) % PALETTES.length];
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="64" height="64"><rect width="64" height="64" rx="14" fill="${color}"/><text x="32" y="38" font-family="-apple-system, BlinkMacSystemFont, 'SF Pro Display', sans-serif" font-size="${initials.length > 1 ? 21 : 26}" font-weight="700" fill="white" text-anchor="middle" dominant-baseline="central">${initials}</text></svg>`;
  return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
}

export function handleImageFallback(e: Event, name?: string, slug?: string) {
  const target = e.target as HTMLImageElement;
  if (!target) return;
  // 必须解除 onerror 事件，防止发生无限循环重载和闪烁
  target.onerror = null;
  target.src = getAppIcon(slug, name);
}

export function getAppIconUrl(app: { slug: string; name: string; icon_url?: string }): string {
  // 如果是内置高质量软件，优先直接使用内置高精度矢量图标（零网络开销、瞬间秒开、永不闪烁）
  if (app.slug && KNOWN_ICONS[app.slug]) {
    return getAppIcon(app.slug, app.name);
  }
  // 如果自定义了可访问的有效图标则使用
  if (app.icon_url && !app.icon_url.includes('raw.githubusercontent.com')) {
    return app.icon_url;
  }
  return getAppIcon(app.slug, app.name);
}
