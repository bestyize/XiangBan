/** Shared desktop UI tokens. Media artwork and the pet renderer keep their own colors. */
export const lightTokens = {
  primary: '#FF2442', primaryHover: '#F51E3D', primaryPressed: '#D81B39', primarySoft: '#FFEBEE', primaryText: '#D81B39',
  appBg: '#F6F6F7', sidebarBg: '#FFFFFF', surface: '#FFFFFF', surfaceAlt: '#F5F5F6',
  border: '#EBEBED', textPrimary: '#1F1F22', textSecondary: '#66666B', textTertiary: '#9A9AA0', accent: '#1FB871',
  scrollbar: '#D6D6DA', shadowHover: 'rgba(31,31,34,.16)', shadowPop: 'rgba(31,31,34,.24)',
  titlebarHeight: '44px', sidebarWidth: '170px', sidebarCollapsed: '64px', pageInset: '32px', pageTop: '28px',
  sectionGap: '24px', cardGap: '20px', radiusTag: '8px', radiusInput: '12px', radiusButton: '12px',
  radiusWallpaper: '16px', radiusCard: '18px', radiusBanner: '20px',
  fontFamily: '"Microsoft YaHei UI", "PingFang SC", "Noto Sans CJK SC", sans-serif',
} as const;
export type UiTokens = { [K in keyof typeof lightTokens]: string };
/** The same roles on near-black. The red stays; surfaces step up in lightness instead of being outlined. */
export const darkTokens: UiTokens = {
  ...lightTokens,
  primaryHover: '#FF4A63', primaryPressed: '#E01E3B', primarySoft: '#3A1A21', primaryText: '#FF7285',
  appBg: '#0F0F10', sidebarBg: '#161618', surface: '#1E1E21', surfaceAlt: '#28282C',
  border: '#303035', textPrimary: '#F2F2F4', textSecondary: '#A9A9B0', textTertiary: '#787880', accent: '#2BD184',
  scrollbar: '#3A3A40', shadowHover: 'rgba(0,0,0,.55)', shadowPop: 'rgba(0,0,0,.6)',
};
export const uiTokens: UiTokens = lightTokens;
export function applyUiTokens(dark = false) {
  const tokens = dark ? darkTokens : lightTokens;
  for (const [name, value] of Object.entries(tokens)) {
    document.documentElement.style.setProperty(`--ui-${name.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)}`, value);
  }
}
