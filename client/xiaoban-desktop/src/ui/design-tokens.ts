import { accent as currentAccent, accentPalette, type AccentId, type AccentPalette } from './appearance.ts';

/**
 * Shared desktop UI tokens (方案四). Media artwork and the pet renderer keep their own colors.
 * Every token becomes a CSS variable `--ui-<kebab-name>` on <html>; components must read colours through them.
 */
export const lightTokens = {
  // Theme colour roles; overwritten from the chosen accent by `applyUiTokens`.
  btn: '#D93A44', btnHi: '#E8565F', soft: '#FCEBEC', ink: '#B42A33', tint: '#F5F1F1', dot: '#E5484D',
  // Legacy names, mapped to the theme colour so older pages follow it.
  primary: '#D93A44', primaryHover: '#E8565F', primaryPressed: '#B42A33', primarySoft: '#FCEBEC', primaryText: '#B42A33',
  // Neutrals.
  appBg: '#F5F1F1', sidebarBg: '#F5F1F1', surface: '#FFFFFF', surfaceAlt: '#F2F2F4', surfaceSunken: '#F5F5F7', surfaceHover: 'rgba(0,0,0,.045)',
  surfaceRaised: 'rgba(255,255,255,.97)', control: '#F2F2F4', controlHover: '#EAEAED',
  border: 'rgba(0,0,0,.08)', borderStrong: 'rgba(0,0,0,.13)', line: 'rgba(0,0,0,.06)',
  textPrimary: '#1D1D1F', textSecondary: '#6E6E73', textTertiary: '#8E8E93', textDisabled: '#A1A1A6', textOnBtn: '#FFFFFF',
  // Status colours. `accent` is the old name of the "running / ok" green and stays green.
  accent: '#34A36D', success: '#34A36D', successText: '#23784F', warning: '#E09A2B', warningText: '#9A5410', danger: '#D93A44', error: '#D93A44',
  bgSecondary: '#F2F2F4',
  scrollbar: 'rgba(0,0,0,.18)', shadowHover: 'rgba(0,0,0,.12)', shadowPop: 'rgba(0,0,0,.18)',
  shadowPanel: '0 0 0 1px rgba(0,0,0,.05),0 4px 16px rgba(0,0,0,.04)',
  shadowCard: '0 0 0 1px rgba(0,0,0,.06),0 1px 2px rgba(0,0,0,.03)',
  shadowMenu: '0 0 0 1px rgba(0,0,0,.08),0 14px 36px rgba(0,0,0,.18)',
  shadowDialog: '0 0 0 1px rgba(0,0,0,.08),0 30px 80px rgba(0,0,0,.28),0 8px 20px rgba(0,0,0,.1)',
  shadowBtn: 'inset 0 1px 0 rgba(255,255,255,.28),0 1px 2px rgba(0,0,0,.18)',
  shadowBtnSecondary: '0 0 0 1px rgba(0,0,0,.12),0 1px 1.5px rgba(0,0,0,.05)',
  overlay: 'rgba(24,24,28,.32)',
  // Layout and shape.
  titlebarHeight: '44px', sidebarWidth: '264px', sidebarCollapsed: '60px', pageInset: '28px', pageTop: '22px',
  sectionGap: '22px', cardGap: '16px', radiusTag: '6px', radiusInput: '8px', radiusButton: '9px', radiusControl: '8px',
  radiusWallpaper: '12px', radiusCard: '12px', radiusCardLg: '18px', radiusBanner: '18px', radiusDialog: '18px', radiusMenu: '12px',
  fontFamily: "-apple-system, 'SF Pro Text', 'PingFang SC', 'Microsoft YaHei UI', 'Noto Sans SC', sans-serif",
  fontMono: "'SF Mono', Menlo, Consolas, monospace",
};
export type UiTokens = { [K in keyof typeof lightTokens]: string };
/** The same roles on near-black: panels step up in lightness (#1C1C1E → #232325 → #2C2C2E), the theme colour brightens. */
export const darkTokens: UiTokens = {
  ...lightTokens,
  appBg: '#1C1C1E', sidebarBg: '#1C1C1E', surface: '#232325', surfaceAlt: '#2C2C2E', surfaceSunken: '#1E1E20', surfaceHover: 'rgba(255,255,255,.06)',
  surfaceRaised: 'rgba(44,44,46,.97)', control: '#2C2C2E', controlHover: '#343437',
  border: 'rgba(255,255,255,.10)', borderStrong: 'rgba(255,255,255,.16)', line: 'rgba(255,255,255,.07)',
  textPrimary: '#F5F5F7', textSecondary: '#A1A1A6', textTertiary: '#7C7C82', textDisabled: '#5E5E63',
  accent: '#3FB87A', success: '#3FB87A', successText: '#6FD6A0', warning: '#F0A93A', warningText: '#F0B65A', danger: '#E5484D', error: '#E5484D',
  bgSecondary: '#2C2C2E',
  scrollbar: 'rgba(255,255,255,.2)', shadowHover: 'rgba(0,0,0,.5)', shadowPop: 'rgba(0,0,0,.6)',
  shadowPanel: '0 0 0 1px rgba(255,255,255,.06),0 4px 16px rgba(0,0,0,.3)',
  shadowCard: '0 0 0 1px rgba(255,255,255,.07)',
  shadowMenu: '0 0 0 1px rgba(255,255,255,.1),0 14px 36px rgba(0,0,0,.5)',
  shadowDialog: '0 0 0 1px rgba(255,255,255,.1),0 30px 80px rgba(0,0,0,.6)',
  shadowBtn: 'inset 0 1px 0 rgba(255,255,255,.18),0 1px 2px rgba(0,0,0,.4)',
  shadowBtnSecondary: '0 0 0 1px rgba(255,255,255,.14)',
  overlay: 'rgba(0,0,0,.5)',
};

/** Light or dark tokens with the theme colour filled in. */
export function tokensFor(dark = false, id: AccentId = currentAccent.value): UiTokens {
  const p: AccentPalette = accentPalette(id, dark);
  const base = dark ? darkTokens : lightTokens;
  return {
    ...base,
    btn: p.btn, btnHi: p.btnHi, soft: p.soft, ink: p.ink, tint: p.tint, dot: p.dot,
    primary: p.btn, primaryHover: p.btnHi, primaryPressed: p.ink, primarySoft: p.soft, primaryText: p.ink,
    appBg: p.tint, sidebarBg: p.tint,
  };
}
/** @deprecated read CSS variables instead; kept for older imports. */
export const uiTokens: UiTokens = lightTokens;
const kebab = (name: string) => name.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`);
export function applyUiTokens(dark = false, id: AccentId = currentAccent.value) {
  const root = document.documentElement;
  for (const [name, value] of Object.entries(tokensFor(dark, id))) root.style.setProperty(`--ui-${kebab(name)}`, value);
  root.dataset.accent = id;
}
