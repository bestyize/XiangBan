# 小伴贪吃蛇

A mobile-first landscape Canvas game with local AI opponents. It runs entirely inside the existing sandboxed game-pack host; no multiplayer service or extra runtime dependency is needed.

## Play

- Landscape phone: drag the joystick or any free area on the left of the arena to steer. Hold the right boost button for speed, consuming energy and a little mass. A second finger can boost while steering.
- Desktop: steer with the mouse, arrow keys or WASD; hold Space to boost. P / Escape pauses.
- Eat vivid candy dots; larger dots award more growth and points. Each dot is one flat color, without highlights, shading, glow or pulsing. Snakes use equally sized, solid-color circles with round heads and simple cartoon eyes; no tapered tails or glossy body surfaces. Spawn protection uses a separate head ring, keeping overlapping body circles fully opaque and uniform. An AI running into your body awards 80 points and releases colorful larger dots. Your own body is safe. Other snakes and arena boundaries are dangerous.
- A round lasts three active minutes or ends when the player is eliminated. Cozy mode has 8 AI opponents; lively mode has 11 and includes cut-off attempts. All snakes use the same movement, energy and collision rules.
- Cozy starts with a five-second player shield and opponents farther away so first-time phone players can learn steering; lively gives three seconds. AI avoids protected bodies as well as ordinary bodies. After protection expires, head/body collisions apply normally.
- The timer starts after the countdown. Backgrounding, focus loss, portrait rotation and exiting fullscreen pause the round; resuming requires a tap. Sound is muted by default and enabled explicitly.

## Host and layout

`orientation: "landscape"` in the manifest opts into a wide host container. The host delegates fullscreen and permits orientation locking in the iframe sandbox. Starting on a touch device attempts fullscreen during the tap; unsupported browsers retain normal landscape play. iPhone Safari uses manual rotation. CSS respects display safe areas, including bounded numeric insets forwarded by the host through `host.init` / `host.layout` for browsers that resolve iframe inset variables to zero.

The pack acknowledges `host.init` with `game.ready` while on its welcome screen, then emits `game.start` only after an actual start. A late host initialization also sends the active round once. Points are monotonic (boosting reduces mass, not points), and the existing guest/account score stores handle persistence. The result's personal best is explicitly limited to this visit; the hub shows the persisted best.

`rules.js` exposes `XiabanSnakeArena` without DOM dependencies. The renderer advances it in fixed 1/60-second steps, caps catch-up after stalls, culls distant sprites, caps display density at 2x, and reuses food and solid-circle body sprite canvases. Snake bodies follow resampled travel paths. Body spatial buckets and swept collision tests cover fast movement; simultaneous deaths are resolved before food collection. Tests execute this exact shipped rules script.
