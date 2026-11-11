"""Render the original 'Garden Skip' instrumental loop. No samples or external service."""
from pathlib import Path
import json
import subprocess
import tempfile
import wave
import numpy as np
import imageio_ffmpeg

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'assets/game-packs/pocket-sprites/assets'
SR, BPM, BARS = 44100, 110, 32
BEAT = 60 / BPM
N = round(BARS * 4 * BEAT * SR)
mix = np.zeros((N, 2), dtype=np.float64)
rng = np.random.default_rng(1142)

def add(signal, beat, gain=1, pan=0):
    pos = (round(beat * BEAT * SR) + np.arange(len(signal))) % N
    mix[pos, 0] += signal * gain * np.sqrt((1 - pan) / 2)
    mix[pos, 1] += signal * gain * np.sqrt((1 + pan) / 2)

def tone(midi, beats, kind='pluck'):
    f = 440 * 2 ** ((midi - 69) / 12)
    t = np.arange(round((beats * BEAT + .45) * SR)) / SR
    attack = 1 - np.exp(-t * 420)
    if kind == 'bell':
        y = sum(a * np.sin(2*np.pi*f*h*t) * np.exp(-t*d)
                for h,a,d in [(1,1,3.7),(2.01,.28,8),(3.98,.08,13)])
    elif kind == 'bass':
        y = (np.sin(2*np.pi*f*t) + .22*np.sin(4*np.pi*f*t)) * np.exp(-t*5)
    else:
        y = sum(np.sin(2*np.pi*f*h*t + .04*h) * (.7**(h-1)/h) * np.exp(-t*(3.8+h*1.4))
                for h in range(1,7))
    release = np.minimum(1, np.maximum(0, (len(t)/SR-t)/.08))
    return y * attack * release

# C6 – Am7 – Fmaj7 – G6; two eight-bar melodies with answering phrases.
chords = [(48,[60,64,67,69]),(45,[60,64,67,69]),(41,[60,64,65,69]),(43,[59,62,67,69])]
motifs = [
 [(0,76,.5),(.75,79,.5),(1.5,81,.5),(2.5,79,.5),(3.25,76,.5)],
 [(0,72,.75),(1,76,.5),(2,79,.5),(3,76,.5)],
 [(.5,77,.5),(1.25,76,.5),(2,74,.5),(3,72,.75)],
 [(0,74,.5),(1,71,.5),(2,67,.75),(3.25,74,.5)],
 [(0,79,.75),(1.25,81,.5),(2,84,.75),(3.25,81,.5)],
 [(0,79,.5),(.75,76,.5),(1.5,72,.75),(3,76,.5)],
 [(0,77,.5),(1,81,.75),(2.25,79,.5),(3,77,.5)],
 [(0,74,.5),(.75,76,.5),(1.5,74,.5),(2.5,71,.5),(3.25,72,.5)],
]
for bar in range(BARS):
    base = bar*4
    root, chord = chords[bar%4]
    add(tone(root, .6, 'bass'), base, .2, -.05)
    add(tone(root+7, .45, 'bass'), base+2, .14, .05)
    for off in [.5,1.5,2.5,3.5]:
        for j,note in enumerate(chord):
            add(tone(note,.3),base+off+j*.018,.065,-.28)
    for j, note in enumerate(chord):
        add(tone(note+12,.4),base+j+.1,.07,.35)
    for off,note,length in motifs[bar%8]:
        # Answering phrase stays within the same friendly, non-shrill register.
        adjusted = note if (bar//8)%2==0 else note-12
        add(tone(adjusted,length,'bell'),base+off,.18,.12)
    if bar%4==3:
        for j,note in enumerate([79,76,72]):
            add(tone(note,.25),base+2.5+j*.5,.065,-.4)
    for off in [0,2]:
        t=np.arange(round(.16*SR))/SR
        kick=np.sin(2*np.pi*(58*t+3*(1-np.exp(-t*28))))*np.exp(-t*26)
        add(kick,base+off,.11)
    for off in [1,3]:
        t=np.arange(round(.095*SR))/SR
        wood=(np.sin(2*np.pi*860*t)+.3*np.sin(2*np.pi*1350*t))*np.exp(-t*75)
        add(wood,base+off,.055,-.15)
    for eighth in range(8):
        t=np.arange(round(.06*SR))/SR
        noise=rng.normal(0,1,len(t)); noise[1:]-=.85*noise[:-1]
        add(noise*np.exp(-t*65),base+eighth*.5,.014 if eighth%2 else .009,.5)

# Circular stereo early reflections keep the musical loop continuous.
dry=mix.copy()
for seconds, gain in [(.071,.12),(.113,.085),(.193,.055),(.307,.035)]:
    mix += np.roll(dry[:,::-1],round(seconds*SR),axis=0)*gain
mix = np.tanh(mix*1.25)
mix[-256:] += (mix[0]-mix[-1]) * ((1-np.cos(np.linspace(0,np.pi,256)))/2)[:,None]
mix *= .76 / np.max(np.abs(mix))
OUT.mkdir(parents=True,exist_ok=True)
with tempfile.TemporaryDirectory(prefix='pocket-music-') as tmp:
    wav=Path(tmp)/'garden-skip.wav'
    with wave.open(str(wav),'wb') as out:
        out.setnchannels(2); out.setsampwidth(2); out.setframerate(SR)
        out.writeframes((mix*32767).astype('<i2').tobytes())
    subprocess.run([imageio_ffmpeg.get_ffmpeg_exe(),'-hide_banner','-loglevel','error','-y','-i',str(wav),
                    '-codec:a','libmp3lame','-b:a','160k',str(OUT/'garden-skip.mp3')],check=True)
metadata={'title':'Garden Skip / 花园小步舞','source':'Original procedural composition; no third-party samples',
          'bpm':BPM,'bars':BARS,'duration':N/SR,'loopStart':0,'loopEnd':N/SR,
          'instrumentation':['nylon-style plucks','wooden mallet melody','soft bass','woodblock','shaker'],
          'peak':float(np.max(np.abs(mix))),'rms':float(np.sqrt(np.mean(mix**2))),
          'loopSeamDelta':float(np.max(np.abs(mix[0]-mix[-1])))}
(OUT/'music.json').write_text(json.dumps(metadata,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(metadata,ensure_ascii=False))
