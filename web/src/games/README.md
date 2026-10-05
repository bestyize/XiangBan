# Game resource pack contract

The game host reads `catalog.json`, loads each listed package's `manifest.json`, and opens its `entry` in a sandboxed iframe. Game rules and assets live in the package directory; they do not import React components from the website.

## Package layout

```text
game-packs/
  catalog.json
  falling-blocks/
    manifest.json
    index.html
    game.js
    game.css
    cover.svg
  block-puzzle/
    manifest.json
    index.html
    rules.js
    game.js
    game.css
    cover.svg
  water-sort/
    manifest.json
    index.html
    rules.js
    game.js
    game.css
    cover.svg
  merge-2048/
    manifest.json
    index.html
    rules.js
    game.js
    game.css
    cover.svg
```

Add a game by creating a package directory with its own entry and manifest, then add its ID to the catalog. The hub and player use the same loader for every package.

## Runtime protocol

Protocol version 1 uses `postMessage`:

- Host to game: `xiaban.game-host / host.init`, with a one-time nonce.
- Game to host: `xiaban.game-pack / game.ready` and `game.exit`, echoing that nonce.
- Records: `game.start` starts a round, `game.progress` updates its nonnegative integer `score` (and optional `lines` / `maxTile`), and `game.result` finishes it. A game sends its current round once after the first valid host initialization, then starts a new round when restarted or when entering another level.

The host verifies both the message source window and nonce. It does not pass account tokens, profile data, or website API clients to a game.

The host saves guest records in the current browser and signed-in records through the game's session API. Packs do not read account or local-storage data directly. New package IDs must also be allowed by `server/src/main/kotlin/xyz/xiaban/server/games/GameRecords.kt`.

## Included games

| ID | Game | Rules |
| --- | --- | --- |
| `falling-blocks` | 俄罗斯方块 | Falling blocks and row clearing |
| `merge-2048` | 2048 | Slide and merge matching numbers |
| `block-puzzle` | 小伴消方块 | Place three pieces per batch on an 8×8 board and clear full rows/columns |
| `water-sort` | 调饮料大师 | Pour matching colors into tubes, with undo and progressively harder levels |
| `snake-arena` | 小伴贪吃蛇 | Landscape, mobile-first arena with local AI snakes, boost energy and three-minute rounds |

Packs may set `orientation: "landscape"` to use the full viewport instead of the phone-width host. The host permits fullscreen and orientation locking; the game must retain manual-rotation play when those APIs are unavailable. The site's PWA orientation is `any` so a landscape pack can also rotate when launched from the home screen.

## Hosting packages separately

By default, the host loads `/game-packs` from the website. To serve game packs from a separate static host, set `VITE_GAME_PACK_BASE_URL` at web build time to that public base URL. The static host must allow browser GET requests for `catalog.json` and each package manifest from the website origin.
