# 小伴贪吃蛇

A mobile-first landscape Canvas game with local AI opponents. It runs entirely inside the existing sandboxed game-pack host; no multiplayer service or extra runtime dependency is needed.

## Play

- Hold the phone sideways: the entire game automatically rotates 90 degrees when the browser viewport stays portrait. The welcome screen and HUD have a direction-switch button for the other way of holding the phone. Native landscape viewports use the ordinary layout. Drag the joystick or any free area on the logical left of the arena to steer. Hold the logical right boost button for speed, consuming energy and a little mass. A second finger can boost while steering.
- Desktop: steer with the mouse, arrow keys or WASD; hold Space to boost. P / Escape pauses.
- Eat vivid candy dots; larger dots award more growth and points. Each dot is one flat color, without highlights, shading, glow or pulsing. Snakes use equally sized, solid-color circles with round heads and simple cartoon eyes; no tapered tails or glossy body surfaces. Spawn protection uses a separate head ring, keeping overlapping body circles fully opaque and uniform. An AI running into your body awards 80 points and releases colorful larger dots. Your own body is safe. Other snakes and arena boundaries are dangerous.
- A round lasts three active minutes or ends when the player is eliminated. Cozy mode has 8 AI opponents; lively mode has 11 and includes cut-off attempts. All snakes use the same movement, energy and collision rules.
- Cozy starts with a five-second player shield and opponents farther away so first-time phone players can learn steering; lively gives three seconds. AI avoids protected bodies as well as ordinary bodies. After protection expires, head/body collisions apply normally.
- The timer starts after the countdown. Backgrounding, focus loss, changing the physical orientation during play, switching the rotation direction and exiting fullscreen pause the round; resuming requires a tap. Native fullscreen changes during the initial countdown can complete without interrupting the first start. Sound is muted by default and enabled explicitly.

## Host and layout

`orientation: "landscape"` in the manifest opts into a wide host container. The host delegates fullscreen and permits orientation locking in the iframe sandbox. Starting on a touch device attempts fullscreen during the tap. Unsupported fullscreen/orientation APIs do not block play: a portrait viewport gets a swapped-size game container with a clockwise or counterclockwise CSS transform. This also works when opening the pack directly, without the React host. It does not hide browser chrome.

`viewport.js` defines the logical dimensions, physical-to-game pointer conversion and rotated safe-area mapping. Canvas resolution, camera, floating joystick placement and mouse aiming all use logical coordinates. Safe areas combine local physical environment insets with bounded numeric insets forwarded through `host.init` / `host.layout`, then map them to the transformed game edges. Input captures are released on layout changes. Compact CSS rules cover both ordinary landscape viewports and rotated portrait viewports.

The pack acknowledges `host.init` with `game.ready` while on its welcome screen, then emits `game.start` only after an actual start. A late host initialization also sends the active round once. Points are monotonic (boosting reduces mass, not points), and the existing guest/account score stores handle persistence. The result's personal best is explicitly limited to this visit; the hub shows the persisted best.

`rules.js` exposes `XiabanSnakeArena` without DOM dependencies. The renderer advances it in fixed 1/60-second steps, caps catch-up after stalls, culls distant sprites, caps display density at 2x, and reuses food and solid-circle body sprite canvases. Snake bodies follow resampled travel paths. Body spatial buckets and swept collision tests cover fast movement; simultaneous deaths are resolved before food collection. Tests execute this exact shipped rules script.
