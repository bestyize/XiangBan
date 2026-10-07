# 水墨剑客

A landscape side-scrolling action game painted entirely by code in an ink-wash style. Each run is one scroll of about three minutes: 山门 → 竹林 → 鸦岭 → 断桥 → 夜雨 → a moonlit duel with the boss 无常. The pack loads no image, font or audio files.

## Play

- Touch: drag left/right anywhere on the left 44% of the screen to walk. On the right, 斩 slashes (press repeatedly for a three-stage combo), 跳 jumps (holding gives a higher jump, and you can jump again in the air), 闪 dashes with brief invulnerability, and 一笔 unleashes a full-screen stroke once 墨意 is full.
- Keyboard: A/D or ←/→ to move, K/W/↑/Space to jump, J/X or left click to slash, L/Shift/C to dash, I/Q/U for 一笔, P/Esc to pause.
- Combat: an enemy blade glints cinnabar just before it strikes. Starting a slash inside the parry window (0.16 s on 江湖, 0.24 s on 入门) parries the blow. Ordinary enemies stagger and a fresh slash executes them. Arrows are knocked back and can kill their archer. Shield bearers block frontal hits, so parry them or dash behind them. Moves marked 「危」 cannot be parried; jump or dash through them.
- The boss has health and posture. Parries fill posture fastest. Once posture breaks, a new slash deals a heavy execution. At half health a second phase adds a phantom dash. Hitting it while it stands guard provokes a fast counter combo.
- Encounters lock the screen behind ink curtains until every wave is defeated. Score comes from hits, kills, parries and executions, multiplied by the combo, plus clear bonuses for time, remaining health and a flawless run. Score never decreases.
- Death or clearing ends the round and opens the result dialog with 分享战绩. Once the boss arena has been reached during this visit, 直面无常 starts a new round at the boss with score counted from zero.

## Files

- `rules.js` — the deterministic simulation (`XiabanInkBlade`): player physics, combat, enemy and boss AI, encounters and scoring. It has no DOM access. `src/games/inkBlade.test.ts` runs this exact file.
- `art.js` — procedural painting: paper grain, ink-wash mountains with 皴 texture strokes and 点苔 tree dots, bamboo, ground, water, props, splashes, sun and moon. Long parallax layers are split into chunks no wider than 2048 px.
- `figures.js` — skeletal ink figures with smoothed poses, two-bone sword arms, fluttering robes, the Verlet scarf, crows and straw dummies.
- `scene.js` — scene composition: time-of-day keyframes along the scroll, parallax layers with bamboo sway, mist, weather (leaves, petals, rain and ripples) and paper post-processing.
- `fx.js` — effects: dry-brush sword trails, splats, ground stains, seals, rings, sparks, ghosts and the 一笔 stroke.
- `audio.js` — Web Audio synthesis: steel and ink effects, a generative pentatonic pluck line, combat drums and rain. Sound is off by default.
- `game.js` — the controller: host protocol, input, round lifecycle, event-driven effects, rendering and HUD.

## Host

The manifest sets `orientation: "landscape"`, and `viewport.js` rotates the game when the phone is held in portrait, exactly like the snake pack. The game reports `game.progress` / `game.result` with `score`, `lines` (kills) and `maxTile` (best combo). The shared `result-share.js` sends `{ score, lines: kills, maxTile: maxCombo, level: cleared ? 1 : 0, steps: parries }`, and `otherResultShare.ts` labels these as 击败 / 最高连击 / 弹反.

Rendering targets about 2.4 megapixels of backing store. If frames stay slow, the game drops the paper-grain pass and lowers resolution once.
