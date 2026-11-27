export type TestId =
  | 'sbti'
  | 'pantone'
  | 'animal'
  | 'manual'
  | 'checkup'
  | 'classic16';

export type TestCategory = 'viral' | 'aesthetic' | 'relation' | 'classic';

export type ScoringMode = 'nearest-profile' | 'axis-code';

export interface DimensionSpec {
  id: string;
  label: string;
  lowLabel?: string;
  highLabel?: string;
  /** 用于 axis-code 模式 */
  highCode?: string;
  lowCode?: string;
  /** 高分是否呈现醒目红条 */
  hotWhenHigh?: boolean;
}

export interface QuestionOption {
  key: 'A' | 'B' | 'C' | 'D';
  text: string;
  sub: string;
  /** 维度增量，通常在 -2 到 +3 之间 */
  weights: Record<string, number>;
}

export interface QuestionSpec {
  id: string;
  tag: string;
  title: string;
  sub: string;
  options: QuestionOption[];
}

export interface MatchInfo {
  name: string;
  desc: string;
}

export interface ProfileExtra {
  /* pantone */
  swatchGradient?: string;
  surfaceColor?: { name: string; hex: string; pct: number };
  soulColor?: { name: string; hex: string; pct: number };
  colorNo?: string;
  /* animal */
  animalKey?: string;
  animalEn?: string;
  animalPills?: string[];
  feedingGuide?: string;
  /* manual */
  modelNo?: string;
  specs?: Array<{ label: string; value: string }>;
  ingredients?: string;
  chargeMethod?: string;
  warning?: string;
  /* checkup */
  mentalAge?: number;
  batteryPct?: number;
  batteryLabel?: string;
  rows?: Array<{ name: string; value: string; status: 'flag' | 'ok' }>;
  prescription?: string;
  /* classic16 */
  jungFunctions?: string;
  /* sbti */
  stampText?: string;
  eggHint?: string;
}

export interface ResultProfile {
  code: string;
  title: string;
  subtitle: string;
  rarity: string;
  quote: string;
  tags: string[];
  buddy: MatchInfo;
  rival: MatchInfo;
  avatarEmoji: string;
  /** 0..100 目标向量，nearest-profile 模式使用 */
  targetVector?: Record<string, number>;
  /** 隐藏彩蛋判定函数条件（纯数据描述，便于序列化） */
  isHiddenEgg?: boolean;
  eggConditions?: Array<{ dim: string; op: 'gte' | 'lte'; value: number }>;
  extra?: ProfileExtra;
}

export interface TestDefinition {
  id: TestId;
  title: string;
  shortTitle: string;
  subtitle: string;
  heroDesc: string;
  category: TestCategory;
  badge: string;
  estimatedMinutes: number;
  cardBg: string;
  posterBg: string;
  tagColor: string;
  accentColor: string;
  scoringMode: ScoringMode;
  dimensions: DimensionSpec[];
  microHints: string[];
  questions: QuestionSpec[];
  profiles: ResultProfile[];
  disclaimer?: string;
}

export interface EvaluatedResult {
  testId: TestId;
  profileCode: string;
  profileTitle: string;
  scores: Record<string, number>;
  isHiddenEgg: boolean;
  completedAt: number;
}

export interface PeerPayload {
  testId: TestId;
  profileCode: string;
  name: string;
  scores: Record<string, number>;
}

export interface CompatAxisItem {
  id: string;
  label: string;
  me: number;
  ta: number;
}

export interface CompatReport {
  testId: TestId;
  score: number;
  badgeLabel: string;
  headline: string;
  summary: string;
  myProfile: ResultProfile;
  peerProfile: ResultProfile;
  peerName: string;
  axes: CompatAxisItem[];
  synergies: string[];
  cautions: string[];
}
