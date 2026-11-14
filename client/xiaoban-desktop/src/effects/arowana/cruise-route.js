import { createSwimRoute } from './swim-route.js';

// A shallow front pass and a higher, deeper return. The wider ends give the
// arowana room to bank and turn without folding its body back on itself.
const waypoints = [
  [-2.17, .21, .78], [-1.38, .18, .98], [-.40, .15, 1.06],
  [ .65, .17, .97], [1.62, .22, .88], [2.43, .26, .70],
  [2.90, .26, .23], [3.02, .24, -.50], [2.72, .23, -1.20],
  [2.10, .25, -1.53], [1.09, .29, -1.62], [.12, .37, -1.68],
  [-.84, .44, -1.52], [-1.77, .48, -1.44], [-2.44, .45, -1.17],
  [-2.93, .37, -.64], [-3.02, .28, .11], [-2.75, .25, .61],
];

export const cruiseRoute = createSwimRoute(waypoints, 3.61);
