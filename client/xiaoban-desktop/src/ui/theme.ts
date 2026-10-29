import type { GlobalThemeOverrides } from "naive-ui";
import { tokensFor } from './design-tokens.ts';
import { accent as currentAccent, type AccentId } from './appearance.ts';

/** Naive UI follows the theme colour and light/dark mode (方案四). Naive needs concrete colours, not CSS variables. */
export function themeFor(dark: boolean, id: AccentId = currentAccent.value): GlobalThemeOverrides {
  const t = tokensFor(dark, id);
  const field = dark ? '#2C2C2E' : '#FFFFFF';
  const fieldBorder = dark ? '1px solid rgba(255,255,255,.14)' : '1px solid rgba(0,0,0,.13)';
  const focusShadow = `0 0 0 3px ${t.soft}`;
  const menu = dark ? '#2C2C2E' : '#FFFFFF';
  const menuShadow = t.shadowMenu;
  return {
    common: {
      primaryColor: t.btn, primaryColorHover: t.btnHi, primaryColorPressed: t.ink, primaryColorSuppl: t.btnHi,
      infoColor: dark ? '#4C8DF6' : '#2F6FDB', successColor: t.success, warningColor: t.warning, errorColor: t.danger,
      errorColorHover: t.danger, errorColorPressed: t.danger,
      textColorBase: t.textPrimary, textColor1: t.textPrimary, textColor2: t.textSecondary, textColor3: t.textTertiary,
      placeholderColor: t.textTertiary, textColorDisabled: t.textDisabled,
      borderColor: dark ? 'rgba(255,255,255,.12)' : 'rgba(0,0,0,.12)', dividerColor: t.line,
      bodyColor: t.appBg, cardColor: t.surface, modalColor: t.surface, popoverColor: menu, tableColor: t.surface,
      inputColor: field, actionColor: t.surfaceAlt, hoverColor: t.surfaceHover,
      borderRadius: '8px', borderRadiusSmall: '6px', fontSize: '13px', fontSizeMedium: '13px', fontWeightStrong: '600',
      fontFamily: t.fontFamily, fontFamilyMono: t.fontMono,
      boxShadow2: menuShadow, boxShadow3: t.shadowDialog,
    },
    Button: {
      fontWeight: '500', heightTiny: '24px', heightSmall: '28px', heightMedium: '32px', heightLarge: '38px',
      paddingMedium: '0 14px', paddingSmall: '0 11px',
      borderRadiusTiny: '6px', borderRadiusSmall: '7px', borderRadiusMedium: '9px', borderRadiusLarge: '10px',
      color: dark ? '#2C2C2E' : '#FFFFFF', colorHover: dark ? '#343437' : '#FAFAFB', colorPressed: dark ? '#3A3A3D' : '#F2F2F4',
      border: dark ? '1px solid rgba(255,255,255,.14)' : '1px solid rgba(0,0,0,.12)',
      borderHover: dark ? '1px solid rgba(255,255,255,.2)' : '1px solid rgba(0,0,0,.16)',
      borderPressed: dark ? '1px solid rgba(255,255,255,.2)' : '1px solid rgba(0,0,0,.16)',
      borderFocus: `1px solid ${t.btn}`,
      textColor: t.textPrimary, textColorHover: t.textPrimary, textColorPressed: t.textPrimary, textColorFocus: t.textPrimary,
      colorPrimary: t.btn, colorHoverPrimary: t.btnHi, colorPressedPrimary: t.ink, colorFocusPrimary: t.btnHi,
      colorDisabledPrimary: dark ? '#3A3A3D' : '#E5E5EA', borderDisabledPrimary: '1px solid transparent',
      borderPrimary: `1px solid ${t.btn}`, borderHoverPrimary: `1px solid ${t.btnHi}`, borderPressedPrimary: `1px solid ${t.ink}`, borderFocusPrimary: `1px solid ${t.btnHi}`,
      textColorPrimary: '#fff', textColorHoverPrimary: '#fff', textColorPressedPrimary: '#fff', textColorFocusPrimary: '#fff',
      textColorDisabledPrimary: t.textDisabled,
      textColorTextPrimary: t.ink, textColorTextHoverPrimary: t.btn, textColorTextPressedPrimary: t.ink, textColorTextFocusPrimary: t.btn,
      textColorGhostPrimary: t.ink, borderPrimaryGhost: `1px solid ${t.btn}`,
    },
    Input: {
      heightSmall: '28px', heightMedium: '32px', heightLarge: '38px', borderRadius: '8px', fontSizeMedium: '13px',
      color: field, colorFocus: field, border: fieldBorder, borderHover: dark ? '1px solid rgba(255,255,255,.22)' : '1px solid rgba(0,0,0,.2)',
      borderFocus: `1px solid ${t.btn}`, boxShadowFocus: focusShadow, caretColor: t.btn,
    },
    InternalSelection: {
      heightSmall: '28px', heightMedium: '32px', heightLarge: '38px', borderRadius: '8px', fontSizeMedium: '13px',
      color: field, colorActive: field, border: fieldBorder, borderHover: dark ? '1px solid rgba(255,255,255,.22)' : '1px solid rgba(0,0,0,.2)',
      borderActive: `1px solid ${t.btn}`, borderFocus: `1px solid ${t.btn}`, boxShadowActive: focusShadow, boxShadowFocus: focusShadow, caretColor: t.btn,
    },
    InternalSelectMenu: {
      borderRadius: '12px', color: menu, boxShadow: menuShadow, height: 'auto',
      optionColorPending: t.surfaceHover, optionColorActive: t.soft, optionColorActivePending: t.soft,
      optionTextColorActive: t.ink, optionCheckColor: t.ink, optionHeightMedium: '32px', paddingMedium: '5px',
    },
    Select: { peers: { InternalSelection: { heightMedium: '32px', borderRadius: '8px' }, InternalSelectMenu: { borderRadius: '12px', color: menu } }, menuBoxShadow: menuShadow },
    Dropdown: {
      borderRadius: '12px', color: menu, boxShadow: menuShadow, padding: '5px', optionHeightMedium: '32px',
      optionColorHover: t.surfaceHover, optionColorActive: t.soft, optionTextColorActive: t.ink, dividerColor: t.line,
    },
    Popover: { borderRadius: '12px', color: menu, boxShadow: menuShadow, padding: '10px 12px' },
    Tooltip: { borderRadius: '7px', padding: '5px 9px' },
    Switch: {
      railHeightMedium: '22px', railWidthMedium: '38px', buttonHeightMedium: '18px', buttonWidthMedium: '18px',
      railColor: dark ? '#3A3A3D' : '#E3E3E7', railColorActive: t.btn, buttonBoxShadow: '0 1px 3px rgba(0,0,0,.2)',
      boxShadowFocus: focusShadow, loadingColor: t.btn,
    },
    Checkbox: { colorChecked: t.btn, borderChecked: `1px solid ${t.btn}`, borderFocus: `1px solid ${t.btn}`, boxShadowFocus: focusShadow, borderRadius: '5px' },
    Radio: { dotColorActive: t.btn, boxShadowActive: `inset 0 0 0 1px ${t.btn}`, boxShadowFocus: `inset 0 0 0 1px ${t.btn}, ${focusShadow}`, buttonBorderColorActive: t.btn, buttonTextColorActive: t.ink, buttonColorActive: t.surface },
    Slider: { fillColor: t.btn, fillColorHover: t.btnHi, handleSize: '16px', railHeight: '4px', railColor: dark ? '#3A3A3D' : '#E3E3E8', railColorHover: dark ? '#444448' : '#D9D9DE', dotBorderActive: `2px solid ${t.btn}` },
    Tabs: { tabTextColorActiveLine: t.textPrimary, tabTextColorHoverLine: t.textPrimary, barColor: t.btn, tabTextColorActiveSegment: t.textPrimary, colorSegment: t.control, tabBorderRadius: '6px', tabFontWeightActive: '600' },
    Card: { paddingMedium: '20px', borderRadius: '12px', titleFontSizeMedium: '15px', titleFontWeight: '600', color: t.surface, colorModal: t.surface, borderColor: t.line },
    Dialog: { borderRadius: '18px', padding: '22px 24px', titleFontSize: '15px', titleFontWeight: '600', color: t.surface, iconColorWarning: t.warning, iconColorError: t.danger },
    Modal: { boxShadow: t.shadowDialog, color: t.surface },
    Drawer: { color: t.surface },
    Message: { borderRadius: '10px', padding: '9px 14px', boxShadow: menuShadow, color: menu, iconColorSuccess: t.success },
    Notification: { borderRadius: '12px', color: menu, boxShadow: menuShadow },
    Tag: { borderRadius: '6px', textColorInfo: t.ink, colorInfo: t.soft, borderInfo: '1px solid transparent', textColorPrimary: t.ink, colorPrimary: t.soft, borderPrimary: '1px solid transparent', heightSmall: '20px', heightMedium: '24px' },
    Badge: { color: t.dot },
    Progress: { fillColor: t.btn, railColor: dark ? '#3A3A3D' : '#EDEDF0' },
    Spin: { color: t.btn },
    Alert: { borderRadius: '10px', padding: '11px 14px' },
    Skeleton: { color: dark ? '#2C2C2E' : '#F0F0F2', colorEnd: dark ? '#343437' : '#E6E6EA' },
    Empty: { textColor: t.textTertiary, iconColor: t.textDisabled },
  } as GlobalThemeOverrides;
}
export const theme = themeFor(false);
