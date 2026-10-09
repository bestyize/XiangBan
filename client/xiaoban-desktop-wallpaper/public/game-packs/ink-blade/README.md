# 水墨剑客

A landscape side-scrolling action game painted entirely by code in an ink-wash style. The first 卷 (竹林卷) is six 幅 played one at a time and unlocked in order: 山门 → 竹林 → 鸦岭 → 断桥 → 夜雨 → a moonlit duel with the boss 无常. Clearing the 终幅 opens 长卷, the whole scroll in one run. The pack loads no image, font or audio files.

The design baseline (art style, gameplay, level layout, tuning values and the consistency rules for new content) is [PRD_水墨剑客.md](../../../../../docs/product/PRD_水墨剑客.md). Read it before changing the game, and keep its tables in sync with the code.

## Play

- Touch: drag left/right anywhere on the left 44% of the screen to walk. On the right, 斩 slashes (press repeatedly for a three-stage combo), 跳 jumps (holding gives a higher jump, and you can jump again in the air), 闪 dashes with brief invulnerability, and 一笔 unleashes a full-screen stroke once 墨意 is full.
- Keyboard: A/D or ←/→ to move, K/W/↑/Space to jump, J/X or left click to slash, L/Shift/C to dash, I/Q/U for 一笔, P/Esc to pause.
- Combat: an enemy blade glints cinnabar just before it strikes. Starting a slash inside the parry window (0.16 s on 江湖, 0.24 s on 入门) parries the blow. Ordinary enemies stagger and a fresh slash executes them. Arrows are knocked back and can kill their archer. Shield bearers block frontal hits, so parry them or dash behind them. Moves marked 「危」 cannot be parried; jump or dash through them.
- The boss has health and posture. Parries fill posture fastest. Once posture breaks, a new slash deals a heavy execution. At half health a second phase adds a phantom dash. Hitting it while it stands guard provokes a fast counter combo.
- Encounters lock the screen behind ink curtains until every wave is defeated. Score comes from hits, kills, parries and executions, multiplied by the combo, plus clear bonuses for time, remaining health and a flawless run. Score never decreases.
- 雪江卷 gives every 幅 its own terrain. 寒峰: slash hanging icicles to drop them on enemies below. 古渡: thin ice cracks open under anyone who stands still, so jump or dash out, or leave a rooted enemy in the crack. 冰泽: three runs of slick ice. 芦荡: crouch still in a reed bed to hide, which silences archers, and slash from hiding to ambush. 夜雪: timed gusts push the player, unbraced enemies and arrows after a one-second warning. 独钓: the arena is frozen. See the PRD, section 3.15.
- Each 幅 awards up to three seals: 过 for clearing, 全 for taking no more than that 幅's hits, and 技 for its own challenge (shown in the HUD while playing). A clear is graded 能品, 妙品, 神品 or 逸品 by score. The select screen paints a cleared 幅 in full, shows the next one as a pale draft and leaves locked ones blank.
- Death or clearing ends the round and opens the result dialog with the seals, the grade, 分享战绩, 下一幅, 再来一次 and 返回画卷. A shared result's challenge link opens the same 幅 directly as a duel (`?stage=<code>&target=<score>`); beating the score shows 破帖.

## Files

- `levels.js` — level data (`XiabanInkLevels`): each 卷's world (areas, platforms, tutorial prompts, encounters and their waves, and in 雪江卷 its `icicles`, `thinIce`, `iceZones`, `reeds` and `wind`) and each 幅 (its window of the world, start, encounters, par time, hit budget, 技 challenge and grade thresholds).
- `rules.js` — the deterministic simulation (`XiabanInkBlade`): player physics, combat, enemy and boss AI, encounters, scoring, seals and grades. It has no DOM access. `client/xiaban-web/src/games/ink-blade/rules.test.ts` runs this exact file.
- `progress.js` — the save document (`XiabanInkProgress`): validates untrusted saves, records clears and unlocks 幅. Tested by `progress.test.ts`.
- `art.js` — procedural painting: paper grain, ink-wash mountains with 皴 texture strokes and 点苔 tree dots, bamboo, ground, water, props, splashes, sun and moon. For 雪江卷 it also paints snowfields left as bare paper, snow pine forests, snow-laden rocks, plum boughs and ink clouds. Long parallax layers are split into chunks no wider than 2048 px.
- `figures.js` — skeletal ink figures with smoothed poses, a bent-wrist one-handed sword grip, two-handed spears, hammers and rods, fluttering robes, the Verlet scarf, crows and straw dummies.
- `scene.js` — scene composition: time-of-day keyframes along the scroll, parallax layers with bamboo sway, mist, weather (leaves, petals, rain and ripples; depth-layered snow and the blizzard in 雪江卷) and paper post-processing.
- `fx.js` — effects: dry-brush sword trails, splats, ground stains, seals, rings, sparks, ghosts and the 一笔 stroke.
- `audio.js` — Web Audio synthesis: steel and ink effects, a generative pentatonic pluck line, combat drums, rain, and in 雪江卷 a wind bed, ice chimes, footsteps on snow and ice, and the crack of the ice wave. Sound is off by default.
- `game.js` — the controller: host protocol, input, round lifecycle, event-driven effects, rendering and HUD.

## Host

The manifest sets `orientation: "landscape"`, and `viewport.js` rotates the game when the phone is held in portrait, exactly like the snake pack. The game reports `game.progress` / `game.result` with `score`, `lines` (kills) and `maxTile` (best combo). The shared `result-share.js` sends `{ score, lines: kills, maxTile: maxCombo, level: cleared ? 1 : 0, steps: parries, stage: levelCode, seals, grade }`, and `client/xiaban-web/src/games/ink-blade/index.ts` labels these as 击败 / 最高连击 / 弹反 and gives 小伴 its poster line. Share links use version 2 of the fragment; version 1 links from the single-scroll release still open, as 长卷.

The manifest also sets `"save": true`. The host passes the save document in `host.init` and the game sends `game.save` after every finished round and when the difficulty changes. Inside the host's sandboxed iframe the pack has no storage of its own. Opened directly (without the host, for development), it keeps progress in that page's `localStorage` under `xiaban.ink-blade.save` when storage is available, and in memory otherwise. Add `?debug=1` to expose `__ink.advance(seconds)` for stepping frames in hidden preview tabs.

Rendering targets about 2.4 megapixels of backing store. If frames stay slow, the game drops the paper-grain pass and lowers resolution once.
