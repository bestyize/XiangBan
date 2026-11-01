import { test } from 'node:test'
import { strict as assert } from 'node:assert'
import { fitGameViewport } from '../src/lib/gameViewport.ts'
test('a narrow desktop window keeps landscape gameplay upright',()=>{
  const box=fitGameViewport(670,1043,'landscape',true)
  assert.deepEqual(box,{width:670,height:376})
  assert.ok(box.width>box.height)
})
test('portrait games fit short desktop windows without stretching or cropping',()=>{
  assert.deepEqual(fitGameViewport(1440,500,'portrait',true),{width:375,height:500})
  assert.deepEqual(fitGameViewport(1920,1080,'portrait',true),{width:480,height:640})
})
test('touch screens retain native rotation space and zero-size stages do not overflow',()=>{
  assert.deepEqual(fitGameViewport(390,756,'landscape',false),{width:390,height:756})
  assert.deepEqual(fitGameViewport(0,0,'landscape',true),{width:0,height:0})
  const box=fitGameViewport(1200,480,'landscape',true)
  assert.ok(box.width<=1200 && box.height<=480 && box.width>box.height)
})
