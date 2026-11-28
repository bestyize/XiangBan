import { computed, ref } from 'vue';

export type PortalView = 'home' | 'desktop' | 'mobile';

function readPath(): string {
  if (typeof window === 'undefined') return '/';
  return normalizePath(window.location.pathname);
}

function readQuery(): Record<string, string> {
  if (typeof window === 'undefined') return {};
  const out: Record<string, string> = {};
  const params = new URLSearchParams(window.location.search);
  params.forEach((value, key) => {
    out[key] = value;
  });
  return out;
}

export function normalizePath(path: string): string {
  const clean = path.replace(/\/+$/, '') || '/';
  return clean.toLowerCase();
}

export function isMobileUa(): boolean {
  if (typeof navigator === 'undefined') return false;
  return /Android|iPhone|iPad|iPod|Mobile/i.test(navigator.userAgent || '');
}

export const routePath = ref<string>(readPath());
export const routeQuery = ref<Record<string, string>>(readQuery());

export function resolveView(path: string): PortalView {
  const p = normalizePath(path);
  if (p === '/desktop' || p === '/pc') return 'desktop';
  if (p === '/app' || p === '/mobile') return 'mobile';
  return 'home';
}

export const currentView = computed<PortalView>(() => resolveView(routePath.value));

export function syncRouteFromLocation() {
  if (typeof window === 'undefined') return;
  const p = readPath();
  if (p === '/download') {
    const target = isMobileUa() ? '/app' : '/desktop';
    const nextUrl = `${target}${window.location.search}`;
    window.history.replaceState(null, '', nextUrl);
    routePath.value = target;
    routeQuery.value = readQuery();
    return;
  }
  routePath.value = p;
  routeQuery.value = readQuery();
}

export function navigate(to: string, opts?: { replace?: boolean }) {
  if (typeof window === 'undefined') return;
  const url = new URL(to, window.location.origin);
  if (url.origin !== window.location.origin) {
    window.open(url.toString(), '_blank', 'noopener,noreferrer');
    return;
  }
  let nextPath = normalizePath(url.pathname);
  if (nextPath === '/download') {
    nextPath = isMobileUa() ? '/app' : '/desktop';
  }
  const finalUrl = `${nextPath}${url.search}${url.hash}`;
  if (opts?.replace) {
    window.history.replaceState(null, '', finalUrl);
  } else {
    window.history.pushState(null, '', finalUrl);
  }
  routePath.value = nextPath;
  routeQuery.value = readQuery();
  window.scrollTo({ top: 0, behavior: 'instant' as ScrollBehavior });
}

export function isInternalPortalUrl(href: string): boolean {
  if (href.startsWith('/')) return true;
  try {
    const u = new URL(href);
    return u.hostname === 'xiaban.xyz' || u.hostname === 'www.xiaban.xyz';
  } catch {
    return false;
  }
}

export function toInternalPath(href: string): string {
  if (href.startsWith('/')) return href;
  try {
    const u = new URL(href);
    return `${u.pathname}${u.search}${u.hash}` || '/';
  } catch {
    return '/';
  }
}
