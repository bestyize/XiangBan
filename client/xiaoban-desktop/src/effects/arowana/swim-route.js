import { MathUtils, Vector3 } from 'three';

const SAMPLES_PER_WAYPOINT = 80;
const { euclideanModulo: wrap, lerp } = MathUtils;

export function lowerIndex(table, key, value) {
  let first = 0;
  let last = table.length - 1;
  while (last - first > 1) {
    const mid = (first + last) >>> 1;
    if (table[mid][key] <= value) first = mid;
    else last = mid;
  }
  return first;
}

export function createSwimRoute(waypoints, startWaypoint = 0, { samplesPerWaypoint = SAMPLES_PER_WAYPOINT } = {}) {
  const points = waypoints.map(([x, y, z]) => new Vector3(x, y, z));

  function evaluateCurve(parameter, position, tangent) {
    const segment = Math.floor(parameter);
    const u = parameter - segment;
    const v = 1 - u;
    const n = points.length;
    const a = points[wrap(segment - 1, n)];
    const b = points[wrap(segment, n)];
    const c = points[wrap(segment + 1, n)];
    const d = points[wrap(segment + 2, n)];
    const u2 = u * u;
    const u3 = u2 * u;
    position.copy(a).multiplyScalar(v ** 3 / 6)
      .addScaledVector(b, (3 * u3 - 6 * u2 + 4) / 6)
      .addScaledVector(c, (-3 * u3 + 3 * u2 + 3 * u + 1) / 6)
      .addScaledVector(d, u3 / 6);
    tangent.copy(a).multiplyScalar(-v * v / 2)
      .addScaledVector(b, (3 * u2 - 4 * u) / 2)
      .addScaledVector(c, (-3 * u2 + 2 * u + 1) / 2)
      .addScaledVector(d, u2 / 2);
    const dx2 = v * a.x + (3 * u - 2) * b.x + (1 - 3 * u) * c.x + u * d.x;
    const dz2 = v * a.z + (3 * u - 2) * b.z + (1 - 3 * u) * c.z + u * d.z;
    const turn = (tangent.z * dx2 - tangent.x * dz2) /
      (tangent.x * tangent.x + tangent.z * tangent.z) ** 1.5;
    tangent.normalize();
    return turn;
  }

  const count = points.length * samplesPerWaypoint;
  const point = new Vector3();
  const tangent = new Vector3();
  const previous = new Vector3();
  const samples = [];
  let distance = 0;

  for (let i = 0; i <= count; i++) {
    const parameter = i * points.length / count;
    const curvature = evaluateCurve(parameter, point, tangent);
    if (i) distance += point.distanceTo(previous);
    samples.push({ parameter, distance, curvature });
    previous.copy(point);
  }
  const length = distance;

  function sample(distance, position, tangent) {
    const s = wrap(distance, length);
    const i = lowerIndex(samples, 'distance', s);
    const a = samples[i];
    const b = samples[i + 1];
    const u = (s - a.distance) / (b.distance - a.distance);
    return evaluateCurve(lerp(a.parameter, b.parameter, u), position, tangent);
  }

  return { length, samples, sample, startIndex: Math.round(startWaypoint * samplesPerWaypoint) };
}
