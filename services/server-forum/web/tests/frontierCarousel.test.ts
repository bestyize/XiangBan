import assert from 'node:assert/strict'
import test from 'node:test'
import { carouselIndex, swipeStep } from '../src/lib/frontierCarousel.ts'

test('banner navigation wraps in both directions and handles empty or single-item feeds',()=>{
  assert.equal(carouselIndex(4,1,5),0)
  assert.equal(carouselIndex(0,-1,5),4)
  assert.equal(carouselIndex(0,1,0),0)
  assert.equal(carouselIndex(0,-1,1),0)
})
test('horizontal swipes switch slides while vertical scrolling and taps do not',()=>{
  assert.equal(swipeStep(-80,10),1)
  assert.equal(swipeStep(80,10),-1)
  assert.equal(swipeStep(-20,0),0)
  assert.equal(swipeStep(-80,100),0)
})
