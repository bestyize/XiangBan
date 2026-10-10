import type { GlobalThemeOverrides } from "naive-ui";
import { darkTokens, lightTokens } from './design-tokens';
export function themeFor(dark: boolean): GlobalThemeOverrides {
  const t = dark ? darkTokens : lightTokens;
  return {
  common: {
    primaryColor: t.primary, primaryColorHover: t.primaryHover, primaryColorPressed: t.primaryPressed, primaryColorSuppl: t.primary,
    infoColor: '#2E90FA', successColor: '#1FB871', warningColor: '#F59E0B', errorColor: t.primaryPressed,
    textColorBase: t.textPrimary, textColor1: t.textPrimary, textColor2: t.textSecondary, textColor3: t.textTertiary,
    borderColor: t.border, dividerColor: t.border, bodyColor: t.appBg, cardColor: t.surface, popoverColor: t.surface,
    borderRadius: t.radiusInput, borderRadiusSmall: t.radiusTag, fontSize: '14px', fontFamily: t.fontFamily,
  },
  Button: { fontWeight: '600', heightMedium: '38px', heightLarge: '42px', borderRadiusMedium: t.radiusButton, borderRadiusSmall: t.radiusButton, borderRadiusLarge: t.radiusButton,
    textColorPrimary: '#fff', textColorHoverPrimary: '#fff', textColorPressedPrimary: '#fff', textColorFocusPrimary: '#fff', textColorDisabledPrimary: '#fff' },
  Input: { heightMedium: '38px', borderRadius: t.radiusInput },
  Select: { peers: { InternalSelection: { heightMedium: '38px', borderRadius: t.radiusInput } } },
  Card: { paddingMedium: '24px', borderRadius: t.radiusCard, titleFontSizeMedium: '18px' },
  Tag: { borderRadius: t.radiusTag, textColorInfo: t.primaryText, colorInfo: t.primarySoft, borderInfo: `1px solid ${t.border}` },
  Slider: { fillColor: t.primary, fillColorHover: t.primaryHover, handleSize: '16px', railHeight: '5px' },
  Switch: { railColorActive: t.primary },
};
}
export const theme = themeFor(false);
