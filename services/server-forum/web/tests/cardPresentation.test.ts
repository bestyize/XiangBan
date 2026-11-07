import assert from 'node:assert/strict'
import test from 'node:test'
import { articleLayout, cardTitleLines, packLayout, wallpaperRatio, masonrySpan } from '../src/lib/cardPresentation.ts'

test('film titles get three lines; music, mixed discussion and other card titles get two', () => {
  for (const category of ['resource-movies','share-movies']) assert.equal(cardTitleLines(category),3)
  for (const category of ['', 'resource-music','resource-tools','learn-codex','casual-media','site-notice']) assert.equal(cardTitleLines(category),2)
})

test('film and music use masonry, while guides, questions and notices remain readable', () => {
  for (const category of ['resource-movies','resource-music','casual','casual-work','casual-offwork','casual-loss','casual-dream','casual-gear','casual-indie','casual-fun','casual-media','casual-think','casual-help','casual-chat']) assert.equal(articleLayout(category),'masonry')
  for (const category of ['learn-codex','learn-prompts','create-wallpaper','resource-help']) assert.equal(articleLayout(category),'reading')
  for (const category of ['site-notice','site-rules','site-feedback','site-reports']) assert.equal(articleLayout(category),'plain')
  for (const category of ['resource-tools','resource-docs']) assert.equal(articleLayout(category),'visual')
})
test('only a wallpaper work feed becomes masonry; abilities remain directories', () => {
  assert.equal(packLayout('wallpaper',true),'masonry')
  for (const kind of ['wallpaper','widget','effect','game']) assert.equal(packLayout(kind),'visual')
  for (const kind of ['reminder','routine','task','goal','skill','expert','workflow']) assert.equal(packLayout(kind,true),'directory')
})
test('portrait posters and square albums keep their ratios within preview bounds', () => {
  assert.equal(wallpaperRatio(600,900,2/3),2/3)
  assert.equal(wallpaperRatio(600,600,2/3),1)
  assert.equal(wallpaperRatio(900,600,2/3),1.5)
  assert.equal(wallpaperRatio(100,10000),4/5)
  assert.equal(wallpaperRatio(10000,100),16/9)
  for (const size of [[0,100],[100,0],[NaN,10],[10,Infinity]]) assert.equal(wallpaperRatio(size[0]!,size[1]!),16/9)
})
test('masonry reserves enough grid rows for each card without a full extra row', () => {
  for (const gap of [8,12,16]) for (const height of [1,140,287.5,399,651]) {
    const span=masonrySpan(height,4,gap), available=span*4+(span-1)*gap
    assert.ok(available>=height)
    assert.ok(available-height<4+gap)
  }
})
