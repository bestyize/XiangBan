import { createSwimRoute } from './swim-route.js';

function createArowanaCruiseRoute({ halfWidth, halfDepth, centerY, centerZ = 0 }) {
  const count = 32;
  // The two lobes turn in opposite directions. Their crossing passes stay
  // at different heights, so the fish can swim cleanly through the middle.
  const waypoints = Array.from({ length: count }, (_, i) => {
    const angle = 2 * Math.PI * i / count;
    const across = Math.sin(angle);
    return [
      halfWidth * across,
      centerY + .155 * Math.cos(angle) + .024 * across,
      centerZ - halfDepth * Math.sin(2 * angle),
    ];
  });
  return createSwimRoute(waypoints, .09 * count);
}

export const cruiseRoute = createArowanaCruiseRoute({
  halfWidth: 2.90, halfDepth: 1.25, centerY: .31, centerZ: -.35,
});
