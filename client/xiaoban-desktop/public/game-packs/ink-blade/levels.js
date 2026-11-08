/* 水墨剑客 · level data. A 卷 is one painted world; each 幅 is a playable window of it, and 长卷 runs the whole world. */
(() => {
  'use strict'

  const BAMBOO = {
    id: 1, name: '竹林卷', length: 6120,
    areas: [
      { x: 0, name: '山门', sub: '第一幅' },
      { x: 1440, name: '竹林', sub: '第二幅' },
      { x: 2330, name: '鸦岭', sub: '第三幅' },
      { x: 3290, name: '断桥', sub: '第四幅' },
      { x: 4230, name: '夜雨', sub: '第五幅' },
      { x: 5230, name: '无常', sub: '终幅' },
    ],
    platforms: [
      { x: 2210, y: 246, w: 124 },
      { x: 2400, y: 202, w: 112 },
      { x: 2590, y: 242, w: 132 },
      { x: 3560, y: 224, w: 210 },
      { x: 3880, y: 238, w: 142 },
      { x: 4560, y: 238, w: 152 },
    ],
    prompts: [{ x: 0, key: 'move' }, { x: 760, key: 'jump' }, { x: 1410, key: 'rigging', local: true }],
    encounters: {
      gate: { id: 'gate', trigger: 900, lock: [640, 1400], waves: [
        [{ type: 'dummy', x: 1110 }, { type: 'dummy', x: 1220, delay: 0.2 }],
        [{ type: 'dummy', x: 1000 }, { type: 'dummy', x: 1160, delay: 0.2 }, { type: 'dummy', x: 1300, delay: 0.4 }],
      ] },
      grove: { id: 'grove', trigger: 1580, lock: [1330, 2090], waves: [
        [{ type: 'puppet', side: 1 }, { type: 'puppet', side: -1, delay: 0.3 }],
        [{ type: 'puppet', side: 1 }, { type: 'puppet', side: -1 }, { type: 'puppet', side: 1, delay: 1.1 }],
        [{ type: 'puppet', side: 1 }, { type: 'puppet', side: -1 }, { type: 'puppet', side: 1, delay: 0.8 }, { type: 'puppet', side: -1, delay: 1.6 }],
      ] },
      crows: { id: 'crows', trigger: 2780, lock: [2410, 3170], waves: [
        [{ type: 'crow', side: 1 }, { type: 'crow', side: -1, delay: 0.4 }, { type: 'crow', side: 1, delay: 0.8 }],
        [{ type: 'puppet', side: -1 }, { type: 'crow', side: 1 }, { type: 'puppet', side: 1, delay: 0.6 }, { type: 'crow', side: -1, delay: 1.2 }],
        [{ type: 'crow', side: 1 }, { type: 'crow', side: -1 }, { type: 'puppet', side: -1, delay: 0.3 }, { type: 'crow', side: 1, delay: 0.6 }, { type: 'puppet', side: 1, delay: 1.2 }],
      ] },
      bridge: { id: 'bridge', trigger: 3700, lock: [3360, 4120], waves: [
        [{ type: 'archer', x: 3672, y: 224 }, { type: 'archer', x: 4060, delay: 0.3 }, { type: 'puppet', side: -1, delay: 0.5 }],
        [{ type: 'shield', side: 1 }, { type: 'puppet', side: -1 }, { type: 'puppet', side: 1, delay: 1.2 }],
        [{ type: 'shield', side: -1 }, { type: 'archer', x: 3900, y: 238, delay: 0.3 }, { type: 'puppet', side: 1 }, { type: 'puppet', side: -1, delay: 1 }],
      ] },
      rain: { id: 'rain', trigger: 4560, lock: [4300, 5060], waves: [
        [{ type: 'shield', side: -1 }, { type: 'crow', side: 1 }, { type: 'archer', x: 4640, y: 238, delay: 0.3 }, { type: 'crow', side: -1, delay: 0.9 }],
        [{ type: 'puppet', side: 1 }, { type: 'puppet', side: -1 }, { type: 'archer', x: 5020, delay: 0.4 }, { type: 'puppet', side: 1, delay: 1.2 }],
        [{ type: 'shield', side: 1 }, { type: 'shield', side: -1, delay: 0.6 }, { type: 'crow', side: 1 }, { type: 'archer', x: 4640, y: 238, delay: 0.4 }, { type: 'crow', side: -1, delay: 0.8 }],
      ] },
      boss: { id: 'boss', trigger: 5560, lock: [5300, 6060], waves: [[{ type: 'boss', x: 5910 }]] },
    },
  }

  // 雪江卷: every 幅 owns one piece of terrain. 寒峰 icicles, 古渡 thin ice, 冰泽 slick ice in three runs,
  // 芦荡 reed cover, 夜雪 gusts, 独钓 a frozen arena.
  const SNOW_RIVER = {
    id: 2, name: '雪江卷', length: 6120,
    areas: [
      { x: 0, name: '寒峰', sub: '第一幅' },
      { x: 1440, name: '古渡', sub: '第二幅' },
      { x: 2330, name: '冰泽', sub: '第三幅' },
      { x: 3290, name: '芦荡', sub: '第四幅' },
      { x: 4230, name: '夜雪', sub: '第五幅' },
      { x: 5230, name: '独钓', sub: '终幅' },
    ],
    platforms: [
      { x: 610, y: 250, w: 120 },
      { x: 1000, y: 240, w: 210 },
      { x: 1780, y: 246, w: 170 },
      { x: 2140, y: 212, w: 120 },
      { x: 2600, y: 246, w: 140 },
      { x: 2850, y: 210, w: 110 },
      { x: 3040, y: 244, w: 130 },
      { x: 3640, y: 238, w: 150 },
      { x: 3980, y: 244, w: 130 },
      { x: 4590, y: 234, w: 150 },
      { x: 4850, y: 226, w: 120 },
    ],
    // y is where the icicle joins the rock; it falls when a slash, a deflected shot or the ink stroke touches it.
    icicles: [
      { x: 1030, y: 246, len: 26 }, { x: 1078, y: 246, len: 32 }, { x: 1126, y: 246, len: 26 }, { x: 1172, y: 246, len: 30 },
    ],
    // Thin ice cracks open under anyone who stands still on it.
    thinIce: [[1520, 2040], [2120, 2270]],
    // Slick ice, in three runs with snow between them.
    iceZones: [[2330, 2610], [2720, 3000], [3080, 3270], [5290, 6070]],
    // Reed beds hide a player who crouches still; shooters lose the target.
    reeds: [[3470, 3590], [3820, 3940]],
    // Gusts: calm, warning, push (seconds); force is px/s at full power.
    wind: { zone: [4230, 5230], cycle: [3.2, 1.0, 2.4], force: 150 },
    prompts: [
      { x: 0, key: 'move' }, { x: 560, key: 'jump' }, { x: 880, key: 'icicle', local: true }, { x: 1410, key: 'rigging', local: true }, { x: 1530, key: 'thin', local: true },
      { x: 2335, key: 'ice', local: true }, { x: 3400, key: 'reed', local: true },
    ],
    encounters: {
      snow_peak: { id: 'snow_peak', trigger: 940, lock: [720, 1460], waves: [
        [{ type: 'ronin', x: 1190 }, { type: 'ronin', x: 1290, delay: 0.2 }],
        [{ type: 'ronin', side: 1 }, { type: 'ronin', side: -1, delay: 0.3 }, { type: 'archer', x: 1085, y: 240, delay: 0.5 }],
      ] },
      ferry: { id: 'ferry', trigger: 1600, lock: [1380, 2100], waves: [
        [{ type: 'spearman', side: 1 }, { type: 'ronin', side: -1, delay: 0.3 }],
        [{ type: 'spearman', side: 1 }, { type: 'spearman', side: -1, delay: 0.4 }, { type: 'ronin', side: 1, delay: 0.9 }],
      ] },
      ice_lake: { id: 'ice_lake', trigger: 2720, lock: [2400, 3140], waves: [
        [{ type: 'ronin', side: 1 }, { type: 'ronin', side: -1, delay: 0.3 }, { type: 'archer', x: 2670, y: 246, delay: 0.5 }],
        [{ type: 'spearman', side: 1 }, { type: 'spearman', side: -1, delay: 0.3 }, { type: 'ronin', side: 1, delay: 0.8 }, { type: 'ronin', side: -1, delay: 1.2 }],
      ] },
      reeds: { id: 'reeds', trigger: 3680, lock: [3360, 4120], waves: [
        [{ type: 'ronin', side: 1 }, { type: 'archer', x: 3715, y: 238, delay: 0.2 }, { type: 'archer', x: 4045, delay: 0.4 }],
        [{ type: 'hammer', side: 1 }, { type: 'ronin', side: -1, delay: 0.3 }, { type: 'ronin', side: 1, delay: 1.1 }],
      ] },
      blizzard: { id: 'blizzard', trigger: 4580, lock: [4300, 5060], waves: [
        [{ type: 'hammer', side: -1 }, { type: 'spearman', side: 1 }, { type: 'crow', side: 1, delay: 0.3 }, { type: 'crow', side: -1, delay: 0.7 }],
        [{ type: 'hammer', side: 1 }, { type: 'spearman', side: -1, delay: 0.4 }, { type: 'ronin', side: 1, delay: 0.8 }, { type: 'ronin', side: -1, delay: 1.2 }],
      ] },
      angler: { id: 'angler', trigger: 5560, lock: [5300, 6060], waves: [[{ type: 'angler', x: 5910 }]] },
    },
  }

  // 烽火卷: the first 浅绛 scroll. 烽燧 arrow volleys behind ruined walls, 胡杨 dry grass that burns downwind,
  // 戈壁 cavalry, 关城 shield walls, 夜袭 a night sandstorm, 将台 the mounted general.
  const BEACON = {
    id: 3, name: '烽火卷', length: 6120,
    // 军中不乱刀: only a fresh swing parries; a swing chained onto a running combo never does.
    strictParry: true,
    areas: [
      { x: 0, name: '烽燧', sub: '第一幅' },
      { x: 1440, name: '胡杨', sub: '第二幅' },
      { x: 2330, name: '戈壁', sub: '第三幅' },
      { x: 3290, name: '关城', sub: '第四幅' },
      { x: 4230, name: '夜袭', sub: '第五幅' },
      { x: 5230, name: '将台', sub: '终幅' },
    ],
    platforms: [
      { x: 1040, y: 238, w: 150 },
      { x: 1690, y: 250, w: 170 },
      { x: 2060, y: 232, w: 110 },
      { x: 2700, y: 236, w: 104 },
      { x: 3530, y: 212, w: 190 },
      { x: 3930, y: 218, w: 170 },
      { x: 4560, y: 246, w: 120 },
      { x: 4870, y: 240, w: 130 },
    ],
    // Volleys come from the east (dir -1, falling westward); a ruined wall of height h shelters the strip just west of it.
    volley: { zone: [0, 1440], cycle: [2.8, 1.2], width: 280, dir: -1 },
    ramparts: [{ x: 900, h: 50 }, { x: 1330, h: 56 }],
    // Dry grass catches fire from a fire pot and burns downwind (fireWind) faster than upwind.
    grass: [[1470, 1700], [1880, 2250]],
    fireWind: 1,
    // 夜袭: a night sandstorm. Light only reaches `radius` around the player and around the camp fires; archers see half as far.
    dark: { zone: [4230, 5230], radius: 165, lights: [4440, 4810, 5020] },
    prompts: [
      { x: 0, key: 'move' }, { x: 300, key: 'strict' }, { x: 560, key: 'jump' }, { x: 1410, key: 'rigging', local: true }, { x: 1460, key: 'grass', local: true }, { x: 4250, key: 'dark', local: true },
    ],
    encounters: {
      beacon: { id: 'beacon', trigger: 940, lock: [700, 1440], waves: [
        [{ type: 'soldier', side: 1 }, { type: 'soldier', side: 1, delay: 0.6 }],
        [{ type: 'soldier', side: 1 }, { type: 'soldier', side: -1, delay: 0.3 }, { type: 'archer', x: 1110, y: 238, delay: 0.5 }, { type: 'spearman', side: 1, delay: 1.2 }],
      ] },
      poplar: { id: 'poplar', trigger: 1600, lock: [1380, 2100], waves: [
        [{ type: 'bomber', side: 1 }, { type: 'soldier', side: -1, delay: 0.3 }, { type: 'soldier', side: 1, delay: 0.8 }],
        [{ type: 'bomber', side: 1 }, { type: 'bomber', x: 1760, y: 250, delay: 0.3 }, { type: 'soldier', side: -1 }, { type: 'soldier', side: 1, delay: 1 }, { type: 'soldier', side: -1, delay: 1.6 }],
      ] },
      gobi: { id: 'gobi', trigger: 2720, lock: [2400, 3140], waves: [
        [{ type: 'cavalry', side: 1 }, { type: 'soldier', side: -1, delay: 0.6 }],
        [{ type: 'cavalry', side: -1 }, { type: 'bomber', side: 1, delay: 0.4 }, { type: 'soldier', side: 1, delay: 1 }],
        [{ type: 'cavalry', side: 1 }, { type: 'cavalry', side: -1, delay: 2.4 }, { type: 'archer', x: 2752, y: 236, delay: 0.5 }],
      ] },
      pass: { id: 'pass', trigger: 3680, lock: [3360, 4120], waves: [
        [{ type: 'pavise', x: 3950, group: 'a' }, { type: 'pavise', x: 3992, group: 'a', delay: 0.15 }, { type: 'pavise', x: 4034, group: 'a', delay: 0.3 }, { type: 'archer', x: 4010, y: 218, delay: 0.5 }],
        [{ type: 'pavise', x: 3440, group: 'b' }, { type: 'pavise', x: 3482, group: 'b', delay: 0.15 }, { type: 'soldier', side: 1, delay: 0.4 }, { type: 'soldier', side: 1, delay: 1.2 }, { type: 'archer', x: 3610, y: 212, delay: 0.6 }],
      ] },
      raid: { id: 'raid', trigger: 4580, lock: [4300, 5060], waves: [
        [{ type: 'soldier', side: 1 }, { type: 'soldier', side: -1, delay: 0.3 }, { type: 'bomber', x: 4620, y: 246, delay: 0.5 }, { type: 'crow', side: 1, delay: 1 }],
        [{ type: 'pavise', x: 4930, group: 'c' }, { type: 'pavise', x: 4972, group: 'c', delay: 0.15 }, { type: 'cavalry', side: -1, delay: 0.8 }, { type: 'soldier', side: -1, delay: 1.8 }],
        [{ type: 'cavalry', side: 1 }, { type: 'soldier', side: -1, delay: 0.3 }, { type: 'bomber', x: 4930, y: 240, delay: 0.5 }, { type: 'cavalry', side: -1, delay: 2.6 }],
      ] },
      dais: { id: 'dais', trigger: 5560, lock: [5300, 6060], waves: [[{ type: 'general', x: 5900 }]] },
    },
  }

  // 怒涛卷: the parried hook brings a pirate into execution range; marked tide surges wash across three arenas.
  const STORM_SEA = {
    id: 4, name: '怒涛卷', length: 6120,
    areas: [
      { x: 0, name: '海门', sub: '第一幅' }, { x: 1440, name: '桅林', sub: '第二幅' },
      { x: 2330, name: '潮洞', sub: '第三幅' }, { x: 3290, name: '礁城', sub: '第四幅' },
      { x: 4230, name: '风眼', sub: '第五幅' }, { x: 5230, name: '墨蛟', sub: '终幅' },
    ],
    platforms: [
      { x: 980, y: 242, w: 150 }, { x: 1710, y: 230, w: 150 }, { x: 2020, y: 246, w: 125 },
      { x: 2640, y: 236, w: 170 }, { x: 2980, y: 218, w: 125 },
      { x: 3540, y: 220, w: 170 }, { x: 3890, y: 240, w: 155 },
      { x: 4590, y: 230, w: 155 }, { x: 4900, y: 246, w: 120 },
    ],
    tide: { zones: [[2400, 3140], [4300, 5060]], cycle: [4.2, 1.15, 1.25] },
    rigging: [{ x: 1570, target: 1700 }, { x: 1910, target: 2020 }, { x: 3490, target: 3610 }, { x: 4430, target: 4570 }, { x: 4810, target: 4930 }],
    prompts: [
      { x: 0, key: 'move' }, { x: 520, key: 'jump' }, { x: 820, key: 'hookPull', local: true },
      { x: 1410, key: 'rigging', local: true }, { x: 2430, key: 'tide', local: true },
    ],
    encounters: {
      sea_gate: { id: 'sea_gate', trigger: 920, lock: [700, 1440], waves: [
        [{ type: 'pirate', side: 1 }, { type: 'pirate', side: -1, delay: 0.4 }],
        [{ type: 'hooker', side: 1 }, { type: 'pirate', side: -1, delay: 0.4 }, { type: 'pirate', side: 1, delay: 1.1 }],
      ] },
      masts: { id: 'masts', trigger: 1600, lock: [1380, 2100], waves: [
        [{ type: 'hooker', side: 1 }, { type: 'pirate', side: -1, delay: 0.3 }, { type: 'archer', x: 1780, y: 230, delay: 0.6 }],
        [{ type: 'hooker', side: -1 }, { type: 'hooker', side: 1, delay: 0.7 }, { type: 'pirate', side: 1 }, { type: 'archer', x: 2070, y: 246, delay: 0.5 }],
      ] },
      cave: { id: 'cave', trigger: 2720, lock: [2400, 3140], waves: [
        [{ type: 'pirate', side: 1 }, { type: 'hooker', side: -1, delay: 0.5 }],
        [{ type: 'hooker', side: 1 }, { type: 'pirate', side: -1 }, { type: 'pirate', side: 1, delay: 1.0 }],
        [{ type: 'hooker', side: -1 }, { type: 'hooker', side: 1, delay: 0.9 }, { type: 'archer', x: 3020, y: 218, delay: 0.5 }],
      ] },
      reef: { id: 'reef', trigger: 3680, lock: [3360, 4120], waves: [
        [{ type: 'hooker', side: 1 }, { type: 'hooker', side: -1, delay: 0.4 }, { type: 'archer', x: 3610, y: 220, delay: 0.7 }],
        [{ type: 'pirate', side: 1 }, { type: 'hooker', side: -1 }, { type: 'hooker', side: 1, delay: 0.9 }, { type: 'archer', x: 3960, y: 240, delay: 0.5 }],
      ] },
      storm: { id: 'storm', trigger: 4580, lock: [4300, 5060], waves: [
        [{ type: 'pirate', side: 1 }, { type: 'hooker', side: -1, delay: 0.3 }, { type: 'archer', x: 4660, y: 230, delay: 0.7 }],
        [{ type: 'hooker', side: 1 }, { type: 'hooker', side: -1, delay: 0.5 }, { type: 'pirate', side: 1, delay: 1.2 }],
        [{ type: 'hooker', side: -1 }, { type: 'pirate', side: 1 }, { type: 'hooker', side: 1, delay: 1.1 }, { type: 'archer', x: 4940, y: 246, delay: 0.5 }],
      ] },
      dragon: { id: 'dragon', trigger: 5560, lock: [5300, 6060], waves: [[{ type: 'dragon', x: 5880 }]] },
    },
  }

  // Each 卷's boss and closing lines, read by the result screen and the boss bar.
  BAMBOO.copy = {
    boss: '无常', bossSub: '斗笠客', win: '一剑破无常', winLine: '斗笠客化作一池淡墨，竹林重归寂静。', loseLine: '无常仍立在月下。朱光一闪，便是出剑之时。',
    phase: '无常怒 · 墨气翻涌', place: '竹林',
  }
  SNOW_RIVER.copy = {
    boss: '苏寒江', bossSub: '寒江钓叟', win: '一剑断寒江', winLine: '钓叟收竿踏雪而去，江面重归沉寂。', loseLine: '钓叟仍立在雪岸。朱光一闪，便是出剑之时。',
    phase: '断竿化剑 · 风雪大作', place: '江岸',
  }
  BEACON.copy = {
    boss: '萧破虏', bossSub: '铁骑将军', win: '一剑落将旗', winLine: '将旗倒在烟里，烽燧的火一盏盏熄了。', loseLine: '将军仍勒马台前。枪尖朱光一闪，便是出剑之时。',
    phase: '弃马提刀 · 令旗招箭', place: '边关',
  }
  STORM_SEA.copy = {
    boss: '墨蛟', bossSub: '沉舟之主', win: '一剑平怒涛', winLine: '墨蛟沉入海眼，潮声终于慢了下来。', loseLine: '墨蛟仍盘在孤礁。看准朱光，再迎浪出剑。',
    phase: '逆鳞尽张 · 潜渊破浪', place: '沧海',
  }

  // Challenge stats: maxCombo, executions, airKills, deflectKills, parries, bossExecutions, icicleKills, crackKills, slideKills, ambushes,
  // volleyKills, fireKills, unhorses, formationBreaks, time (seconds, at most).
  // grades: scores for 妙品 / 神品 / 逸品; any clear is at least 能品.
  const LEVELS = [
    { id: '1-1', code: 101, volume: 1, name: '山门', sub: '第一幅', range: [0, 1440], start: 520, encounters: ['gate'],
      par: 45, hurt: 0, challenge: { stat: 'maxCombo', goal: 6, text: '连击达到 6' }, grades: [1700, 1850, 2000] },
    { id: '1-2', code: 102, volume: 1, name: '竹林', sub: '第二幅', range: [1340, 2330], start: 1460, encounters: ['grove'],
      par: 90, hurt: 1, challenge: { stat: 'executions', goal: 3, text: '处决 3 名敌人' }, grades: [2200, 2900, 3500] },
    { id: '1-3', code: 103, volume: 1, name: '鸦岭', sub: '第三幅', range: [2290, 3290], start: 2340, encounters: ['crows'],
      par: 90, hurt: 1, challenge: { stat: 'airKills', goal: 3, text: '在空中击败 3 名敌人' }, grades: [2300, 2900, 3400] },
    { id: '1-4', code: 104, volume: 1, name: '断桥', sub: '第四幅', range: [3250, 4230], start: 3300, encounters: ['bridge'],
      par: 100, hurt: 2, challenge: { stat: 'deflectKills', goal: 2, text: '以斩回的箭击败 2 名敌人' }, grades: [2600, 3500, 4300] },
    { id: '1-5', code: 105, volume: 1, name: '夜雨', sub: '第五幅', range: [4200, 5230], start: 4250, encounters: ['rain'],
      par: 110, hurt: 2, challenge: { stat: 'parries', goal: 8, text: '弹反 8 次' }, grades: [3800, 4800, 6000] },
    { id: '1-6', code: 106, volume: 1, name: '无常', sub: '终幅', range: [5190, 6120], start: 5240, encounters: ['boss'],
      par: 120, hurt: 2, boss: true, challenge: { stat: 'bossExecutions', goal: 2, text: '破势处决无常 2 次' }, grades: [4800, 6000, 7400] },
    { id: '1-scroll', code: 100, volume: 1, name: '长卷', sub: '全卷连战', range: [0, 6120], start: 520,
      encounters: ['gate', 'grove', 'crows', 'bridge', 'rain', 'boss'], par: 420, hurt: 3, scroll: true,
      challenge: { stat: 'time', goal: 360, at: 'most', text: '6 分钟内一气呵成' }, grades: [14000, 18000, 22000] },
    // 卷二 · 雪江卷
    { id: '2-1', code: 201, volume: 2, name: '寒峰', sub: '第一幅', range: [0, 1440], start: 520, encounters: ['snow_peak'],
      par: 55, hurt: 1, challenge: { stat: 'icicleKills', goal: 2, text: '斩落冰凌砸倒 2 名敌人' }, grades: [1800, 2400, 3000] },
    { id: '2-2', code: 202, volume: 2, name: '古渡', sub: '第二幅', range: [1340, 2330], start: 1460, encounters: ['ferry'],
      par: 95, hurt: 2, challenge: { stat: 'crackKills', goal: 2, text: '以裂冰击败 2 名敌人' }, grades: [2200, 3000, 3800] },
    { id: '2-3', code: 203, volume: 2, name: '冰泽', sub: '第三幅', range: [2290, 3290], start: 2340, encounters: ['ice_lake'],
      par: 95, hurt: 2, challenge: { stat: 'slideKills', goal: 4, text: '在冰面滑行中击败 4 名敌人' }, grades: [2400, 3200, 4200] },
    { id: '2-4', code: 204, volume: 2, name: '芦荡', sub: '第四幅', range: [3250, 4230], start: 3300, encounters: ['reeds'],
      par: 100, hurt: 2, challenge: { stat: 'ambushes', goal: 3, text: '芦丛伏击 3 次' }, grades: [2600, 3500, 4500] },
    { id: '2-5', code: 205, volume: 2, name: '夜雪', sub: '第五幅', range: [4200, 5230], start: 4250, encounters: ['blizzard'],
      par: 110, hurt: 2, challenge: { stat: 'parries', goal: 8, text: '弹反 8 次' }, grades: [3600, 4800, 6200] },
    { id: '2-6', code: 206, volume: 2, name: '独钓', sub: '终幅', range: [5190, 6120], start: 5240, encounters: ['angler'],
      par: 130, hurt: 2, boss: true, challenge: { stat: 'bossExecutions', goal: 2, text: '破势处决钓叟 2 次' }, grades: [4800, 6200, 7800] },
    { id: '2-scroll', code: 200, volume: 2, name: '长卷', sub: '全卷连战', range: [0, 6120], start: 520,
      encounters: ['snow_peak', 'ferry', 'ice_lake', 'reeds', 'blizzard', 'angler'], par: 450, hurt: 4, scroll: true,
      challenge: { stat: 'time', goal: 380, at: 'most', text: '6 分 20 秒内踏雪通卷' }, grades: [15000, 20000, 25000] },
    // 卷三 · 烽火卷
    { id: '3-1', code: 301, volume: 3, name: '烽燧', sub: '第一幅', range: [0, 1440], start: 520, encounters: ['beacon'],
      par: 60, hurt: 1, challenge: { stat: 'volleyKills', goal: 2, text: '引箭雨射倒 2 名敌人' }, grades: [2100, 2700, 3400] },
    { id: '3-2', code: 302, volume: 3, name: '胡杨', sub: '第二幅', range: [1340, 2330], start: 1460, encounters: ['poplar'],
      par: 100, hurt: 2, challenge: { stat: 'fireKills', goal: 3, text: '以野火击倒 3 名敌人' }, grades: [2800, 3400, 4200] },
    { id: '3-3', code: 303, volume: 3, name: '戈壁', sub: '第三幅', range: [2290, 3290], start: 2340, encounters: ['gobi'],
      par: 115, hurt: 2, challenge: { stat: 'unhorses', goal: 3, text: '弹反冲锋，打落 3 名骑兵' }, grades: [3400, 4800, 6200] },
    { id: '3-4', code: 304, volume: 3, name: '关城', sub: '第四幅', range: [3250, 4230], start: 3300, encounters: ['pass'],
      par: 105, hurt: 2, challenge: { stat: 'formationBreaks', goal: 2, text: '弹反领阵者，破阵 2 次' }, grades: [3200, 4400, 5600] },
    { id: '3-5', code: 305, volume: 3, name: '夜袭', sub: '第五幅', range: [4200, 5230], start: 4250, encounters: ['raid'],
      par: 110, hurt: 2, challenge: { stat: 'parries', goal: 8, text: '弹反 8 次' }, grades: [4200, 5600, 7000] },
    { id: '3-6', code: 306, volume: 3, name: '将台', sub: '终幅', range: [5190, 6120], start: 5240, encounters: ['dais'],
      par: 150, hurt: 2, boss: true, challenge: { stat: 'bossExecutions', goal: 2, text: '破势处决将军 2 次' }, grades: [6000, 8000, 9800] },
    { id: '3-scroll', code: 300, volume: 3, name: '长卷', sub: '全卷连战', range: [0, 6120], start: 520,
      encounters: ['beacon', 'poplar', 'gobi', 'pass', 'raid', 'dais'], par: 480, hurt: 4, scroll: true,
      challenge: { stat: 'time', goal: 420, at: 'most', text: '7 分钟内踏破烽烟' }, grades: [18000, 24000, 30000] },
    // 卷四 · 怒涛卷
    { id: '4-1', code: 401, volume: 4, name: '海门', sub: '第一幅', range: [0, 1440], start: 520, encounters: ['sea_gate'],
      par: 70, hurt: 1, challenge: { stat: 'hookPulls', goal: 1, text: '弹反钩镰并拽近海寇 1 次' }, grades: [2200, 2900, 3600] },
    { id: '4-2', code: 402, volume: 4, name: '桅林', sub: '第二幅', range: [1340, 2330], start: 1460, encounters: ['masts'],
      par: 100, hurt: 2, challenge: { stat: 'riggingKills', goal: 2, text: '斩断帆索，落桅击败 2 人' }, grades: [3000, 4000, 5100] },
    { id: '4-3', code: 403, volume: 4, name: '潮洞', sub: '第三幅', range: [2290, 3290], start: 2340, encounters: ['cave'],
      par: 110, hurt: 2, challenge: { stat: 'tideKills', goal: 2, text: '借潮涌卷走 2 名敌人' }, grades: [3400, 4600, 5800] },
    { id: '4-4', code: 404, volume: 4, name: '礁城', sub: '第四幅', range: [3250, 4230], start: 3300, encounters: ['reef'],
      par: 110, hurt: 2, challenge: { stat: 'hookPulls', goal: 4, text: '拽近钩镰海寇 4 次' }, grades: [3600, 4800, 6200] },
    { id: '4-5', code: 405, volume: 4, name: '风眼', sub: '第五幅', range: [4200, 5230], start: 4250, encounters: ['storm'],
      par: 125, hurt: 2, challenge: { stat: 'tideKills', goal: 3, text: '借潮涌卷走 3 名敌人' }, grades: [4300, 5800, 7400] },
    { id: '4-6', code: 406, volume: 4, name: '墨蛟', sub: '终幅', range: [5190, 6120], start: 5240, encounters: ['dragon'],
      par: 160, hurt: 2, boss: true, challenge: { stat: 'bossExecutions', goal: 2, text: '破势处决墨蛟 2 次' }, grades: [6500, 8500, 10500] },
    { id: '4-scroll', code: 400, volume: 4, name: '长卷', sub: '全卷连战', range: [0, 6120], start: 520,
      encounters: ['sea_gate', 'masts', 'cave', 'reef', 'storm', 'dragon'], par: 510, hurt: 4, scroll: true,
      challenge: { stat: 'time', goal: 450, at: 'most', text: '7 分 30 秒内踏浪通卷' }, grades: [20000, 27000, 34000] },
  ]

  const GRADES = ['', '能品', '妙品', '神品', '逸品']
  const SEALS = [{ bit: 1, char: '过', name: '通关' }, { bit: 2, char: '全', name: '少伤' }, { bit: 4, char: '技', name: '绝技' }]
  const byId = new Map(LEVELS.map((level) => [level.id, level]))
  const byCode = new Map(LEVELS.map((level) => [level.code, level]))

  const VOLUMES = [BAMBOO, SNOW_RIVER, BEACON, STORM_SEA]
  function volume(id) { return VOLUMES.find((entry) => entry.id === id) || null }
  function level(id) { return byId.get(id) || null }
  function levelByCode(code) { return byCode.get(code) || null }
  function volumeLevels(id) { return LEVELS.filter((entry) => entry.volume === id) }

  globalThis.XiabanInkLevels = { VOLUMES, LEVELS, GRADES, SEALS, volume, level, levelByCode, volumeLevels }
})()
