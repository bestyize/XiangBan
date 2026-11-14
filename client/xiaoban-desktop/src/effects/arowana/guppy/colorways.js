import { Color } from 'three';

const palette = (...hex) => hex.map(value => new Color(value));

const COLORWAYS = {
  red: {
    body: palette(0x3e1a2f, 0xd26859, 0xf4c9ad),
    fin: palette(0x581f32, 0xcc3e38, 0xff8466),
    metalness: .88,
    iridescence: .32,
    leopard: 0,
  },
  leopard: {
    body: palette(0x2c302b, 0xb09c60, 0xe5cd90),
    fin: palette(0x72622b, 0xd5a532, 0xffdd78),
    metalness: .71,
    iridescence: .51,
    leopard: 1,
  },
  platinum: {
    body: palette(0x485365, 0xabbfca, 0xf4f3e8),
    fin: palette(0x637985, 0xadbfcb, 0xf3f3e8),
    metalness: 1.20,
    iridescence: .66,
    leopard: 0,
  },
};

const pigmentShader = /* glsl */ `
  uniform vec3 guppyPigmentDark;
  uniform vec3 guppyPigmentMiddle;
  uniform vec3 guppyPigmentLight;
  uniform float guppyFin;
  uniform float guppyLeopard;
  uniform float guppyMarkingSeed;

  float markingHash(vec2 point) {
    return fract(sin(dot(point, vec2(127.1, 311.7)) + guppyMarkingSeed) * 43758.5453);
  }

  float leopardSpots(vec2 uv, vec2 frequency) {
    vec2 cell = floor(uv * frequency);
    vec2 local = fract(uv * frequency);
    float edge = 2.0;
    for (int x = -1; x <= 1; x++) {
      for (int y = -1; y <= 1; y++) {
        vec2 offset = vec2(float(x), float(y));
        vec2 key = cell + offset;
        float t = markingHash(key);
        vec2 center = vec2(t, markingHash(key + vec2(19.3, 7.6)));
        vec2 diff = offset + .22 + .56 * center - local;
        float radius = .13 + .22 * markingHash(key + vec2(9.4, 6.2));
        float wobble = .029 * sin(4.0 * atan(diff.y, diff.x) + t * 6.28);
        edge = min(edge, length(diff * vec2(1.12, .94)) - radius + wobble + step(.94, t));
      }
    }
    float softEdge = max(.035, fwidth(edge));
    return 1.0 - smoothstep(-softEdge, softEdge, edge);
  }

  vec3 guppyPigment(vec3 original, vec2 uv) {
    // The source luminance retains small scales and the ridges between fin rays.
    float tone = dot(original, vec3(.2126, .7152, .0722));
    float ridge = pow(clamp(tone, 0.0, 1.0), mix(.51, .42, guppyFin));
    vec3 dyed = mix(guppyPigmentDark, guppyPigmentMiddle, smoothstep(.04, .49, ridge));
    dyed = mix(dyed, guppyPigmentLight, smoothstep(.34, .91, ridge));
    if (guppyLeopard > 0.5) {
      // The body wraps around the dorsal seam. Fade markings there.
      // The fin mask follows the pigment area, clear of the insertion.
      float bodyCoverage = (1.0 - smoothstep(.54, .87, uv.x)) * sin(3.141593 * uv.y);
      float finCoverage = 1.0 - smoothstep(.64, .88, uv.x);
      vec2 frequency = mix(vec2(15.0, 12.0), vec2(9.2, 15.0), guppyFin);
      float mask = leopardSpots(uv, frequency) * mix(bodyCoverage, finCoverage, guppyFin);
      dyed = mix(dyed, vec3(.014, .010, .008) * (.5 + ridge), mask * .94);
    }
    return dyed;
  }
`;

export function applyGuppyColorway(object, colorway, seed = 0) {
  const colors = COLORWAYS[colorway];
  if (!colors) return;

  const materials = new Map();
  object.traverse(mesh => {
    if (!mesh.isMesh || !mesh.material.map || mesh.name.startsWith('Cornea')) return;
    const source = mesh.material;
    if (materials.has(source)) {
      mesh.material = materials.get(source);
      return;
    }
    const fin = mesh.name !== 'Scales_and_gill_plates';
    const [dark, middle, light] = fin ? colors.fin : colors.body;
    const material = source.clone();
    material.metalness *= colors.metalness;
    if (!fin) material.iridescence *= colors.iridescence;
    material.onBeforeCompile = shader => {
      Object.assign(shader.uniforms, {
        guppyPigmentDark: { value: dark },
        guppyPigmentMiddle: { value: middle },
        guppyPigmentLight: { value: light },
        guppyFin: { value: Number(fin) },
        guppyLeopard: { value: colors.leopard },
        guppyMarkingSeed: { value: seed },
      });
      shader.fragmentShader = pigmentShader + shader.fragmentShader.replace('#include <map_fragment>', `
        vec4 pigment = texture2D(map, vMapUv);
        diffuseColor *= vec4(guppyPigment(pigment.rgb, vMapUv), pigment.a);
      `);
    };
    materials.set(source, material);
    mesh.material = material;
  });
}
