import assert from 'node:assert/strict';
import {createRewards,rewardFor} from '../assets/game-packs/pocket-sprites/rewards.mjs';
const wave=(changes={})=>({type:'clear',chain:1,created:[],cleared:[1,2,3],combo:null,...changes});
assert.equal(rewardFor([wave()],'playing'),null);
assert.equal(rewardFor([wave({created:[{tile:{special:'row'}}]})],'playing'),'nice');
assert.equal(rewardFor([wave({created:[{tile:{special:'bomb'}}]})],'playing'),'good');
assert.equal(rewardFor([wave({chain:2})],'playing'),'good');
assert.equal(rewardFor([wave({chain:3})],'playing'),'great');
assert.equal(rewardFor([wave({created:[{tile:{special:'rainbow'}}]})],'playing'),'great');
assert.equal(rewardFor([wave({combo:'line-line'})],'playing'),'great');
assert.equal(rewardFor([wave({chain:4})],'playing'),'wonderful');
assert.equal(rewardFor([],'won'),'wonderful');
assert.equal(rewardFor([wave({chain:4})],'lost'),null);
const sources=[],ducks=[];
const context={currentTime:0,state:'running',destination:{},
 decodeAudioData:async()=>({duration:.7}),
 createGain:()=>({gain:{value:0},connect(){},disconnect(){}}),
 createBufferSource(){const source={connect(){},disconnect(){},start(){this.started=true},stop(){this.stopped=true}};sources.push(source);return source},
};
const original=globalThis.fetch;
globalThis.fetch=async()=>({ok:true,arrayBuffer:async()=>new ArrayBuffer(1)});
try {
 let allowed=true;
 const voice=createRewards(context,()=>allowed,duration=>ducks.push(duration));
 assert.equal(voice.play('good'),false,'no late queued callout before decode');
 await voice.load();
 assert.equal(voice.play('nice'),true);
 assert.equal(voice.play('great'),false,'do not overlap current word');
 sources.at(-1).onended(); context.currentTime=.8;
 assert.equal(voice.play('nice'),false,'cooldown');
 assert.equal(voice.play('good'),true,'higher achievement can bypass cooldown after the word ends');
 voice.stop(); assert.equal(sources.at(-1).stopped,true);
 allowed=false;context.currentTime=4;
 assert.equal(voice.play('wonderful',true),false,'mute/background gate');
 allowed=true;context.state='suspended';assert.equal(voice.play('good'),false);
 context.state='running';assert.equal(voice.play('wonderful',true),true);
 assert.equal(ducks.length,3);
 assert.ok(ducks.every(duration=>duration>.7));
 voice.stop();
 globalThis.fetch=async()=>({ok:false});
 const unavailable=createRewards(context,()=>true,()=>{});await unavailable.load();
 assert.equal(unavailable.play('nice'),false,'unavailable audio is harmless');
 console.log('PASS rewards: achievement tiers, no ordinary praise, cooldown, no overlap, mute/pause and ducking');
}finally{globalThis.fetch=original}
