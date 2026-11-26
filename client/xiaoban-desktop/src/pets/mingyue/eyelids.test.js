import { readFileSync } from 'node:fs';
import { expect, it } from 'vitest';
import { eyelidFrame, prepareFaceCues } from './eyelids.js';

it('keeps one head through both blinks in every authored action', () => {
  const glb = readFileSync(new URL('../../../public/pets/mingyue/dafeiyu-maid.glb', import.meta.url));
  const length = glb.readUInt32LE(12);
  const gltf = JSON.parse(glb.subarray(20, 20 + length).toString());
  function floats(index, stride) {
    const accessor = gltf.accessors[index], view = gltf.bufferViews[accessor.bufferView];
    return new Float32Array(glb.buffer, glb.byteOffset + 28 + length + view.byteOffset + (accessor.byteOffset ?? 0), accessor.count * stride);
  }
  const faces = ['WhaleFace.scale', 'Smile.scale', 'Blink.scale', 'SmileBlink.scale', 'ClosedEyes.scale'];
  const clips = gltf.animations.map(animation => ({
    name: animation.name, duration: 10,
    tracks: animation.channels.map(({ target, sampler }) => {
      const name = `${gltf.nodes[target.node].name}.${target.path}`;
      if (!faces.includes(name)) return { name };
      const { input, output } = animation.samplers[sampler];
      return { name, times: floats(input, 1), values: floats(output, 3) };
    }),
  }));
  const bodyTracks = clips.map(clip => clip.tracks.filter(track => !faces.includes(track.name)));
  const cues = prepareFaceCues(clips);
  for (const [i, clip] of clips.entries()) {
    expect(clip.tracks).toEqual(bodyTracks[i]);
    const cue = cues.get(clip);
    expect(cue.smiling).toBe(['contact', 'swing', 'play'].includes(clip.name));
    expect(cue.asleep).toBe(clip.name === 'sleep');
    const times = [0, 3.73, 3.78, 3.82, 3.93, 7.70, 7.81, 0];
    expect(times.map(time => eyelidFrame(cue, time))).toEqual(
      cue.asleep ? times.map(() => 4) : [-1, -1, 2, 4, -1, 4, -1, -1],
    );
  }
});
