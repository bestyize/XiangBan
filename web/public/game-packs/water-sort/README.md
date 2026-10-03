# 下班调饮料

`water-sort` is a self-contained, dependency-free classic-script game pack for the version 1 Xiaban host. Serve this directory as static files. All artwork, rules, and styling are local; the sandbox only needs `allow-scripts`, with no storage or external services.

## Play

Select a filled glass, then another glass. Its top flavor must match the source flavor, or it must be empty. Each pour moves the entire contiguous top flavor, limited by the destination's four-unit capacity. A level ends only when every nonempty glass is full and has one flavor. Flavor letters A–H and Chinese labels supplement colors.

There are eight fixed levels with 2, 3, 4, 5, 6, 6, 7, and 8 flavors. Undo restores the previous board and step count; reset restarts the current level. Finishing unlocks the next level. The menu can replay unlocked levels. There is no timer. Level unlocks and the completed-level count last for this loaded game session.

Native buttons support Tab / Shift+Tab and Enter / Space. Escape cancels a selected source. The completion dialog traps focus until the player selects its next-level or replay action. Small portrait and short landscape layouts are supported, as is reduced motion.

## Rules and verification

`rules.js` exposes `globalThis.XiabanWaterSort`. Boards are arrays of glasses; each glass lists flavors from bottom to top. Moves return a fresh board without mutating the input. The game uses the same rules that `web/src/games/waterSort.test.ts` loads through `node:vm`.

The fixed puzzles were constructed offline by reversing pours from fully sorted glasses with two spare glasses. Reverse moves never uncover a different flavor in the original source and never add to an already matching target top, so each reverse move has a legal forward inverse. Tests replay an explicit solution witness for all eight boards and independently search for solutions with a bounded solver. No solver or random generator runs during play.

## Host protocol and scores

The game accepts `xiaban.game-host / host.init` only from its parent, at protocol 1 with the URL `hostNonce`. It then sends `game.ready`, `game.start`, and current `game.progress`, echoing the nonce. Duplicate initialization does not start a second round. Valid pours and undo update progress. Score is the number of full, single-flavor glasses × 100; `lines` and `maxTile` are zero. Undo can reduce the score.

Each level is one round. Completion sends `game.result`; next level, reset, and replay send a new `game.start`. The host may save the resulting scores as game records. The pack does not read accounts or call website APIs. The exit button sends `game.exit`; when opened directly it navigates to `/games`.
