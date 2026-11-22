import assert from 'node:assert/strict'
import test from 'node:test'
import { checkAvatarFile, clampCrop, doubleTapZoom, initialCrop, isSmallCrop, panCrop, rotateCrop, sourceRect, zoomCrop } from '../src/lib/avatarCrop.ts'

const D = 300
const near = (a: number, b: number) => assert.ok(Math.abs(a - b) < 1e-6, `${a} ≈ ${b}`)

test('starts centred with the short side filling the circle', () => {
  const r = sourceRect(initialCrop(640, 480), D)
  near(r.size, 480)
  near(r.sx, 80)
  near(r.sy, 0)
})

test('panning never uncovers the circle', () => {
  const wide = initialCrop(640, 480)
  const s = panCrop(wide, D, 10_000, 10_000)
  near(s.ty, 0)
  const r = sourceRect(s, D)
  near(r.sx, 0)
  const left = sourceRect(panCrop(wide, D, -10_000, 0), D)
  near(left.sx + left.size, 640)
})

test('zoom keeps the chosen point still and stays within 1–4', () => {
  const s = zoomCrop(initialCrop(400, 400), D, 2, 50, 0)
  near(s.zoom, 2)
  near(s.tx, -50)
  assert.equal(zoomCrop(s, D, 9).zoom, 4)
  assert.equal(zoomCrop(s, D, 0.1).zoom, 1)
  near(sourceRect(zoomCrop(s, D, 0.1), D).size, 400)
})

test('rotating swaps the sides and keeps the crop valid', () => {
  const s = rotateCrop(panCrop(initialCrop(640, 480), D, 40, 0), D)
  assert.equal(s.iw, 480)
  assert.equal(s.ih, 640)
  near(s.tx, 0)
  near(s.ty, 40)
  assert.deepEqual(clampCrop(s, D), s)
})

test('small source and double tap', () => {
  assert.equal(isSmallCrop(initialCrop(200, 300)), true)
  assert.equal(isSmallCrop(initialCrop(1024, 768)), false)
  assert.equal(isSmallCrop({ ...initialCrop(1024, 768), zoom: 4 }), true)
  assert.equal(doubleTapZoom(1), 2.2)
  assert.equal(doubleTapZoom(2.2), 1)
})

test('local files: images only, at most 10 MB', () => {
  assert.equal(checkAvatarFile({ type: 'image/png', size: 1000 }), null)
  assert.equal(checkAvatarFile({ type: 'application/pdf', size: 1000 }), '请选择图片文件')
  assert.equal(checkAvatarFile({ type: 'image/jpeg', size: 10 * 1024 * 1024 + 1 }), '图片超过 10 MB，换一张试试')
})
