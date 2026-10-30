import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import { expect, it } from 'vitest';

it('ships the Three.js classes needed to build and interact with the cube', () => {
  const context: { THREE?: Record<string, unknown> } = {};
  runInNewContext(readFileSync('public/game-packs/pro-cube/vendor/three.min.js', 'utf8'), context);
  for (const name of ['CubeTexture', 'Shape', 'ExtrudeGeometry', 'ShadowMaterial', 'MeshPhysicalMaterial', 'Raycaster', 'Matrix3']) {
    expect(context.THREE?.[name], name).toBeTypeOf('function');
  }
});
