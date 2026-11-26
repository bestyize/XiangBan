export type AccentId = 'red' | 'green' | 'blue' | 'purple' | 'amber' | 'graphite';

export interface AccentTokens {
  btn: string;
  hi: string;
  soft: string;
  ink: string;
}

export interface AccentPreset {
  id: AccentId;
  name: string;
  light: AccentTokens;
  dark: AccentTokens;
}

export const ACCENT_PRESETS: readonly AccentPreset[] = [
  {
    id: 'red',
    name: '小伴红',
    light: { btn: '#E3203C', hi: '#F0506A', soft: '#FFF3F4', ink: '#E3203C' },
    dark: { btn: '#EE4B5F', hi: '#F76B7D', soft: '#3A2225', ink: '#FF8C91' },
  },
  {
    id: 'green',
    name: '薄荷绿',
    light: { btn: '#2E8B5E', hi: '#3FA172', soft: '#E3F2E9', ink: '#1E6A45' },
    dark: { btn: '#34A36D', hi: '#48B981', soft: '#1D3328', ink: '#72D6A2' },
  },
  {
    id: 'blue',
    name: '海湾蓝',
    light: { btn: '#2F6FDB', hi: '#4A84E6', soft: '#E6EEFC', ink: '#1D53AE' },
    dark: { btn: '#3B7BE8', hi: '#5A92F0', soft: '#1E2A42', ink: '#8DB6FF' },
  },
  {
    id: 'purple',
    name: '暮紫',
    light: { btn: '#7152D8', hi: '#8669E3', soft: '#EDE8FC', ink: '#5537B8' },
    dark: { btn: '#8466E6', hi: '#9A80EE', soft: '#2A2342', ink: '#BBA8FF' },
  },
  {
    id: 'amber',
    name: '琥珀',
    light: { btn: '#A8640C', hi: '#BD7A1C', soft: '#FAEEDB', ink: '#87500A' },
    dark: { btn: '#C27A1A', hi: '#D69130', soft: '#3A2C17', ink: '#F0B65A' },
  },
  {
    id: 'graphite',
    name: '石墨',
    light: { btn: '#2B2B30', hi: '#4A4A50', soft: '#E9E9EC', ink: '#2B2B30' },
    dark: { btn: '#636368', hi: '#7C7C82', soft: '#3A3A3D', ink: '#E5E5EA' },
  },
] as const;

export const ACCENT_IDS: readonly AccentId[] = ACCENT_PRESETS.map((a) => a.id);

export function getAccentPreset(id: AccentId): AccentPreset {
  return ACCENT_PRESETS.find((a) => a.id === id) ?? ACCENT_PRESETS[0];
}
