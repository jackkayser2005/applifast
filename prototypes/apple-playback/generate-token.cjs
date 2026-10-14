// Local MusicKit setup. Never prints the signing key or developer token.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

function sign(key, keyId, teamId, now) {
  if (key.asymmetricKeyType !== 'ec' || key.asymmetricKeyDetails.namedCurve !== 'prime256v1') {
    throw new Error('Use a MusicKit P-256 .p8 private key.');
  }
  const encode = value => Buffer.from(JSON.stringify(value)).toString('base64url');
  const header = encode({ alg: 'ES256', kid: keyId });
  const payload = encode({ iss: teamId, iat: now, exp: now + 30 * 86400,
    origin: ['https://applifast.invalid'] });
  const input = `${header}.${payload}`;
  const signature = crypto.sign('sha256', Buffer.from(input), {
    key, dsaEncoding: 'ieee-p1363'
  }).toString('base64url');
  return `${input}.${signature}`;
}

function main(args) {
  if (args.length === 1 && args[0] === '--self-test') {
    const assert = require('node:assert/strict');
    const { privateKey, publicKey } = crypto.generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
    const token = sign(privateKey, 'TESTKEY123', 'TESTTEAM12', 100);
    const [header, payload, signature] = token.split('.');
    assert.equal(JSON.parse(Buffer.from(header, 'base64url')).alg, 'ES256');
    const claims = JSON.parse(Buffer.from(payload, 'base64url'));
    assert.equal(claims.exp - claims.iat, 30 * 86400);
    assert.deepEqual(claims.origin, ['https://applifast.invalid']);
    assert.equal(Buffer.from(signature, 'base64url').length, 64);
    assert(crypto.verify('sha256', Buffer.from(`${header}.${payload}`), {
      key: publicKey, dsaEncoding: 'ieee-p1363'
    }, Buffer.from(signature, 'base64url')));
    console.log('Token generator self-test passed (temporary in-memory key only).');
    return;
  }
  const options = {};
  for (let index = 0; index < args.length; index += 2) {
    const flag = args[index];
    if (!['--key-file', '--key-id', '--team-id'].includes(flag) || !args[index + 1] || options[flag]) {
      throw new Error('Usage: node generate-token.cjs --key-file PATH --key-id KEYID --team-id TEAMID');
    }
    options[flag] = args[index + 1];
  }
  for (const flag of ['--key-id', '--team-id']) {
    if (!/^[A-Z0-9]{10}$/.test(options[flag] || '')) {
      throw new Error('Key ID and Team ID must each have 10 uppercase letters or digits.');
    }
  }
  if (!options['--key-file']) throw new Error('Provide the path to your downloaded MusicKit .p8 file.');
  const keyPath = path.resolve(options['--key-file']);
  if (fs.statSync(keyPath).size > 16384) throw new Error('Signing key file is unexpectedly large.');
  let key;
  try { key = crypto.createPrivateKey(fs.readFileSync(keyPath)); }
  catch { throw new Error('Cannot read a valid .p8 private key from that file.'); }
  const token = sign(key, options['--key-id'], options['--team-id'], Math.floor(Date.now() / 1000));
  const directory = path.resolve(__dirname, '../../.secrets/apple-music');
  fs.mkdirSync(directory, { recursive: true });
  const destination = path.join(directory, 'developer-token.txt');
  const temporary = path.join(directory, `.developer-token-${crypto.randomUUID()}.tmp`);
  try {
    fs.writeFileSync(temporary, `${token}\n`, { flag: 'wx', mode: 0o600 });
    fs.renameSync(temporary, destination);
  } finally {
    if (fs.existsSync(temporary)) fs.unlinkSync(temporary);
  }
  console.log(`Saved a 30-day developer token to ${destination}`);
}

try { main(process.argv.slice(2)); }
catch (error) {
  // File-system errors include paths but not file contents. No signing inputs are logged.
  console.error(error instanceof Error ? error.message : 'Token generation failed.');
  process.exitCode = 1;
}
