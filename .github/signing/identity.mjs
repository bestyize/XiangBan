// Build-machine setup only. Never shipped to users or run by the desktop Agent.
import { createHash, randomBytes } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { chmodSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const SIGNING_NAME = 'XiangBan Self-Signed Code Signing 2026';
export const SIGNING_FINGERPRINT = 'E8B36162292F910859DFDFD66D81D8D265764779';
const signingDir = path.dirname(fileURLToPath(import.meta.url));

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { encoding: 'utf8', timeout: 30_000, ...options });
  // Do not include argv/output: security commands can contain passwords and key attributes.
  if (result.error || result.status !== 0) throw new Error(`${path.basename(command)} failed while preparing the fixed signing identity; no ad-hoc fallback. Check the build keychain is unlocked.`);
  return result.stdout || '';
}

export function signingIdentity(env = process.env) {
  const identity = env.APPLE_SIGNING_IDENTITY?.trim() || SIGNING_NAME;
  if (identity === '-') throw new Error('Ad-hoc macOS signing is disabled. Use the persistent signing certificate.');
  return identity;
}

function preferKeychain(keychain) {
  const searchList = run('/usr/bin/security', ['list-keychains', '-d', 'user']);
  const entries = [...searchList.matchAll(/"([^"\n]+)"/g)].map(match => match[1]);
  // Private-key lookup may otherwise select an older, locked copy of the same certificate.
  if (entries[0] !== keychain) run('/usr/bin/security', ['list-keychains', '-d', 'user', '-s', keychain, ...entries.filter(entry => entry !== keychain)]);
}

export function ensureSigningIdentity() {
  const requested = signingIdentity();
  const identity = requested === SIGNING_NAME ? SIGNING_FINGERPRINT : requested;
  if (process.platform !== 'darwin') return identity;
  const local = path.join(homedir(), 'Library/Application Support/XiaoBanBuildSigning');
  const keychain = path.join(local, 'signing.keychain-db');
  const passwordFile = path.join(local, 'keychain-password');
  if (identity.toUpperCase() === SIGNING_FINGERPRINT && existsSync(keychain) && existsSync(passwordFile)) {
    const password = readFileSync(passwordFile, 'utf8').trim();
    run('/usr/bin/security', ['unlock-keychain', '-p', password, keychain]);
    process.env.XIAOBAN_SIGNING_KEYCHAIN = keychain;
    preferKeychain(keychain);
  }
  const identities = run('/usr/bin/security', ['find-identity', '-v', '-p', 'codesigning']);
  if (identities.includes(`"${identity}"`) || identities.includes(` ${identity} `)) return identity;
  if (identity.toUpperCase() !== SIGNING_FINGERPRINT) throw new Error('APPLE_SIGNING_IDENTITY is not available in the keychain; no ad-hoc fallback.');

  // A dedicated persistent build keychain; preserve all existing search-list entries.
  mkdirSync(local, { recursive: true, mode: 0o700 });
  chmodSync(local, 0o700);
  if (!existsSync(passwordFile)) writeFileSync(passwordFile, randomBytes(32).toString('hex'), { mode: 0o600 });
  chmodSync(passwordFile, 0o600);
  const keychainPassword = readFileSync(passwordFile, 'utf8').trim();
  if (!existsSync(keychain)) run('/usr/bin/security', ['create-keychain', '-p', keychainPassword, keychain]);
  run('/usr/bin/security', ['unlock-keychain', '-p', keychainPassword, keychain]);
  run('/usr/bin/security', ['set-keychain-settings', '-lut', '21600', keychain]);
  const p12 = Buffer.from(readFileSync(path.join(signingDir, 'macos-xiaoban.p12.base64'), 'utf8'), 'base64');
  if (createHash('sha256').update(p12).digest('hex') !== 'd6be6ee1d4e50bfe3d4d6a861b347120486fd97aae4d53f137aa0e4b4fdc77a5') {
    throw new Error('Persistent signing certificate archive has changed; refusing to import a different identity.');
  }
  const p12File = path.join(local, 'identity.p12');
  writeFileSync(p12File, p12, { mode: 0o600 });
  const password = readFileSync(path.join(signingDir, 'macos-xiaoban.p12.password'), 'utf8').trim();
  // Import only if this keychain does not already hold the identity.
  if (!run('/usr/bin/security', ['find-identity', '-p', 'codesigning', keychain]).includes(SIGNING_FINGERPRINT)) {
    run('/usr/bin/security', ['import', p12File, '-k', keychain, '-P', password, '-f', 'pkcs12', '-T', '/usr/bin/codesign']);
  }
  run('/usr/bin/security', ['set-key-partition-list', '-S', 'apple-tool:,apple:,codesign:', '-s', '-k', keychainPassword, keychain]);
  const cert = path.join(local, 'certificate.pem');
  const raw = run('/usr/bin/openssl', ['pkcs12', '-in', p12File, '-passin', 'stdin', '-nokeys', '-clcerts'], { input: `${password}\n` });
  writeFileSync(cert, raw, { mode: 0o600 });
  // User-domain trust for code signing only, not the system root store or TLS trust.
  run('/usr/bin/security', ['add-trusted-cert', '-r', 'trustRoot', '-p', 'codeSign', '-k', keychain, cert]);
  preferKeychain(keychain);
  if (!run('/usr/bin/security', ['find-identity', '-v', '-p', 'codesigning']).includes(SIGNING_FINGERPRINT)) {
    throw new Error('The persistent signing identity is not valid for code signing; no ad-hoc fallback.');
  }
  console.log(`Using persistent macOS signing identity: ${SIGNING_NAME}`);
  process.env.XIAOBAN_SIGNING_KEYCHAIN = keychain;
  return identity;
}

export function verifyStableSignature(target) {
  run('/usr/bin/codesign', ['--verify', '--strict', target]);
  const requirement = run('/usr/bin/codesign', ['--display', '--requirements', '-', target], { stdio: ['ignore', 'pipe', 'pipe'] });
  // codesign writes the requirement to stdout; reject version-specific requirements.
  if (!requirement.includes('designated =>') || /\bcdhash\b/.test(requirement)) {
    throw new Error(`Unstable signing requirement: ${target}`);
  }
}

export function signMacos(target, extra = []) {
  const identity = ensureSigningIdentity();
  run('/usr/bin/codesign', ['--force', '--timestamp=none', '--sign', identity, ...signingKeychainArgs(), ...extra, target]);
  verifyStableSignature(target);
}

export function signingKeychainArgs() {
  return process.env.XIAOBAN_SIGNING_KEYCHAIN ? ['--keychain', process.env.XIAOBAN_SIGNING_KEYCHAIN] : [];
}
