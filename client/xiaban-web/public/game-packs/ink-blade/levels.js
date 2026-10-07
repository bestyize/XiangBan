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
    prompts: [{ x: 0, key: 'move' }, { x: 760, key: 'jump' }, { x: 1410, key: 'dash' }],
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
      { x: 800, y: 242, w: 150 },
      { x: 1820, y: 244, w: 180 },
      { x: 2100, y: 206, w: 130 },
      { x: 2620, y: 246, w: 140 },
      { x: 2950, y: 240, w: 150 },
      { x: 3620, y: 236, w: 160 },
      { x: 3950, y: 242, w: 150 },
      { x: 4620, y: 234, w: 150 },
    ],
    iceZones: [[2290, 3290]],
    prompts: [{ x: 0, key: 'move' }, { x: 760, key: 'jump' }, { x: 1410, key: 'dash' }],
    encounters: {
      snow_peak: { id: 'snow_peak', trigger: 900, lock: [640, 1400], waves: [
        [{ type: 'ronin', x: 1110 }, { type: 'ronin', x: 1220, delay: 0.2 }],
        [{ type: 'ronin', side: 1 }, { type: 'ronin', side: -1, delay: 0.3 }, { type: 'archer', x: 875, y: 242, delay: 0.5 }],
      ] },
      ferry: { id: 'ferry', trigger: 1580, lock: [1330, 2090], waves: [
        [{ type: 'spearman', side: 1 }, { type: 'ronin', side: -1, delay: 0.3 }],
        [{ type: 'spearman', side: 1 }, { type: 'spearman', side: -1, delay: 0.4 }, { type: 'ronin', side: 1, delay: 0.9 }],
      ] },
      ice_lake: { id: 'ice_lake', trigger: 2780, lock: [2410, 3170], waves: [
        [{ type: 'ronin', side: 1 }, { type: 'ronin', side: -1, delay: 0.3 }, { type: 'archer', x: 2690, y: 246, delay: 0.5 }],
        [{ type: 'spearman', side: 1 }, { type: 'spearman', side: -1, delay: 0.3 }, { type: 'ronin', side: 1, delay: 0.8 }, { type: 'ronin', side: -1, delay: 1.2 }],
      ] },
      reeds: { id: 'reeds', trigger: 3700, lock: [3360, 4120], waves: [
        [{ type: 'spearman', side: 1 }, { type: 'archer', x: 3700, y: 236, delay: 0.2 }, { type: 'archer', x: 4025, delay: 0.4 }],
        [{ type: 'hammer', side: 1 }, { type: 'ronin', side: -1, delay: 0.3 }, { type: 'ronin', side: 1, delay: 1.1 }],
      ] },
      blizzard: { id: 'blizzard', trigger: 4560, lock: [4300, 5060], waves: [
        [{ type: 'hammer', side: -1 }, { type: 'spearman', side: 1 }, { type: 'crow', side: 1, delay: 0.3 }, { type: 'crow', side: -1, delay: 0.7 }],
        [{ type: 'hammer', side: 1 }, { type: 'spearman', side: -1, delay: 0.4 }, { type: 'ronin', side: 1, delay: 0.8 }, { type: 'ronin', side: -1, delay: 1.2 }],
      ] },
      angler: { id: 'angler', trigger: 5560, lock: [5300, 6060], waves: [[{ type: 'angler', x: 5910 }]] },
    },
  }

  // Challenge stats: maxCombo, executions, airKills, deflectKills, parries, bossExecutions, time (seconds, at most).
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
      par: 55, hurt: 1, challenge: { stat: 'executions', goal: 2, text: '处决 2 名敌人' }, grades: [1800, 2400, 3000] },
    { id: '2-2', code: 202, volume: 2, name: '古渡', sub: '第二幅', range: [1340, 2330], start: 1460, encounters: ['ferry'],
      par: 90, hurt: 1, challenge: { stat: 'parries', goal: 5, text: '弹反长枪横扫 5 次' }, grades: [2200, 3000, 3800] },
    { id: '2-3', code: 203, volume: 2, name: '冰泽', sub: '第三幅', range: [2290, 3290], start: 2340, encounters: ['ice_lake'],
      par: 95, hurt: 2, challenge: { stat: 'maxCombo', goal: 10, text: '在冰上达成 10 连击' }, grades: [2400, 3200, 4200] },
    { id: '2-4', code: 204, volume: 2, name: '芦荡', sub: '第四幅', range: [3250, 4230], start: 3300, encounters: ['reeds'],
      par: 100, hurt: 2, challenge: { stat: 'airKills', goal: 3, text: '在空中击败 3 名敌人' }, grades: [2600, 3500, 4500] },
    { id: '2-5', code: 205, volume: 2, name: '夜雪', sub: '第五幅', range: [4200, 5230], start: 4250, encounters: ['blizzard'],
      par: 110, hurt: 2, challenge: { stat: 'parries', goal: 8, text: '弹反 8 次' }, grades: [3600, 4800, 6200] },
    { id: '2-6', code: 206, volume: 2, name: '独钓', sub: '终幅', range: [5190, 6120], start: 5240, encounters: ['angler'],
      par: 130, hurt: 2, boss: true, challenge: { stat: 'bossExecutions', goal: 2, text: '破势处决钓叟 2 次' }, grades: [4800, 6200, 7800] },
    { id: '2-scroll', code: 200, volume: 2, name: '长卷', sub: '全卷连战', range: [0, 6120], start: 520,
      encounters: ['snow_peak', 'ferry', 'ice_lake', 'reeds', 'blizzard', 'angler'], par: 450, hurt: 4, scroll: true,
      challenge: { stat: 'time', goal: 380, at: 'most', text: '6 分 20 秒内踏雪通卷' }, grades: [15000, 20000, 25000] },
  ]

  const GRADES = ['', '能品', '妙品', '神品', '逸品']
  const SEALS = [{ bit: 1, char: '过', name: '通关' }, { bit: 2, char: '全', name: '少伤' }, { bit: 4, char: '技', name: '绝技' }]
  const byId = new Map(LEVELS.map((level) => [level.id, level]))
  const byCode = new Map(LEVELS.map((level) => [level.code, level]))

  function volume(id) { return [BAMBOO, SNOW_RIVER].find((entry) => entry.id === id) || null }
  function level(id) { return byId.get(id) || null }
  function levelByCode(code) { return byCode.get(code) || null }
  function volumeLevels(id) { return LEVELS.filter((entry) => entry.volume === id) }

  globalThis.XiabanInkLevels = { VOLUMES: [BAMBOO, SNOW_RIVER], LEVELS, GRADES, SEALS, volume, level, levelByCode, volumeLevels }
})()
