// Static release-contract checks. Run before the Windows NSIS build.
// These checks do not replace a real Windows installer compile or installation smoke test.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const source = (relative) => readFileSync(fileURLToPath(new URL(relative, import.meta.url)), 'utf8');
const config = JSON.parse(source('../src-tauri/tauri.windows.conf.json'));
const nsis = source('../src-tauri/windows/one-click-installer.nsi');
const workflow = source('../../../.github/workflows/build-xiaoban-wallpaper.yml');

assert.equal(config.bundle.active, true);
assert.deepEqual(config.bundle.targets, ['nsis']);
assert.equal(config.bundle.windows.nsis.installMode, 'currentUser');
assert.equal(config.bundle.windows.nsis.displayLanguageSelector, false);
assert.equal(config.bundle.windows.nsis.template, 'windows/one-click-installer.nsi');

assert.match(nsis, /RequestExecutionLevel user/);
assert.match(nsis, /MUI_WELCOMEPAGE_TITLE "欢迎使用小伴桌面"/);
assert.match(nsis, /STR:一键安装/);
assert.match(nsis, /!insertmacro MUI_PAGE_WELCOME/);
assert.doesNotMatch(nsis, /!insertmacro MUI_PAGE_DIRECTORY/);
assert.doesNotMatch(nsis, /!insertmacro MUI_PAGE_FINISH/);
assert.match(nsis, /ShowInstDetails nevershow/);
assert.match(nsis, /SetAutoClose true/);
assert.match(nsis, /Function \.onInstSuccess[\s\S]*Call RunMainBinary[\s\S]*FunctionEnd/);
assert.match(nsis, /!insertmacro MUI_UNPAGE_CONFIRM/);
assert.match(nsis, /!insertmacro MUI_STARTMENU_WRITE_BEGIN/);
const installBody = nsis.split('Section Install')[1]?.split('SectionEnd')[0];
assert.ok(installBody, 'the official install section must remain');
assert.doesNotMatch(installBody, /Call CreateOrUpdateDesktopShortcut/);
assert.match(workflow, /npm run desktop:windows:build/);
assert.match(workflow, /bundle\/nsis\/\*-setup\.exe/);
assert.doesNotMatch(workflow, /Copy-Item src-tauri\/target\/release\/xiaoban-desktop-wallpaper\.exe/);
console.log('One-click Windows NSIS release contract OK');
