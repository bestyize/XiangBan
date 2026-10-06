import { mkdir, writeFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { deflateSync } from 'node:zlib'

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

// PNG/ICNS use the same app-icon design, scaled for Retina. Encode locally so
// building the Mac app needs no image service or extra imaging dependency.
function crc32(bytes) {
  let crc = 0xffffffff
  for (const byte of bytes) {
    crc ^= byte
    for (let bit = 0; bit < 8; bit++) crc = (crc >>> 1) ^ ((crc & 1) ? 0xedb88320 : 0)
  }
  return (crc ^ 0xffffffff) >>> 0
}
function chunk(type, data) {
  const body = Buffer.concat([Buffer.from(type), data])
  const size = Buffer.alloc(4); size.writeUInt32BE(data.length)
  const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(body))
  return Buffer.concat([size, body, crc])
}
function png(size) {
  const rows = Buffer.alloc(size * (size * 4 + 1))
  for (let y = 0; y < size; y++) for (let x = 0; x < size; x++) {
    const sourceX = Math.floor(x * 32 / size), sourceY = Math.floor(y * 32 / size)
    const source = 40 + ((31 - sourceY) * 32 + sourceX) * 4
    const offset = y * (size * 4 + 1) + 1 + x * 4
    rows[offset] = bitmap[source + 2]; rows[offset + 1] = bitmap[source + 1]
    rows[offset + 2] = bitmap[source]; rows[offset + 3] = bitmap[source + 3]
  }
  const ihdr = Buffer.alloc(13)
  ihdr.writeUInt32BE(size); ihdr.writeUInt32BE(size, 4); ihdr[8] = 8; ihdr[9] = 6
  return Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]), chunk('IHDR', ihdr), chunk('IDAT', deflateSync(rows)), chunk('IEND', Buffer.alloc(0))])
}
const icon256 = png(256), icon512 = png(512)
await writeFile(`${dir}/icon.png`, icon256)
const entries = [['ic08', icon256], ['ic09', icon512]].map(([type, image]) => {
  const header = Buffer.alloc(8); header.write(type); header.writeUInt32BE(image.length + 8, 4)
  return Buffer.concat([header, image])
})
const icnsHeader = Buffer.alloc(8)
icnsHeader.write('icns'); icnsHeader.writeUInt32BE(8 + entries.reduce((total, entry) => total + entry.length, 0), 4)
await writeFile(`${dir}/icon.icns`, Buffer.concat([icnsHeader, ...entries]))
