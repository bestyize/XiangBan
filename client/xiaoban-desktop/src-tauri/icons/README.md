# Xiaoban Desktop application icon

The single editable default source is **`icon-source.svg`**: **Twin Stars（双星相伴 · 精修 A）** in pure white (`#FFFFFF`) over the **Sunset Coral glaze** (`#FF6584` → `#F9744B`). The companion dark-mode source is **`icon-source-dark.svg`** (pure white over **Obsidian** `#262023` → `#121012`). Both share the same optical centering and cross-platform brand identity with 小伴社区 and 小伴手机 App.

## Regenerate native assets

From `client/xiaoban-desktop`, after `npm ci`:

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
