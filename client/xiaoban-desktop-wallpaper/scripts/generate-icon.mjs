import { mkdir, writeFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'

// A tiny native ICO, no third-party image tool or extra runtime dependency.
const width = 32, height = 32
const bitmap = Buffer.alloc(40 + width * height * 4 + height * 4)
bitmap.writeUInt32LE(40, 0); bitmap.writeInt32LE(width, 4); bitmap.writeInt32LE(height * 2, 8)
bitmap.writeUInt16LE(1, 12); bitmap.writeUInt16LE(32, 14)
bitmap.writeUInt32LE(width * height * 4, 20)
for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
  const inside = x >= 3 && x < 29 && y >= 5 && y < 29
  const screen = x >= 7 && x < 25 && y >= 11 && y < 24
  const eye = y >= 14 && y < 17 && ((x >= 10 && x < 13) || (x >= 19 && x < 22))
  const rgb = eye ? [234, 249, 255] : screen ? [9, 10, 14] : [246, 235, 219]
  const offset = 40 + ((height - y - 1) * width + x) * 4
  bitmap[offset] = rgb[2]; bitmap[offset + 1] = rgb[1]; bitmap[offset + 2] = rgb[0]; bitmap[offset + 3] = inside ? 255 : 0
}
const header = Buffer.alloc(22)
header.writeUInt16LE(1, 2); header.writeUInt16LE(1, 4)
header[6] = width; header[7] = height
header.writeUInt16LE(1, 10); header.writeUInt16LE(32, 12)
header.writeUInt32LE(bitmap.length, 14); header.writeUInt32LE(22, 18)
const dir = fileURLToPath(new URL('../src-tauri/icons/', import.meta.url))
await mkdir(dir, { recursive: true })
await writeFile(`${dir}/icon.ico`, Buffer.concat([header, bitmap]))
