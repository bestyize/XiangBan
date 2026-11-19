import { createSwimRoute } from './swim-route.js';

// Match the two-lap route in RealWorldBuilder's aquarium (a92b6dd2).
const count = 32;
const eight = Array.from({ length: count }, (_, i) => {
  const angle = 2 * Math.PI * i / count;
  const across = Math.sin(angle);
  return [
    2.90 * across,
    .31 + .155 * Math.cos(angle) + .024 * across,
    -.35 - 1.25 * Math.sin(2 * angle),
  ];
});

// Both laps share the approach and departure at the rear-right turn.
const join = 4;
const figureEight = [...eight.slice(join), ...eight.slice(0, join)];
const perimeter = [
  ...eight.slice(join, 13),
  [1.58, .205, 1.02],
  [1.02, .196, 1.09],
  [.44, .193, 1.11],
  [-.15, .204, 1.09],
  [-.74, .226, 1.04],
  [-1.32, .257, .95],
  [-1.88, .295, .83],
  [-2.36, .338, .62],
  [-2.74, .371, .31],
  [-2.97, .395, -.10],
  [-3.01, .415, -.55],
  [-2.77, .432, -.97],
  [-2.36, .444, -1.25],
  [-1.84, .452, -1.37],
  [-1.28, .456, -1.38],
  [-.69, .461, -1.30],
  [-.10, .465, -1.14],
  [.52, .466, -1.02],
  ...eight.slice(join - 2, join),
];

export const cruiseRoute = createSwimRoute([...figureEight, ...perimeter]);
export const vermilionCruiseDuration = 64;
