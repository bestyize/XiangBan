// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest';
import { generatedCover } from './cover.ts';

const game = { id: 'classic-snake', name: '经典贪吃蛇', description: '吃到食物，越长越有趣' };
const parse = (url: string) => new DOMParser().parseFromString(decodeURIComponent(url.split(',')[1]), 'image/svg+xml');

describe('generated game covers', () => {
  it('creates deterministic, self-contained SVG artwork with the game title', () => {
    const cover = generatedCover(game);
    expect(cover).toBe(generatedCover(game));
    const svg = parse(cover);
    expect(svg.querySelector('svg')?.getAttribute('viewBox')).toBe('0 0 640 360');
    expect(svg.querySelector('parsererror')).toBeNull();
    expect(svg.querySelectorAll('tspan')[0].textContent).toBe(game.name);
    expect(svg.querySelectorAll('image, script, foreignObject')).toHaveLength(0);
    expect(generatedCover({ ...game, id: 'super-mario', name: '超级玛丽', description: '平台跳跃冒险' })).not.toBe(cover);
  });

  it('escapes names and wraps long titles into at most two lines', () => {
    const hostile = parse(generatedCover({ ...game, name: '<script>&"\'\u0000' }));
    expect(hostile.querySelector('script')).toBeNull();
    expect(hostile.querySelector('parsererror')).toBeNull();
    expect(hostile.querySelector('tspan')?.textContent).toBe('<script>&"\'');
    const long = parse(generatedCover({ ...game, name: '超级无敌好玩的经典休闲益智游戏特别珍藏完整版'.repeat(10) }));
    expect(long.querySelectorAll('tspan')).toHaveLength(2);
    expect(long.querySelectorAll('tspan')[1].textContent).toMatch(/…$/);
  });
});
