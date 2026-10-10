/** Shared desktop UI tokens. Media artwork and the pet renderer keep their own colors. */
export const uiTokens = {
  primary: '#ED7467', primaryHover: '#DF6256', primaryPressed: '#CA5148', primarySoft: '#FCE9E3',
  appBg: '#FFFAF6', sidebarBg: '#FFF5EE', surface: '#FFFFFF', surfaceAlt: '#FFF8F2',
  border: '#F0E4DC', textPrimary: '#352C2A', textSecondary: '#786B65', textTertiary: '#9C908A', accent: '#E5A35C',
  titlebarHeight: '44px', sidebarWidth: '208px', sidebarCollapsed: '64px', pageInset: '28px', pageTop: '24px',
  sectionGap: '24px', cardGap: '16px', radiusTag: '6px', radiusInput: '10px', radiusButton: '10px',
  radiusWallpaper: '12px', radiusCard: '14px', radiusBanner: '18px',
  fontFamily: '"Microsoft YaHei UI", "PingFang SC", "Noto Sans CJK SC", sans-serif',
} as const;
export function applyUiTokens() {
  for (const [name, value] of Object.entries(uiTokens)) {
    document.documentElement.style.setProperty(`--ui-${name.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`)}`, value);
  }
}
