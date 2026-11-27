import type { EvaluatedResult, TestDefinition, TestId } from '../engine/types.ts';
import { sbtiMaskTest } from './sbtiMask.ts';
import { dualColorTest } from './dualColor.ts';
import { animalPersonaTest } from './animalPersona.ts';
import { loveManualTest } from './loveManual.ts';
import { mentalCheckupTest } from './mentalCheckup.ts';
import { classic16Test } from './classic16.ts';
import { ingredientsTest } from './ingredients.ts';
import { xianxiaTest } from './xianxia.ts';
import { rpgCardTest } from './rpgCard.ts';
import { bigFiveTest } from './bigFive.ts';
import { attachmentTest } from './attachment.ts';
import { darkTriadTest } from './darkTriad.ts';

export const ALL_TESTS: TestDefinition[] = [
  sbtiMaskTest,
  dualColorTest,
  animalPersonaTest,
  loveManualTest,
  mentalCheckupTest,
  classic16Test,
  ingredientsTest,
  xianxiaTest,
  rpgCardTest,
  bigFiveTest,
  attachmentTest,
  darkTriadTest
];

export const TEST_MAP: Record<TestId, TestDefinition> = {
  sbti: sbtiMaskTest,
  pantone: dualColorTest,
  animal: animalPersonaTest,
  manual: loveManualTest,
  checkup: mentalCheckupTest,
  classic16: classic16Test,
  ingredients: ingredientsTest,
  xianxia: xianxiaTest,
  rpg: rpgCardTest,
  bigfive: bigFiveTest,
  attachment: attachmentTest,
  darktriad: darkTriadTest
};

export function getTestById(id: TestId): TestDefinition {
  return TEST_MAP[id] ?? sbtiMaskTest;
}

/**
 * 基于已完成的多个测评，合成跨测评共性灵魂洞察
 */
export function buildSoulInsight(
  results: Partial<Record<TestId, EvaluatedResult>>
): string {
  const entries = Object.values(results).filter((r): r is EvaluatedResult => Boolean(r));
  if (entries.length === 0) {
    return '投币测完任意 1 个盲盒即可点亮你的第一枚灵魂贴纸，集齐 3 枚自动解锁跨测评共性洞察。';
  }
  if (entries.length < 3) {
    return `已点亮 ${entries.length} 枚贴纸！再测 ${3 - entries.length} 个不同维度的盲盒，系统将横向比对你的外在面具与底层底色，生成专属跨测评洞察。`;
  }

  const traits: string[] = [];
  if (results.sbti) {
    if ((results.sbti.scores.mask ?? 50) >= 65) traits.push('礼貌外壳厚实');
    else if ((results.sbti.scores.chill ?? 50) >= 65) traits.push('天然反内卷松弛');
    else traits.push('直率不爱演戏');
  }
  if (results.animal) {
    if ((results.animal.scores.zen ?? 50) >= 65) traits.push('情绪稳如水豚');
    else if ((results.animal.scores.sharp ?? 50) >= 65) traits.push('骨子里自带破局锐度');
    else traits.push('极度护短又慢热');
  }
  if (results.bigfive) {
    if ((results.bigfive.scores.open ?? 50) >= 65) traits.push('高开放好奇心');
    else if ((results.bigfive.scores.consc ?? 50) >= 65) traits.push('高尽责稳健秩序');
  }
  if (results.darktriad) {
    if ((results.darktriad.scores.mach ?? 50) >= 65 || (results.darktriad.scores.cold ?? 50) >= 65) {
      traits.push('边界锋利难被拿捏');
    } else {
      traits.push('保留稀缺纯白真诚');
    }
  }
  if (results.attachment) {
    if (results.attachment.profileCode === 'SC') traits.push('亲密关系内核稳态');
    else if (results.attachment.profileCode === 'AV') traits.push('外冷内热自我保护');
  }
  if (results.classic16) {
    const code = results.classic16.profileCode;
    traits.push(code.startsWith('I') ? '偏好深度独处充电' : '擅长在互动中点燃能量');
  }
  if (results.pantone) {
    traits.push((results.pantone.scores.depth ?? 50) >= 60 ? '拥有丰富的深夜精神世界' : '自带明亮治愈氛围感');
  }

  const picked = traits.slice(0, 3).join('、') || '外松内紧、高边界、清醒独立';
  return `跨 ${entries.length} 套盲盒交叉验证，你反复呈现出「${picked}」的稳定底色——既懂得守住自己的能量边界，也在认定的人与事上保留着极高纯度的真诚。`;
}
