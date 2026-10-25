# Xiaoban Desktop application icon

The single editable source is **`icon-source.svg`**. It reproduces the Lucide
`Sparkles` mark used immediately before **小伴 AI** in
`src/ui/ControlShell.vue`, in white over the **default 小伴红** (`#D93A44`,
`src/ui/appearance.ts`). The application icon is intentionally fixed to the
brand's default red and does **not** follow the user's runtime accent selection.

## Regenerate native assets

From `client/xiaoban-desktop-wallpaper`, after `npm ci`:

```sh
node scripts/generate-icon.mjs
```

This uses the already-installed **@tauri-apps/cli** SVG renderer to generate
Windows `icon.ico`, macOS `icon.icns`, and the `icon.png` used for Linux,
then writes `tray.rgba` as raw 64×64 RGBA to match the native tray code.

Tauri's **before-dev / before-build** commands run the generator automatically;
this also updates the Windows NSIS installer icon because
`tauri.windows.conf.json` references `icons/icon.ico`.

The checked-in `.ico`, `.png`, and `tray.rgba` files are generated
artifacts, **not** design sources. `icon.icns` is generated too but is **not**
tracked by git (its bytes change on every run); `tauri build`/`tauri dev` and
`macos-dev-runner.mjs` get it from the before-build/before-dev command, so run
`node scripts/generate-icon.mjs` first when using it any other way. Re-run the command before any direct Cargo
build, or use the standard Tauri entry points, so the generated native icons
cannot remain from an earlier design. Do not edit them independently.

The source was updated to SVG because the previous `icon-source.png` was an old
robot-face bitmap and the prior generator hardcoded a pixel-art approximation.
