// These test our boundary only. They do not establish real MusicKit playback.
const assert = require('node:assert/strict');
const vm = require('node:vm');
const fs = require('node:fs');
const path = require('node:path');
const messages = [], calls = [], listeners = new Map();
let libraryReply = { data: { data: [], next: null } }, authorization;
const music = {
  playbackState: 2, currentPlaybackTime: 0, currentPlaybackDuration: 240,
  isAuthorized: true, storefrontId: 'us', queue: { length: 1 },
  addEventListener: (event, handler) => {
    if (!listeners.has(event)) listeners.set(event, []);
    listeners.get(event).push(handler);
  },
  api: { music: async () => libraryReply },
  authorize: () => new Promise(resolve => { authorization = resolve; }),
  unauthorize: async () => {},
  setQueue: async options => {
    calls.push(options);
    if (options.items[0].id === 'i.unavailable') throw new Error('secret SDK response');
  },
  play: async () => {}, pause: async () => {}, seekToTime: async () => {}
};
const eventNames = ['playbackTimeDidChange', 'playbackStateDidChange', 'nowPlayingItemDidChange', 'mediaPlaybackError'];
const sandbox = {
  window: { MusicKit: { configure: async () => music, version: 'test',
    Events: Object.fromEntries(eventNames.map(name => [name, name])),
    PlaybackMode: { FULL_PLAYBACK_ONLY: 2 }, PlaybackStates: { ended: 10 } },
    chrome: { webview: { postMessage: event => messages.push(event) } }, isSecureContext: true },
  document: { addEventListener: () => {}, dispatchEvent: () => {} },
  CustomEvent: class { constructor(name, properties) { Object.assign(this, properties); } },
  structuredClone
};
vm.runInNewContext(fs.readFileSync(path.join(__dirname, 'bridge.js'), 'utf8'), sandbox);
const app = sandbox.window.applifast;
const emit = name => (listeners.get(name) || []).forEach(handler => handler());
const lastState = () => messages.filter(event => event.type === 'state').at(-1);

(async () => {
  await app.bootstrap({ developerToken: 'mock-only', session: 1 });
  assert.equal(music.playbackMode, 2);
  const uploaded = { kind: 'library', id: 'i.upload', playParams: { id: 'i.upload', kind: 'song', isLibrary: true } };
  const catalog = { kind: 'catalog', id: '123', playParams: null };
  await app.dispatch({ type: 'play', items: [uploaded, catalog, uploaded], index: 0 });
  await app.dispatch({ type: 'next' });
  await app.dispatch({ type: 'next' });
  assert.deepEqual(calls.map(call => call.items[0].id), ['i.upload', '123', 'i.upload']);
  assert(calls.every(call => call.items.length === 1));
  assert.equal(calls[0].items[0].isLibrary, true);
  assert.equal(lastState().queueLength, 3);
  assert.equal(lastState().index, 2);

  libraryReply = { data: { data: [{ id: 'i.upload', attributes: { name: 'Upload' } }],
    next: '/v1/me/library/songs?offset=100' } };
  await app.dispatch({ type: 'library', next: null });
  const page = messages.find(event => event.type === 'library');
  assert.equal(page.items[0].id, 'i.upload');
  assert.equal(page.items[0].playParams, null); // Unavailable metadata stays visible.
  const count = messages.filter(event => event.type === 'library').length;
  libraryReply.data.next = 'https://untrusted.example/';
  await app.dispatch({ type: 'library', next: null });
  assert.equal(messages.filter(event => event.type === 'library').length, count);

  await app.dispatch({ type: 'play', items: [uploaded,
    { kind: 'library', id: 'i.unavailable', playParams: null }, catalog], index: 0 });
  await app.dispatch({ type: 'next' });
  assert.equal(lastState().index, 1);
  assert.equal(lastState().queueLength, 3);
  const failedCalls = calls.length;
  music.playbackState = 10; emit('playbackStateDidChange');
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(calls.length, failedCalls); // Failure cannot silently skip to catalog.
  assert(!JSON.stringify(messages).includes('secret SDK response'));

  const pendingAuthorization = app.dispatch({ type: 'authorize' });
  while (!authorization) await new Promise(resolve => setImmediate(resolve));
  const signOut = app.dispatch({ type: 'signOut' });
  authorization('late-user-token');
  await pendingAuthorization; await signOut;
  assert(!messages.some(event => event.type === 'authorized'));
  assert.equal(messages.at(-1).type, 'signedOut');
  assert.equal(messages.at(-1).session, 2);
  console.log('Bridge self-check passed: uploaded IDs, occurrences, pagination, failures, stale authorization.');
})().catch(failure => { console.error(failure); process.exitCode = 1; });
