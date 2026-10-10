import type { GlobalThemeOverrides } from "naive-ui";
import { uiTokens as t } from './design-tokens';
export const theme: GlobalThemeOverrides = {
  common: {
    primaryColor: t.primary, primaryColorHover: t.primaryHover, primaryColorPressed: t.primaryPressed, primaryColorSuppl: t.primary,
    infoColor: t.primaryPressed, successColor: '#4F856A', warningColor: '#A66D27', errorColor: '#B94C42',
    textColorBase: t.textPrimary, textColor1: t.textPrimary, textColor2: t.textSecondary, textColor3: t.textTertiary,
    borderColor: t.border, dividerColor: t.border, bodyColor: t.appBg, cardColor: t.surface, popoverColor: t.surface,
    borderRadius: t.radiusInput, borderRadiusSmall: t.radiusTag, fontSize: '14px', fontFamily: t.fontFamily,
  },
  Button: { fontWeight: '600', heightMedium: '38px', heightLarge: '42px', borderRadiusMedium: t.radiusButton, borderRadiusSmall: t.radiusButton, borderRadiusLarge: t.radiusButton,
    textColorPrimary: t.textPrimary, textColorHoverPrimary: t.textPrimary, textColorPressedPrimary: t.textPrimary },
  Input: { heightMedium: '38px', borderRadius: t.radiusInput },
  Select: { peers: { InternalSelection: { heightMedium: '38px', borderRadius: t.radiusInput } } },
  Card: { paddingMedium: '24px', borderRadius: t.radiusCard, titleFontSizeMedium: '18px' },
  Tag: { borderRadius: t.radiusTag, textColorInfo: t.primaryPressed, colorInfo: t.primarySoft, borderInfo: `1px solid ${t.border}` },
};
