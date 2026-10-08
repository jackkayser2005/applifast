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
    PlaybackMode: { FULL_PLAYBACK_ONLY: 2 }, PlaybackStates: { ended: 10, playing: 2 } },
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
  await app.dispatch({ type: 'intent', generation: 7, command: { type: 'seek', seconds: 30 } });
  assert.equal(lastState().requestGeneration, 7);
  music.seekToTime = () => new Promise(() => {});
  let paused = false;
  music.pause = async () => { paused = true; };
  await Promise.race([
    app.dispatch({ type: 'intent', generation: 8, command: { type: 'seek', seconds: 60 } })
      .then(() => app.dispatch({ type: 'intent', generation: 9, command: { type: 'pause' } })),
    new Promise((_, reject) => setTimeout(() => reject(new Error('Seek blocked pause')), 250))
  ]);
  assert(paused);
  paused = false;
  emit('playbackStateDidChange');
  assert(paused); // A late seek completion must honor the user's pause.
  assert.equal(lastState().requestGeneration, 9);
  let played = false;
  music.play = async () => { played = true; };
  await app.dispatch({ type: 'previous' });
  assert(!played); // Previous keeps paused playback paused.

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
  await app.dispatch({type:'play',items:[{kind:'catalog',id:'123',playParams:{id:'456',kind:'song'}}],index:0});
  assert.equal(calls.at(-1).items[0].id,'456'); // Apple's own catalog playback ID can differ from its resource ID.

  await app.dispatch({type:'play',items:[uploaded,catalog],index:0});
  const beforeEdit = calls.length;
  const order = {upcoming:[2,3,4,1],manualCount:3,context:[0,1]};
  await app.dispatch({type:'intent',generation:10,command:{type:'queue',items:[uploaded,catalog,uploaded,catalog,uploaded],index:0,order}});
  assert.equal(calls.length,beforeEdit); // Editing the queue must not restart audio.
  assert.equal(lastState().order.manualCount,3);
  await app.dispatch({type:'next'});
  assert.equal(lastState().index,2); // A manual duplicate plays before the catalog context.
  await app.dispatch({type:'jump',position:1});
  assert.equal(lastState().index,4);
  assert.equal(lastState().order.manualCount,0);
  assert.deepEqual(Array.from(lastState().order.upcoming),[1]);
  await app.dispatch({type:'queue',items:[uploaded,catalog,uploaded],index:2,
    order:{upcoming:[1],manualCount:0,context:[0,1],history:[0]}});
  assert.equal(lastState().index,2); // Reclaimed rows can remap the current occurrence without changing audio.
  await app.dispatch({type:'next'});
  assert.equal(calls.at(-1).items[0].id,'123');
  assert.equal(lastState().order.manualCount,0);
  const validOrder = JSON.stringify(lastState().order);
  await app.dispatch({type:'queue',items:[uploaded,catalog,uploaded],index:1,
    order:{upcoming:[2,2],manualCount:2,context:[0,1]}});
  assert.equal(JSON.stringify(lastState().order),validOrder);
  const endedCalls = calls.length;
  music.playbackState = 10; emit('playbackStateDidChange');
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(calls.length,endedCalls); // Native orders advance only after the native coordinator chooses the occurrence.
  await app.dispatch({type:'intent',generation:11,command:{type:'select',index:2,playing:true,
    order:{upcoming:[],manualCount:0,context:[0,1],history:[0,1]}}});
  assert.equal(lastState().index,2);
  assert.equal(calls.at(-1).items[0].id,'i.upload');
  music.playbackState = 2;

  // Restoration retains occurrences, uses library parameters and never plays.
  played = false;
  music.playbackState = 3;
  const restoredOrder = { upcoming: [2, 1], manualCount: 1, context: [0, 1], history: [] };
  await app.dispatch({ type: 'intent', generation: 12, command: {
    type: 'restore', items: [uploaded, catalog, uploaded], index: 0,
    order: restoredOrder, seconds: 42, shuffle: false, repeat: 0
  } });
  assert(!played);
  assert.equal(lastState().position, 42);
  assert.equal(lastState().requestGeneration, 12);
  assert.equal(calls.at(-1).items[0].isLibrary, true);
  assert.deepEqual(Array.from(lastState().order.upcoming), [2, 1]);
  await app.dispatch({ type: 'next' });
  assert(!played);
  assert.equal(lastState().index, 2);

  let resolveRead;
  music.api.music = () => new Promise(resolve => {resolveRead=resolve;});
  const pendingRead=app.dispatch({type:'request',id:42,path:'/v1/me/library/albums?limit=100'});
  while (!resolveRead) await new Promise(resolve => setImmediate(resolve));
  await Promise.race([app.dispatch({type:'pause'}),new Promise((_,reject)=>setTimeout(()=>reject(new Error('Page read blocked pause')),250))]);
  resolveRead({data:{data:[]}});
  await pendingRead;
  assert.equal(messages.filter(event=>event.type==='response').at(-1).id,42);

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
