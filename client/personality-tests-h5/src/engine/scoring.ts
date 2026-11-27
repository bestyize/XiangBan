import type {
  CompatAxisItem,
  CompatReport,
  EvaluatedResult,
  PeerPayload,
  ResultProfile,
  TestDefinition
} from './types.ts';

function clamp(n: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, n));
}

/**
 * 将用户的作答转换为每个维度的 0~100 标准化分值。
 * 根据题库中各题各选项能达到的理论最小/最大值做精准归一化，
 * 避免题量变化或权重不同导致结果偏移。
 */
export function computeDimensionScores(
  test: TestDefinition,
  answers: Record<string, string>
): Record<string, number> {
  const raw: Record<string, number> = {};
  const minSum: Record<string, number> = {};
  const maxSum: Record<string, number> = {};

  for (const dim of test.dimensions) {
    raw[dim.id] = 0;
    minSum[dim.id] = 0;
    maxSum[dim.id] = 0;
  }

  for (const q of test.questions) {
    for (const dim of test.dimensions) {
      let lo = Infinity;
      let hi = -Infinity;
      for (const opt of q.options) {
        const w = opt.weights[dim.id] ?? 0;
        if (w < lo) lo = w;
        if (w > hi) hi = w;
      }
      if (lo === Infinity) lo = 0;
      if (hi === -Infinity) hi = 0;
      minSum[dim.id] += lo;
      maxSum[dim.id] += hi;
    }

    const pickedKey = answers[q.id];
    const picked = q.options.find((o) => o.key === pickedKey);
    if (picked) {
      for (const [dimId, weight] of Object.entries(picked.weights)) {
        if (dimId in raw) {
          raw[dimId] += weight;
        }
      }
    }
  }

  const normalized: Record<string, number> = {};
  for (const dim of test.dimensions) {
    const span = maxSum[dim.id] - minSum[dim.id];
    if (span <= 0) {
      normalized[dim.id] = 50;
    } else {
      const pct = Math.round(((raw[dim.id] - minSum[dim.id]) / span) * 100);
      normalized[dim.id] = clamp(pct, 6, 98);
    }
  }
  return normalized;
}

function checkEggMatch(
  profile: ResultProfile,
  scores: Record<string, number>
): boolean {
  if (!profile.isHiddenEgg || !profile.eggConditions?.length) return false;
  return profile.eggConditions.every((cond) => {
    const val = scores[cond.dim] ?? 50;
    return cond.op === 'gte' ? val >= cond.value : val <= cond.value;
  });
}

export function resolveProfile(
  test: TestDefinition,
  scores: Record<string, number>
): ResultProfile {
  // 1. 优先检查隐藏彩蛋条件
  for (const p of test.profiles) {
    if (checkEggMatch(p, scores)) {
      return p;
    }
  }

  // 2. 四轴编码模式（16 型人格）
  if (test.scoringMode === 'axis-code') {
    const code = test.dimensions
      .map((d) => {
        const val = scores[d.id] ?? 50;
        return val >= 50 ? (d.highCode ?? 'X') : (d.lowCode ?? 'Y');
      })
      .join('');
    const exact = test.profiles.find((p) => p.code === code);
    if (exact) return exact;
  }

  // 3. 欧氏距离最近邻匹配常规原型
  const normalProfiles = test.profiles.filter((p) => !p.isHiddenEgg);
  let best = normalProfiles[0] ?? test.profiles[0];
  let bestDist = Infinity;

  for (const p of normalProfiles) {
    if (!p.targetVector) continue;
    let dist = 0;
    for (const dim of test.dimensions) {
      const target = p.targetVector[dim.id] ?? 50;
      const diff = (scores[dim.id] ?? 50) - target;
      dist += diff * diff;
    }
    if (dist < bestDist) {
      bestDist = dist;
      best = p;
    }
  }

  return best;
}

export function evaluateTest(
  test: TestDefinition,
  answers: Record<string, string>
): EvaluatedResult {
  const scores = computeDimensionScores(test, answers);
  const profile = resolveProfile(test, scores);
  return {
    testId: test.id,
    profileCode: profile.code,
    profileTitle: profile.title,
    scores,
    isHiddenEgg: Boolean(profile.isHiddenEgg),
    completedAt: Date.now()
  };
}

export function findProfileByCode(
  test: TestDefinition,
  code: string
): ResultProfile {
  return test.profiles.find((p) => p.code === code) ?? test.profiles[0];
}

/**
 * 基于双方维度向量与原型关系，真实生成双人合盘报告
 */
export function computeCompatibility(
  test: TestDefinition,
  mine: EvaluatedResult,
  peer: PeerPayload
): CompatReport {
  const myProfile = findProfileByCode(test, mine.profileCode);
  const peerProfile = findProfileByCode(test, peer.profileCode);

  const topDims = test.dimensions.slice(0, 4);
  const axes: CompatAxisItem[] = topDims.map((d) => {
    const me = mine.scores[d.id] ?? 50;
    const ta = peer.scores[d.id] ?? peerProfile.targetVector?.[d.id] ?? 50;
    return { id: d.id, label: d.label, me, ta };
  });

  // 计算平均差值：适度相似 + 一点互补最舒服
  const avgDiff =
    axes.reduce((acc, item) => acc + Math.abs(item.me - item.ta), 0) /
    Math.max(1, axes.length);

  let score = Math.round(97 - avgDiff * 0.52);
  if (myProfile.buddy.name.includes(peerProfile.title) || myProfile.buddy.name.includes(peerProfile.code)) {
    score += 7;
  } else if (myProfile.rival.name.includes(peerProfile.title) || myProfile.rival.name.includes(peerProfile.code)) {
    score -= 6;
  }
  score = clamp(score, 66, 98);

  let badgeLabel = '灵魂绝配';
  let headline = '同频松弛搭子 · 默契自然流';
  let summary = `「${myProfile.title}」遇上「${peerProfile.title}」，你们在核心节奏上高度同频，不用费力寒暄也能自在相处。`;

  if (score >= 92) {
    badgeLabel = '天选双生';
    headline = '镜像级灵魂搭子 · 一眼秒懂';
    summary = `「${myProfile.title}」×「${peerProfile.title}」简直像共享同一套底层系统，连吐槽的点和充电方式都惊人一致。`;
  } else if (score >= 83) {
    badgeLabel = '灵魂绝配';
    headline = '外冷内热组合 · 越处越上头';
    summary = `一个接得住情绪，一个懂得分寸感。待在一起哪怕各刷各的手机，也完全不会尴尬。`;
  } else if (score >= 75) {
    badgeLabel = '奇妙互补';
    headline = '反差感拍档 · 互相拓宽边界';
    summary = `你们的思考方式刚好错开半拍，一个负责稳住底盘，一个负责带来新鲜火花。`;
  } else {
    badgeLabel = '欢喜冤家';
    headline = '高张力碰撞组 · 互怼出真情';
    summary = `性格反差感拉满，虽然偶尔会因为节奏不同互相嫌弃，但关键时刻往往最能互补短板。`;
  }

  // 动态挑出最接近的 2 维做默契点，差距最大的 2 维做避雷提醒
  const sortedByClose = [...axes].sort(
    (a, b) => Math.abs(a.me - a.ta) - Math.abs(b.me - b.ta)
  );
  const closest1 = sortedByClose[0];
  const closest2 = sortedByClose[1] ?? sortedByClose[0];
  const farthest = sortedByClose[sortedByClose.length - 1];

  const synergies: string[] = [
    `在「${closest1.label}」上高度同频（${closest1.me}% vs ${closest1.ta}%），基本不用解释前提就能秒懂对方意图。`,
    `在「${closest2.label}」上节奏接近，日常相处摩擦力极小，属于低耗能高回血关系。`,
    `${peer.name}的「${peerProfile.title}」特质能很好地呼应你「${myProfile.title}」的内在期待。`
  ];

  const cautions: string[] = [
    `双方在「${farthest.label}」上存在明显反差（${farthest.me}% vs ${farthest.ta}%），遇到分歧时记得先对齐节奏再下结论。`,
    score >= 85
      ? `因为彼此太懂分寸，闹小别扭时容易同时开启“礼貌冷处理”，建议谁先察觉就先递个台阶。`
      : `别用自己的默认标准猜测对方，直球说清真实需求能立刻省掉 80% 的误会。`
  ];

  return {
    testId: test.id,
    score,
    badgeLabel,
    headline,
    summary,
    myProfile,
    peerProfile,
    peerName: peer.name,
    axes,
    synergies,
    cautions
  };
}
