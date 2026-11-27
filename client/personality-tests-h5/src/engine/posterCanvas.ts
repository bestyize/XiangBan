import type { CompatReport, EvaluatedResult, ResultProfile, TestDefinition } from './types.ts';

export type PosterRatio = '916' | '34';

function wrapLines(
  ctx: CanvasRenderingContext2D,
  text: string,
  maxWidth: number
): string[] {
  const lines: string[] = [];
  let line = '';
  for (const ch of text) {
    const test = line + ch;
    if (ctx.measureText(test).width > maxWidth && line) {
      lines.push(line);
      line = ch;
    } else {
      line = test;
    }
  }
  if (line) lines.push(line);
  return lines;
}

function drawRoundedRect(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  r: number
): void {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.arcTo(x + w, y, x + w, y + h, r);
  ctx.arcTo(x + w, y + h, x, y + h, r);
  ctx.arcTo(x, y + h, x, y, r);
  ctx.arcTo(x, y, x + w, y, r);
  ctx.closePath();
}

function drawDecorativeQr(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  size: number,
  seedStr: string
): void {
  drawRoundedRect(ctx, x, y, size, size, 16);
  ctx.fillStyle = '#FFFFFF';
  ctx.fill();
  ctx.lineWidth = 5;
  ctx.strokeStyle = '#18181B';
  ctx.stroke();

  const pad = 16;
  const grid = 9;
  const cell = (size - pad * 2) / grid;
  let hash = 7;
  for (let i = 0; i < seedStr.length; i++) {
    hash = (hash * 31 + seedStr.charCodeAt(i)) % 99991;
  }

  ctx.fillStyle = '#18181B';
  for (let r = 0; r < grid; r++) {
    for (let c = 0; c < grid; c++) {
      const corner =
        (r < 3 && c < 3) || (r < 3 && c >= grid - 3) || (r >= grid - 3 && c < 3);
      const cornerHole =
        (r === 1 && c === 1) ||
        (r === 1 && c === grid - 2) ||
        (r === grid - 2 && c === 1);
      const fill = corner
        ? !cornerHole
        : ((hash + r * 17 + c * 29 + r * c * 7) % 11) > 4;
      if (fill) {
        ctx.fillRect(x + pad + c * cell, y + pad + r * cell, cell - 2, cell - 2);
      }
    }
  }
}

/**
 * 渲染高分辨率潮玩撞色结果海报（1080×1920 或 1080×1440）
 * 返回 PNG DataURL，可直接赋给 <img src>（兼容微信长按保存）或触发下载
 */
export function renderResultPosterDataUrl(
  test: TestDefinition,
  result: EvaluatedResult,
  profile: ResultProfile,
  ratio: PosterRatio
): string {
  const W = 1080;
  const H = ratio === '34' ? 1440 : 1920;
  const canvas = document.createElement('canvas');
  canvas.width = W;
  canvas.height = H;
  const ctx = canvas.getContext('2d');
  if (!ctx) return '';

  // 背景
  if (test.id === 'pantone' && profile.extra?.surfaceColor && profile.extra?.soulColor) {
    const grad = ctx.createLinearGradient(0, 0, W, H);
    grad.addColorStop(0, profile.extra.surfaceColor.hex);
    grad.addColorStop(0.55, '#FBF6EE');
    grad.addColorStop(1, profile.extra.soulColor.hex);
    ctx.fillStyle = grad;
  } else {
    ctx.fillStyle = test.posterBg;
  }
  ctx.fillRect(0, 0, W, H);

  // 轻微网点纹理
  ctx.fillStyle = 'rgba(24,24,27,0.06)';
  for (let gx = 28; gx < W; gx += 34) {
    for (let gy = 28; gy < H; gy += 34) {
      ctx.beginPath();
      ctx.arc(gx, gy, 2.2, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  // 主卡片外框
  const M = 54;
  drawRoundedRect(ctx, M + 12, M + 12, W - M * 2, H - M * 2, 42);
  ctx.fillStyle = '#18181B';
  ctx.fill();

  drawRoundedRect(ctx, M, M, W - M * 2, H - M * 2, 42);
  ctx.fillStyle = '#FFFDF8';
  ctx.fill();
  ctx.lineWidth = 8;
  ctx.strokeStyle = '#18181B';
  ctx.stroke();

  // 顶部色带区
  const topH = ratio === '34' ? 490 : 580;
  ctx.save();
  drawRoundedRect(ctx, M, M, W - M * 2, topH, 42);
  ctx.clip();
  if (test.id === 'pantone' && profile.extra?.surfaceColor && profile.extra?.soulColor) {
    const g2 = ctx.createLinearGradient(M, M, W - M, M + topH);
    g2.addColorStop(0, profile.extra.surfaceColor.hex);
    g2.addColorStop(1, profile.extra.soulColor.hex);
    ctx.fillStyle = g2;
  } else {
    ctx.fillStyle = test.posterBg;
  }
  ctx.fillRect(M, M, W - M * 2, topH);
  ctx.restore();

  // 顶部分割线
  ctx.beginPath();
  ctx.moveTo(M, M + topH);
  ctx.lineTo(W - M, M + topH);
  ctx.lineWidth = 6;
  ctx.strokeStyle = '#18181B';
  ctx.stroke();

  const headerInk = test.id === 'pantone' ? '#FFFFFF' : '#18181B';
  ctx.fillStyle = headerInk;
  ctx.font = '900 32px -apple-system,"PingFang SC",sans-serif';
  ctx.fillText(`人格贩卖机 · ${test.title}`, M + 52, M + 84);

  // 稀有度红章贴纸
  const stampText = profile.isHiddenEgg ? `★ 隐藏彩蛋 · ${profile.rarity}` : profile.rarity;
  ctx.font = '900 28px -apple-system,"PingFang SC",sans-serif';
  const stampW = ctx.measureText(stampText).width + 52;
  const sx = W - M - 48 - stampW;
  const sy = M + 44;
  drawRoundedRect(ctx, sx + 5, sy + 5, stampW, 56, 28);
  ctx.fillStyle = '#18181B';
  ctx.fill();
  drawRoundedRect(ctx, sx, sy, stampW, 56, 28);
  ctx.fillStyle = '#FF4D5E';
  ctx.fill();
  ctx.lineWidth = 4.5;
  ctx.strokeStyle = '#18181B';
  ctx.stroke();
  ctx.fillStyle = '#FFFFFF';
  ctx.fillText(stampText, sx + 26, sy + 38);

  // 主视觉代码 + 标题
  ctx.fillStyle = headerInk;
  ctx.font = '900 124px ui-monospace,SFMono-Regular,Menlo,sans-serif';
  ctx.fillText(profile.code, M + 52, M + 258);

  ctx.font = '900 66px -apple-system,"PingFang SC",sans-serif';
  ctx.fillText(`${profile.avatarEmoji} ${profile.title}`, M + 52, M + 354);

  ctx.font = '700 32px -apple-system,"PingFang SC",sans-serif';
  ctx.fillStyle = test.id === 'pantone' ? 'rgba(255,255,255,0.92)' : '#3C3934';
  ctx.fillText(profile.subtitle, M + 52, M + 418);

  // 标签胶囊行
  let tx = M + 52;
  const ty = M + topH - 76;
  ctx.font = '800 26px -apple-system,"PingFang SC",sans-serif';
  for (const tag of profile.tags.slice(0, 3)) {
    const tw = ctx.measureText(tag).width + 38;
    drawRoundedRect(ctx, tx, ty, tw, 46, 23);
    ctx.fillStyle = '#FFFDF8';
    ctx.fill();
    ctx.lineWidth = 3.5;
    ctx.strokeStyle = '#18181B';
    ctx.stroke();
    ctx.fillStyle = '#18181B';
    ctx.fillText(tag, tx + 19, ty + 32);
    tx += tw + 16;
  }

  // 金句黑框
  let curY = M + topH + 44;
  ctx.font = '600 34px -apple-system,"PingFang SC",sans-serif';
  const qLines = wrapLines(ctx, `“${profile.quote}”`, W - M * 2 - 168).slice(0, 3);
  const qBoxH = 62 + qLines.length * 52;
  drawRoundedRect(ctx, M + 48, curY, W - M * 2 - 96, qBoxH, 24);
  ctx.fillStyle = '#18181B';
  ctx.fill();
  ctx.fillStyle = '#FFFDF8';
  qLines.forEach((ln, i) => {
    ctx.fillText(ln, M + 84, curY + 64 + i * 52);
  });
  curY += qBoxH + 42;

  // 4 维能量条（2×2 网格）
  const dims = test.dimensions.slice(0, 4);
  const colW = (W - M * 2 - 96 - 36) / 2;
  dims.forEach((d, idx) => {
    const col = idx % 2;
    const row = Math.floor(idx / 2);
    const dx = M + 48 + col * (colW + 36);
    const dy = curY + row * 98;
    const val = result.scores[d.id] ?? 50;

    ctx.fillStyle = '#18181B';
    ctx.font = '800 28px -apple-system,"PingFang SC",sans-serif';
    ctx.fillText(d.label, dx, dy + 28);
    ctx.font = '900 28px ui-monospace,Menlo,sans-serif';
    ctx.fillText(`${val}%`, dx + colW - 76, dy + 28);

    drawRoundedRect(ctx, dx, dy + 44, colW, 24, 12);
    ctx.fillStyle = '#ECE6D8';
    ctx.fill();
    ctx.lineWidth = 3;
    ctx.strokeStyle = '#18181B';
    ctx.stroke();

    const fillW = Math.max(24, (colW * val) / 100);
    drawRoundedRect(ctx, dx, dy + 44, fillW, 24, 12);
    ctx.fillStyle = d.hotWhenHigh && val >= 70 ? '#FF4D5E' : '#18181B';
    ctx.fill();
  });
  curY += Math.ceil(dims.length / 2) * 98 + 26;

  // 9:16 专属：天选搭子 & 相处宿敌双卡
  if (ratio === '916') {
    const pairH = 220;
    const drawPairCard = (
      x: number,
      bg: string,
      badge: string,
      name: string,
      desc: string
    ) => {
      drawRoundedRect(ctx, x + 6, curY + 6, colW, pairH, 26);
      ctx.fillStyle = '#18181B';
      ctx.fill();
      drawRoundedRect(ctx, x, curY, colW, pairH, 26);
      ctx.fillStyle = bg;
      ctx.fill();
      ctx.lineWidth = 4.5;
      ctx.strokeStyle = '#18181B';
      ctx.stroke();

      ctx.fillStyle = '#18181B';
      ctx.font = '800 24px -apple-system,"PingFang SC",sans-serif';
      ctx.fillText(badge, x + 26, curY + 46);
      ctx.font = '900 32px -apple-system,"PingFang SC",sans-serif';
      ctx.fillText(name.slice(0, 11), x + 26, curY + 98);
      ctx.font = '600 24px -apple-system,"PingFang SC",sans-serif';
      const dLines = wrapLines(ctx, desc, colW - 52).slice(0, 2);
      dLines.forEach((l, i) => ctx.fillText(l, x + 26, curY + 144 + i * 36));
    };

    drawPairCard(
      M + 48,
      '#6BE5A8',
      '★ 天选搭子',
      profile.buddy.name,
      profile.buddy.desc
    );
    drawPairCard(
      M + 48 + colW + 36,
      '#FFB5C8',
      '⚡ 相处宿敌',
      profile.rival.name,
      profile.rival.desc
    );
  }

  // 底部回流区
  const footY = H - M - 188;
  ctx.beginPath();
  ctx.setLineDash([12, 10]);
  ctx.moveTo(M + 48, footY);
  ctx.lineTo(W - M - 48, footY);
  ctx.lineWidth = 4;
  ctx.strokeStyle = 'rgba(24,24,27,0.35)';
  ctx.stroke();
  ctx.setLineDash([]);

  ctx.fillStyle = '#18181B';
  ctx.font = '900 36px -apple-system,"PingFang SC",sans-serif';
  ctx.fillText('扫码测你的同款盲盒人格', M + 48, footY + 74);
  ctx.fillStyle = '#6E6960';
  ctx.font = '700 26px -apple-system,"PingFang SC",sans-serif';
  ctx.fillText('people.xiaban.xyz · 0元全免费 · 支持双人合盘', M + 48, footY + 124);

  drawDecorativeQr(ctx, W - M - 48 - 134, footY + 24, 134, `${test.id}-${profile.code}`);

  return canvas.toDataURL('image/png');
}

/**
 * 渲染双人合盘合照 PNG
 */
export function renderCompatPosterDataUrl(
  test: TestDefinition,
  report: CompatReport,
  myName: string
): string {
  const W = 1080;
  const H = 1440;
  const canvas = document.createElement('canvas');
  canvas.width = W;
  canvas.height = H;
  const ctx = canvas.getContext('2d');
  if (!ctx) return '';

  ctx.fillStyle = '#FBF6EE';
  ctx.fillRect(0, 0, W, H);

  const M = 54;
  drawRoundedRect(ctx, M + 10, M + 10, W - M * 2, H - M * 2, 42);
  ctx.fillStyle = '#18181B';
  ctx.fill();
  drawRoundedRect(ctx, M, M, W - M * 2, H - M * 2, 42);
  ctx.fillStyle = '#FFFDF8';
  ctx.fill();
  ctx.lineWidth = 8;
  ctx.strokeStyle = '#18181B';
  ctx.stroke();

  ctx.fillStyle = '#18181B';
  ctx.font = '900 36px -apple-system,"PingFang SC",sans-serif';
  ctx.fillText(`人格贩卖机 · ${test.title} 双人合盘`, M + 52, M + 90);

  // 左右双人卡
  const cardW = 360;
  const cardY = M + 150;
  const drawPerson = (x: number, bg: string, name: string, p: ResultProfile) => {
    drawRoundedRect(ctx, x + 8, cardY + 8, cardW, 250, 32);
    ctx.fillStyle = '#18181B';
    ctx.fill();
    drawRoundedRect(ctx, x, cardY, cardW, 250, 32);
    ctx.fillStyle = bg;
    ctx.fill();
    ctx.lineWidth = 5;
    ctx.strokeStyle = '#18181B';
    ctx.stroke();

    ctx.fillStyle = '#18181B';
    ctx.font = '700 66px sans-serif';
    ctx.fillText(p.avatarEmoji, x + 144, cardY + 102);
    ctx.font = '900 36px -apple-system,"PingFang SC",sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText(name, x + cardW / 2, cardY + 166);
    ctx.font = '700 28px -apple-system,"PingFang SC",sans-serif';
    ctx.fillText(p.title, x + cardW / 2, cardY + 214);
    ctx.textAlign = 'left';
  };

  drawPerson(M + 48, '#FFD89B', myName, report.myProfile);
  drawPerson(W - M - 48 - cardW, '#8DD3FF', report.peerName, report.peerProfile);

  // 中间合拍度圆章
  ctx.beginPath();
  ctx.arc(W / 2, cardY + 125, 96, 0, Math.PI * 2);
  ctx.fillStyle = '#FF4D5E';
  ctx.fill();
  ctx.lineWidth = 7;
  ctx.strokeStyle = '#18181B';
  ctx.stroke();
  ctx.fillStyle = '#FFFFFF';
  ctx.textAlign = 'center';
  ctx.font = '900 68px ui-monospace,Menlo,sans-serif';
  ctx.fillText(`${report.score}%`, W / 2, cardY + 136);
  ctx.font = '800 24px -apple-system,"PingFang SC",sans-serif';
  ctx.fillText(report.badgeLabel, W / 2, cardY + 178);

  // 标题与综述
  ctx.fillStyle = '#18181B';
  ctx.font = '900 44px -apple-system,"PingFang SC",sans-serif';
  ctx.fillText(report.headline, W / 2, cardY + 340);
  ctx.textAlign = 'left';

  ctx.fillStyle = '#5E5A53';
  ctx.font = '600 30px -apple-system,"PingFang SC",sans-serif';
  const sumLines = wrapLines(ctx, report.summary, W - M * 2 - 120).slice(0, 2);
  sumLines.forEach((l, i) => ctx.fillText(l, M + 60, cardY + 404 + i * 46));

  // 默契与避雷卡片
  const boxY = cardY + 510;
  drawRoundedRect(ctx, M + 48, boxY, W - M * 2 - 96, 210, 26);
  ctx.fillStyle = '#6BE5A8';
  ctx.fill();
  ctx.lineWidth = 5;
  ctx.strokeStyle = '#18181B';
  ctx.stroke();
  ctx.fillStyle = '#18181B';
  ctx.font = '900 30px -apple-system,"PingFang SC",sans-serif';
  ctx.fillText('★ 核心默契点', M + 80, boxY + 56);
  ctx.font = '600 26px -apple-system,"PingFang SC",sans-serif';
  wrapLines(ctx, report.synergies[0], W - M * 2 - 160)
    .slice(0, 2)
    .forEach((l, i) => ctx.fillText(l, M + 80, boxY + 110 + i * 42));

  const box2Y = boxY + 238;
  drawRoundedRect(ctx, M + 48, box2Y, W - M * 2 - 96, 180, 26);
  ctx.fillStyle = '#FFB5C8';
  ctx.fill();
  ctx.lineWidth = 5;
  ctx.strokeStyle = '#18181B';
  ctx.stroke();
  ctx.fillStyle = '#18181B';
  ctx.font = '900 30px -apple-system,"PingFang SC",sans-serif';
  ctx.fillText('⚡ 相处避雷提醒', M + 80, box2Y + 56);
  ctx.font = '600 26px -apple-system,"PingFang SC",sans-serif';
  wrapLines(ctx, report.cautions[0], W - M * 2 - 160)
    .slice(0, 2)
    .forEach((l, i) => ctx.fillText(l, M + 80, box2Y + 110 + i * 42));

  const footY = H - M - 164;
  ctx.fillStyle = '#18181B';
  ctx.font = '900 34px -apple-system,"PingFang SC",sans-serif';
  ctx.fillText('发给好友测同款 · 看看你们合拍几分', M + 52, footY + 64);
  ctx.fillStyle = '#6E6960';
  ctx.font = '700 26px -apple-system,"PingFang SC",sans-serif';
  ctx.fillText('people.xiaban.xyz · 人格贩卖机 · 0元全免费', M + 52, footY + 112);
  drawDecorativeQr(ctx, W - M - 52 - 120, footY + 10, 120, `${report.myProfile.code}-${report.peerProfile.code}`);

  return canvas.toDataURL('image/png');
}
