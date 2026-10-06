# Game resource pack contract

The game host reads `catalog.json`, loads each listed package's `manifest.json`, and opens its `entry` in a sandboxed iframe. Game rules and assets live in the package directory; they do not import React components from the website.

## Package layout

```text
game-packs/
  catalog.json
  fresh-theme.css
  result-share.js
  result-share.css
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
- Result sharing: `game.share` carries final aggregate metrics. The host accepts it only from the verified current game's iframe for its completed current round and matching recorded metrics, then opens `/games/<gameId>/share`. The page shares a QR-coded PNG or result link and offers a guest-play challenge entry; it never imports shared metrics as account records.

## Required end-of-round experience for every new game

Every game must automatically show a result dialog when the player dies, loses, runs out of valid moves, reaches the time limit, or completes a level. The dialog shows final results and a prominent **分享战绩** action alongside restart / next level. Players may continue without sharing; never open an OS share sheet automatically. Pausing, leaving the page, and mid-round progress do not show this dialog or generate shareable finished results.

For portrait packs, include `../result-share.css` and load `../result-share.js` before `game.js`. Add a hidden `shareResultButton` with `result-share-button` styling inside the existing result dialog. On a real round ending, call `window.XiabanResultShare?.finish({ gameId, score, lines: 0, maxTile: 0, level: 0, steps: 0 })` with applicable final metrics and send `game.result` with matching score / lines / maxTile. Call `reset()` when starting another round. Snake retains its landscape-specific result dialog and protocol payload. A 2048 milestone that allows continued play is not a finished round; share when no valid moves remain.

Register each new game's name, allowed result metrics and poster presentation in the host sharing modules (`otherResultShare.ts` / `otherResultPoster.ts`, or a game-specific implementation). The result page must work without login, carry a public challenge URL and QR code, and offer image saving and link copying when native sharing is unavailable. Share only final aggregate game data and an optional public invitation code; never include account IDs, tokens or the host nonce. Validate the iframe source, nonce, game ID, completed-round state and recorded results before accepting a share request. Include the share button in dialog keyboard focus handling, use at least 44px touch targets, and keep restart / next level reachable on phone screens.

The host verifies both the message source window and nonce. It does not pass account tokens, profile data, or website API clients to a game.

The host saves guest records in the current browser and signed-in records through the game's session API. Packs do not read account or local-storage data directly. New package IDs must also be allowed by `server/src/main/kotlin/xyz/xiaban/server/games/GameRecords.kt`.

## Included games

| ID | Game | Rules |
| --- | --- | --- |
| `falling-blocks` | 俄罗斯方块 | Falling blocks and row clearing |
| `merge-2048` | 2048 | Slide and merge matching numbers |
| `block-puzzle` | 消失高手 | Place three pieces per batch on an 8×8 board and clear full rows/columns |
| `water-sort` | 奶茶大师 | Pour matching colors into tubes, with undo and progressively harder levels |
| `snake-arena` | 小伴贪吃蛇 | Landscape, mobile-first arena with local AI snakes, boost energy and three-minute or endless rounds |
| `ink-blade` | 水墨剑客 | Landscape ink-wash side-scrolling action: parries, executions, five painted areas and a boss duel, all drawn by code |

Packs may set `orientation: "landscape"` to use the full viewport instead of the phone-width host. The host permits fullscreen and orientation locking; the game must retain manual-rotation play when those APIs are unavailable. The site's PWA orientation is `any` so a landscape pack can also rotate when launched from the home screen.

## Hosting packages separately

By default, the host loads `/game-packs` from the website. To serve game packs from a separate static host, set `VITE_GAME_PACK_BASE_URL` at web build time to that public base URL. The static host must allow browser GET requests for `catalog.json` and each package manifest from the website origin.
