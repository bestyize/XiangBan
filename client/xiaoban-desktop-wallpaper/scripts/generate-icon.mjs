import { spawnSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
import { inflateSync } from 'node:zlib';

/**
 * The canonical icon is icons/icon-source.svg: exactly the Lucide Sparkles
 * used before "小伴 AI", in white over the default #D93A44 小伴红.
 * @tauri-apps/cli is an existing dev dependency, not a new image dependency.
 * Run before packaging; it creates .ico (Windows), .icns (macOS), .png (Linux)
 * and the raw .rgba that setup_tray includes without a runtime image decoder.
 */
const root = fileURLToPath(new URL('..', import.meta.url));
const iconDir = join(root, 'src-tauri/icons');
const cli = join(root, 'node_modules/@tauri-apps/cli/tauri.js');
const source = join(iconDir, 'icon-source.svg');
const result = spawnSync(process.execPath, [cli, 'icon', source, '--output', iconDir], {
  cwd: root, stdio: 'inherit', windowsHide: true,
});
if (result.error) throw result.error;
if (result.status !== 0) throw new Error(`Tauri icon generation failed (${result.status ?? result.signal})`);

// Tray icon must be uncompressed 64×64 RGBA; decode the newly generated PNG.
// PNG channels 3/4 and all standard row filters are supported without add-ons.
const bytes = await readFile(join(iconDir, 'icon.png'));
const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
if (!bytes.subarray(0, 8).equals(signature)) throw Error('Invalid generated icon.png');
let width = 0, height = 0, channels = 0, depth = 0, interlace = 0;
const idats = [];
for (let pos = 8; pos + 12 <= bytes.length;) {
  const length = bytes.readUInt32BE(pos);
  if (pos + 12 + length > bytes.length) throw Error('Truncated icon.png');
  const type = bytes.toString('ascii', pos + 4, pos + 8);
  const data = bytes.subarray(pos + 8, pos + 8 + length);
  if (type === 'IHDR') {
    width = data.readUInt32BE(0); height = data.readUInt32BE(4);
    depth = data[8]; channels = data[9] === 6 ? 4 : data[9] === 2 ? 3 : 0;
    interlace = data[12];
  } else if (type === 'IDAT') idats.push(data);
  pos += length + 12;
  if (type === 'IEND') break;
}
if (width !== height || width < 64 || width % 64 !== 0 || depth !== 8 || !channels || interlace !== 0)
  throw Error('Expected a square 8-bit RGB/RGBA icon PNG with dimension divisible by 64');
const raw = inflateSync(Buffer.concat(idats));
const stride = width * channels;
if (raw.length !== height * (stride + 1)) throw Error('Invalid PNG scanline length');
const rgba = Buffer.alloc(width * height * 4);
let previous = Buffer.alloc(stride);
for (let y = 0; y < height; y++) {
  const filter = raw[y * (stride + 1)];
  if (filter > 4) throw Error('Unsupported PNG filter');
  const line = Buffer.from(raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1)));
  for (let x = 0; x < stride; x++) {
    const left = x >= channels ? line[x - channels] : 0, above = previous[x];
    const upperLeft = x >= channels ? previous[x - channels] : 0;
    const prediction = left + above - upperLeft;
    const paeth = Math.abs(prediction - left) <= Math.abs(prediction - above)
      && Math.abs(prediction - left) <= Math.abs(prediction - upperLeft) ? left
      : Math.abs(prediction - above) <= Math.abs(prediction - upperLeft) ? above : upperLeft;
    line[x] = (line[x] + [0, left, above, (left + above) >>> 1, paeth][filter]) & 255;
  }
  for (let x = 0; x < width; x++) {
    const source = x * channels, target = (y * width + x) * 4;
    rgba[target] = line[source]; rgba[target + 1] = line[source + 1];
    rgba[target + 2] = line[source + 2]; rgba[target + 3] = channels === 4 ? line[source + 3] : 255;
  }
  previous = line;
}
const size = 64, scale = width / size, tray = Buffer.alloc(size * size * 4);
for (let y = 0; y < size; y++) for (let x = 0; x < size; x++) {
  let red = 0, green = 0, blue = 0, alpha = 0;
  for (let sy = y * scale; sy < (y + 1) * scale; sy++)
    for (let sx = x * scale; sx < (x + 1) * scale; sx++) {
      const p = (sy * width + sx) * 4, a = rgba[p + 3];
      red += rgba[p] * a; green += rgba[p + 1] * a; blue += rgba[p + 2] * a; alpha += a;
    }
  const dst = (y * size + x) * 4;
  tray[dst] = alpha ? Math.round(red / alpha) : 0;
  tray[dst + 1] = alpha ? Math.round(green / alpha) : 0;
  tray[dst + 2] = alpha ? Math.round(blue / alpha) : 0;
  tray[dst + 3] = Math.round(alpha / scale / scale);
}
await writeFile(join(iconDir, 'tray.rgba'), tray);
