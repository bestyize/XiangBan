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
```

Add a game by creating a package directory with its own entry and manifest, then add its ID to the catalog. The hub and player use the same loader for every package.

## Runtime protocol

Protocol version 1 uses `postMessage`:

- Host to game: `xiaban.game-host / host.init`, with a one-time nonce.
- Game to host: `xiaban.game-pack / game.ready` and `game.exit`, echoing that nonce.

The host verifies both the message source window and nonce. It does not pass account tokens, profile data, or website API clients to a game.

## Hosting packages separately

By default, the host loads `/game-packs` from the website. To serve game packs from a separate static host, set `VITE_GAME_PACK_BASE_URL` at web build time to that public base URL. The static host must allow browser GET requests for `catalog.json` and each package manifest from the website origin.
