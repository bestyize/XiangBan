// Water distortion and light adapted from RealWorldBuilder's aquarium backdrop.
const vertexSource = `#version 300 es
out vec2 uv;
void main() {
  vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
  uv = p;
  gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}`;

const fragmentSource = `#version 300 es
precision highp float;
uniform sampler2D backgroundImage;
uniform vec2 visibleFraction;
uniform float elapsed;
uniform float strength;
in vec2 uv;
out vec4 color;

void main() {
  vec2 p = 0.5 + (uv - 0.5) * visibleFraction;
  float surface = smoothstep(0.87, 0.995, p.y) * strength;
  float floorMask = (1.0 - smoothstep(0.04, 0.17, p.y)) * strength;
  float edges = smoothstep(0.0, 0.033, p.x) * (1.0 - smoothstep(0.967, 1.0, p.x));
  float floorWave = (1.0 - smoothstep(0.025, 0.135, p.y)) * edges;
  vec2 displacement = vec2(
    sin(p.y * 39.0 + p.x * 17.0 - elapsed * 0.72) + 0.49 * sin(p.x * 31.0 + p.y * 15.0 + elapsed * 0.51),
    0.52 * sin(p.x * 23.0 + p.y * 31.0 + elapsed * 1.21) + 0.38 * cos(p.x * 11.0 - p.y * 54.0 - elapsed * 0.70)
  );
  vec2 floorFlow = vec2(
    sin(p.y * 31.0 + p.x * 11.0 + elapsed * 0.73),
    0.46 * cos(p.x * 17.0 - p.y * 24.0 + elapsed * 0.83)
  );
  vec2 sampleUv = p + displacement * (0.00066 * strength + surface * 0.0034 + floorMask * 0.0042) * edges;
  sampleUv += floorFlow * floorWave * 0.0062 * strength;
  vec3 rgb = texture(backgroundImage, sampleUv).rgb;
  float drift = sin(p.x * 23.0 - p.y * 8.0 + elapsed * 0.37) * sin(p.x * 11.0 + p.y * 9.0 - elapsed * 0.51);
  float beam = pow(max(0.0, sin((p.x + p.y * 0.2 + 0.017 * sin(elapsed * 0.23)) * 28.0)), 8.0);
  float wash = smoothstep(0.60, 0.94, p.y) * (1.0 - smoothstep(0.92, 0.99, p.y));
  rgb *= 1.0 - 0.21 * floorMask + 0.086 * floorMask * drift + 0.065 * surface * drift;
  rgb = mix(rgb, vec3(dot(rgb, vec3(0.2126, 0.7152, 0.0722))), 0.085 * floorMask);
  rgb += vec3(0.014, 0.033, 0.049) * wash * beam * 0.26 * strength;
  vec3 srgb = mix(1.055 * pow(max(rgb, vec3(0.0)), vec3(1.0 / 2.4)) - 0.055,
    rgb * 12.92, lessThanEqual(rgb, vec3(0.0031308)));
  color = vec4(srgb, 1.0);
}`;

export default async function create({ canvas, settings, assets }) {
  const image = new Image();
  image.src = assets.background.url;
  await image.decode();
  // The same full-screen image remains visible if WebGL is unavailable or lost.
  document.body.style.background = `center / cover no-repeat url("${assets.background.url}")`;
  const gl = canvas.getContext('webgl2', { alpha: false, antialias: false });
  if (!gl) return { render() {} };

  const program = gl.createProgram();
  for (const [type, source] of [[gl.VERTEX_SHADER, vertexSource], [gl.FRAGMENT_SHADER, fragmentSource]]) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    gl.attachShader(program, shader);
    gl.deleteShader(shader);
  }
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    gl.deleteProgram(program);
    canvas.style.visibility = 'hidden';
    return { render() {} };
  }
  gl.useProgram(program);
  const visibleFraction = gl.getUniformLocation(program, 'visibleFraction');
  const elapsedUniform = gl.getUniformLocation(program, 'elapsed');
  const strengthUniform = gl.getUniformLocation(program, 'strength');
  const texture = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, true);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.SRGB8_ALPHA8, gl.RGBA, gl.UNSIGNED_BYTE, image);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  const onContextLost = () => { canvas.style.visibility = 'hidden'; };
  canvas.addEventListener('webglcontextlost', onContextLost);

  const imageAspect = image.naturalWidth / image.naturalHeight;
  let elapsed = 0;
  let strength = settings?.strength ?? 1;
  function draw() {
    gl.uniform1f(elapsedUniform, elapsed);
    gl.uniform1f(strengthUniform, strength);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
  }
  return {
    resize({ width, height, dpr }) {
      const scale = Math.min(dpr, 1920 / width, 1080 / height);
      canvas.width = Math.max(1, Math.round(width * scale));
      canvas.height = Math.max(1, Math.round(height * scale));
      gl.viewport(0, 0, canvas.width, canvas.height);
      const aspect = width / height;
      gl.uniform2f(visibleFraction, Math.min(1, aspect / imageAspect), Math.min(1, imageAspect / aspect));
      draw();
    },
    render({ dt }) {
      elapsed += dt;
      draw();
    },
    update(next) {
      strength = next.strength ?? strength;
      draw();
    },
    destroy() {
      canvas.removeEventListener('webglcontextlost', onContextLost);
      gl.deleteTexture(texture);
      gl.deleteProgram(program);
    },
  };
}
