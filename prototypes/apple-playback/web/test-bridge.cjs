// These test our boundary only. They do not establish real MusicKit playback.
const assert = require('node:assert/strict');
const vm = require('node:vm');
const fs = require('node:fs');
const path = require('node:path');
const messages = [], calls = [], listeners = new Map();
const readTimers = new Map();
let timerId = 0;
let libraryReply = { data: { data: [], next: null } }, authorization;
const requests = [];
const music = {
  playbackState: 2, currentPlaybackTime: 0, currentPlaybackDuration: 240,
  isAuthorized: true, storefrontId: 'us', queue: { length: 1 },
  addEventListener: (event, handler) => {
    if (!listeners.has(event)) listeners.set(event, []);
    listeners.get(event).push(handler);
  },
  api: { music: async (...args) => { requests.push(args); return libraryReply; } },
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
    PlaybackMode: { FULL_PLAYBACK_ONLY: 2 }, PlaybackStates: {
      none: 0, loading: 1, playing: 2, paused: 3, stopped: 4,
      ended: 5, seeking: 6, waiting: 8, stalled: 9, completed: 10
    } },
    chrome: { webview: { postMessage: event => messages.push(event) } }, isSecureContext: true },
  document: { addEventListener: () => {}, dispatchEvent: () => {} },
  CustomEvent: class { constructor(name, properties) { Object.assign(this, properties); } },
  structuredClone, TextEncoder, AbortController, URLSearchParams,
  setTimeout: (callback, delay) => { assert.equal(delay, 20_000); readTimers.set(++timerId, callback); return timerId; },
  clearTimeout: id => readTimers.delete(id)
};
vm.runInNewContext(fs.readFileSync(path.join(__dirname, 'bridge.js'), 'utf8'), sandbox);
const app = sandbox.window.applifast;
const emit = name => (listeners.get(name) || []).forEach(handler => handler());
const lastState = () => messages.filter(event => event.type === 'state').at(-1);

async function checkSdkStartup() {
  for (const outcome of ['timeout', 'loaded', 'preloaded', 'configuration-failed']) {
    const events = [], timers = new Map(), handlers = new Map();
    let configured = 0;
    const sdk = { ...sandbox.window.MusicKit, configure: async () => {
      configured++;
      if (outcome === 'configuration-failed') throw new Error('secret SDK failure');
      return { ...music, addEventListener: () => {} };
    } };
    const context = { ...sandbox,
      window: { MusicKit: ['preloaded', 'configuration-failed'].includes(outcome) ? sdk : undefined,
        chrome: { webview: { postMessage: event => events.push(event) } } },
      document: {
        addEventListener: (name, handler) => handlers.set(name, handler),
        removeEventListener: (name, handler) => { assert.equal(handlers.get(name), handler); handlers.delete(name); },
        dispatchEvent: () => {}
      },
      setTimeout: (callback, delay) => { assert.equal(delay, 45_000); timers.set(1, callback); return 1; },
      clearTimeout: id => timers.delete(id)
    };
    vm.runInNewContext(fs.readFileSync(path.join(__dirname, 'bridge.js'), 'utf8'), context);
    const starting = context.window.applifast.bootstrap({ developerToken: 'mock-only', session: 17 });
    assert.equal(context.window.applifast.bootstrap({ developerToken: 'mock-only', session: 17 }), starting);
    if (outcome === 'timeout') {
      assert.equal(timers.size, 1, 'A missing SDK must have a startup deadline');
      const rejected = assert.rejects(starting);
      timers.get(1)();
      await rejected;
      context.window.MusicKit = sdk;
      handlers.get('musickitloaded')?.();
      await Promise.resolve();
      assert.equal(configured, 0, 'A late SDK cannot revive failed startup');
    } else if (outcome === 'configuration-failed') {
      await assert.rejects(starting);
    } else {
      if (outcome === 'loaded') {
        assert.equal(timers.size, 1);
        context.window.MusicKit = sdk;
        handlers.get('musickitloaded')();
      }
      await starting;
      assert.equal(configured, 1);
    }
    assert.equal(timers.size, 0);
    assert.equal(handlers.size, 0, 'Startup must release its SDK listener');
    const failed = ['timeout', 'configuration-failed'].includes(outcome);
    assert.equal(events.length, 1);
    assert.equal(events[0].session, 17);
    assert.equal(events[0].type, failed ? 'error' : 'ready');
    if (failed) assert.equal(events[0].code, 'configuration');
    assert(!JSON.stringify(events).includes('secret SDK failure'));
  }
}

async function checkAuthorizationCancellation() {
  const events = [], attempts = [];
  const localMusic = { ...music, addEventListener: () => {}, musicUserToken: '',
    authorize: () => new Promise(resolve => attempts.push(token => {
      localMusic.musicUserToken = token;
      resolve(token);
    })),
    unauthorize: async () => { localMusic.musicUserToken = ''; }
  };
  const context = { ...sandbox, window: { ...sandbox.window,
    MusicKit: { ...sandbox.window.MusicKit, configure: async () => localMusic },
    chrome: { webview: { postMessage: event => events.push(event) } }
  } };
  vm.runInNewContext(fs.readFileSync(path.join(__dirname, 'bridge.js'), 'utf8'), context);
  const localApp = context.window.applifast;
  await localApp.bootstrap({ developerToken: 'mock-only', session: 7 });
  const pending = localApp.dispatch({ type: 'authorize' });
  while (attempts.length < 1) await new Promise(resolve => setImmediate(resolve));
  await Promise.race([localApp.dispatch({ type: 'signOut' }),
    new Promise((_, reject) => setTimeout(() => reject(new Error('Unresolved authorization blocked sign-out')), 250))]);
  await pending;
  assert.equal(events.at(-1).type, 'signedOut');
  assert.equal(events.at(-1).session, 8);
  const retry = localApp.dispatch({ type: 'authorize' });
  while (attempts.length < 2) await new Promise(resolve => setImmediate(resolve));
  attempts[0]('dummy-stale-user-token');
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(localMusic.musicUserToken, '', 'A canceled SDK completion must restore the current grant');
  assert.equal(attempts.length, 2, 'A stale completion must not clear the newer attempt guard');
  attempts[1]('dummy-current-user-token');
  await retry;
  assert.equal(localMusic.musicUserToken, 'dummy-current-user-token');
  assert.equal(events.filter(event => event.type === 'authorized').length, 1);

  const closing = localApp.dispatch({ type: 'authorize' });
  while (attempts.length < 3) await new Promise(resolve => setImmediate(resolve));
  localApp.cancelAuthorization();
  await Promise.race([closing,
    new Promise((_, reject) => setTimeout(() => reject(new Error('Popup close left authorization pending')), 250))]);
  const reopened = localApp.dispatch({ type: 'authorize' });
  while (attempts.length < 4) await new Promise(resolve => setImmediate(resolve));
  attempts[3]('dummy-newer-user-token');
  await reopened;
  attempts[2]('dummy-late-popup-token');
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(localMusic.musicUserToken, 'dummy-newer-user-token');
  const authorized = events.filter(event => event.type === 'authorized');
  assert.equal(authorized.length, 2);
  assert(authorized.every(event => event.session === 8));
  assert(!JSON.stringify(events).includes('dummy-stale-user-token'));
  assert(!JSON.stringify(events).includes('dummy-late-popup-token'));
  const racing = localApp.dispatch({ type: 'authorize' });
  while (attempts.length < 5) await new Promise(resolve => setImmediate(resolve));
  attempts[4]('dummy-racing-user-token');
  await Promise.resolve();
  localApp.cancelAuthorization();
  await racing;
  assert.equal(localMusic.musicUserToken, 'dummy-newer-user-token');
  assert.equal(events.filter(event => event.type === 'authorized').length, 2,
    'Cancellation between SDK resolution and token publication must win');
  assert(!JSON.stringify(events).includes('dummy-racing-user-token'));
  let configure;
  context.window.MusicKit.configure = () => new Promise(resolve => { configure = resolve; });
  vm.runInNewContext(fs.readFileSync(path.join(__dirname, 'bridge.js'), 'utf8'), context);
  const startingApp = context.window.applifast;
  const starting = startingApp.bootstrap({ developerToken: 'mock-only',
    userToken: 'dummy-restored-user-token', session: 20 });
  const earlySignOut = startingApp.dispatch({ type: 'signOut' });
  while (!configure) await new Promise(resolve => setImmediate(resolve));
  configure(localMusic);
  await starting;
  await earlySignOut;
  const canceled = startingApp.dispatch({ type: 'authorize' });
  while (attempts.length < 6) await new Promise(resolve => setImmediate(resolve));
  startingApp.cancelAuthorization();
  await canceled;
  attempts[5]('dummy-late-startup-token');
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(localMusic.musicUserToken, '', 'Sign-out during startup must clear the accepted grant too');
}

(async () => {
  await checkSdkStartup();
  await checkAuthorizationCancellation();
  await app.bootstrap({ developerToken: 'mock-only', session: 1 });
  assert.equal(music.playbackMode, 2);
  const uploaded = { kind: 'library', id: 'i.upload', playParams: { id: 'i.upload', kind: 'song', isLibrary: true } };
  const catalog = { kind: 'catalog', id: '123', playParams: null };
  libraryReply = { data: { data: [{ id: 'p.created', type: 'library-playlists', attributes: { canEdit: true } }] } };
  await app.dispatch({ type: 'createPlaylist', id: 101, name: ' Queue ', public: false,
    items: [uploaded, { ...catalog, playParams: { id: '456', kind: 'song' } }, uploaded] });
  const created = requests.at(-1);
  assert.equal(created[0], '/v1/me/library/playlists');
  assert.equal(created[2].fetchOptions.method, 'POST');
  assert.deepEqual(JSON.parse(created[2].fetchOptions.body), {
    attributes: { name: 'Queue', isPublic: false }, relationships: { tracks: { data: [
      { id: 'i.upload', type: 'library-songs' }, { id: '123', type: 'songs' }, { id: 'i.upload', type: 'library-songs' }
    ] } }
  });
  assert.equal(messages.find(event => event.id === 101).data.data[0].id, 'p.created');
  libraryReply = undefined; // A successful append has no response body.
  await app.dispatch({ type: 'appendPlaylist', id: 102, playlist: 'p.created', items: [uploaded, uploaded] });
  assert.equal(requests.at(-1)[0], '/v1/me/library/playlists/p.created/tracks');
  assert.equal(JSON.parse(requests.at(-1)[2].fetchOptions.body).data.length, 2);
  assert.equal(messages.find(event => event.id === 102).data, null);
  const initialApi = music.api.music;
  music.api.music = async (...args) => {
    requests.push(args);
    if (args[2].fetchOptions.method === 'POST') return undefined;
    const library = args[0] === '/v1/me/library/songs';
    return { data: { data: [{ id: library ? 'i.upload' : '123',
      type: library ? 'library-songs' : 'songs', attributes: { inFavorites: true } }] } };
  };
  const favoriteStart = requests.length;
  await app.dispatch({ type: 'favoriteSongs', id: 2010,
    items: [uploaded, { ...catalog, playParams: { id: '456', kind: 'song' } }, uploaded] });
  const favorite = requests[favoriteStart];
  assert.equal(favorite[0], '/v1/me/favorites');
  assert.equal(favorite[2].fetchOptions.method, 'POST');
  assert.equal(favorite[2].fetchOptions.body, undefined);
  assert.deepEqual(JSON.parse(JSON.stringify(favorite[1])), {
    'ids[songs]': '123', 'ids[library-songs]': 'i.upload'
  });
  assert.equal(requests.length - favoriteStart, 3, 'One write and one read per resource type');
  assert.equal(requests[favoriteStart + 1][0], '/v1/catalog/us/songs');
  assert.equal(requests[favoriteStart + 2][0], '/v1/me/library/songs');
  assert.deepEqual(JSON.parse(JSON.stringify(requests[favoriteStart + 1][1])), { ids: '123', extend: 'inFavorites' });
  assert.deepEqual(JSON.parse(JSON.stringify(requests[favoriteStart + 2][1])), { ids: 'i.upload', extend: 'inFavorites' });
  assert.equal(requests[favoriteStart + 1][2].fetchOptions.cache, 'no-store');
  assert.equal(requests[favoriteStart + 2][2].fetchOptions.cache, 'no-store');
  const confirmed = messages.find(event => event.id === 2010);
  assert.equal(confirmed.error, undefined);
  assert.deepEqual(JSON.parse(JSON.stringify(confirmed.data)), { data: [
    { id: '123', type: 'songs', attributes: { inFavorites: true } },
    { id: 'i.upload', type: 'library-songs', attributes: { inFavorites: true } }
  ] });
  // Missing, malformed, substituted and ambiguous results cannot confirm a favorite.
  for (const [n, rows] of [
    [], [{ id: '123', type: 'songs', attributes: { inFavorites: false } }],
    [{ id: '123', type: 'songs', attributes: { inFavorites: 'true' } }],
    [{ id: '456', type: 'songs', attributes: { inFavorites: true } }],
    [{ id: '123', type: 'library-songs', attributes: { inFavorites: true } }],
    Array(2).fill({ id: '123', type: 'songs', attributes: { inFavorites: true } })
  ].entries()) {
    music.api.music = async (_path, _parameters, options) => options.fetchOptions.method === 'POST'
      ? undefined : { data: { data: rows } };
    await app.dispatch({ type: 'favoriteSongs', id: 2100 + n, items: [catalog] });
    const result = messages.find(event => event.id === 2100 + n);
    assert(result.error.includes('Refresh'));
    assert.equal(result.data.data[0].id, '123');
    assert.equal(result.data.data[0].attributes.inFavorites, n === 1 ? false : undefined);
  }
  music.api.music = async (path, _parameters, options) => {
    if (options.fetchOptions.method === 'POST') return undefined;
    if (path === '/v1/me/library/songs') throw new Error('secret confirmation failure');
    return { data: { data: [{ id: '123', type: 'songs', attributes: { inFavorites: true, token: 'secret extra' } }] } };
  };
  await app.dispatch({ type: 'favoriteSongs', id: 2110, items: [catalog, uploaded] });
  const partial = messages.find(event => event.id === 2110);
  assert(partial.error);
  assert.equal(partial.data.data[0].attributes.inFavorites, true);
  assert.equal(partial.data.data[1].attributes.inFavorites, undefined);
  assert(!JSON.stringify(partial).includes('secret'));
  let confirmationSignal, confirmationPosts = 0;
  music.api.music = (_path, _parameters, options) => {
    if (options.fetchOptions.method === 'POST') { confirmationPosts++; return Promise.resolve(); }
    confirmationSignal = options.fetchOptions.signal;
    return new Promise((_resolve, reject) => confirmationSignal.addEventListener('abort',
      () => reject(new Error('secret confirmation timeout')), { once: true }));
  };
  const timedConfirmation = app.dispatch({ type: 'favoriteSongs', id: 2112, items: [catalog] });
  while (!confirmationSignal) await new Promise(resolve => setImmediate(resolve));
  let confirmationPaused = false;
  const confirmationPause = music.pause;
  music.pause = async () => { confirmationPaused = true; };
  await app.dispatch({ type: 'pause' });
  assert(confirmationPaused, 'Favorite confirmation must not block transport');
  assert.equal(readTimers.size, 1);
  [...readTimers.values()][0]();
  await timedConfirmation;
  assert(confirmationSignal.aborted);
  assert.equal(confirmationPosts, 1, 'A failed confirmation must never repeat the mutation');
  assert(messages.find(event => event.id === 2112).error);
  assert.equal(readTimers.size, 0);
  music.pause = confirmationPause;
  music.api.music = initialApi;
  const storefront = music.storefrontId;
  music.storefrontId = '../secret';
  const storefrontRequests = requests.length;
  await app.dispatch({ type: 'favoriteSongs', id: 2111, items: [catalog, uploaded] });
  assert.equal(requests.length, storefrontRequests, 'Invalid storefront must fail before the write');
  assert(messages.find(event => event.id === 2111).error);
  music.storefrontId = storefront;
  for (const [index, patch] of [
    { items: [] }, { items: Array(101).fill(catalog) }, { id: Number.MAX_SAFE_INTEGER + 1 },
    { items: [{ ...uploaded, id: 'i.x?token=secret' }] },
    { items: [{ ...uploaded, playParams: { id: '123', isLibrary: true } }] }
  ].entries()) {
    const count = requests.length;
    const id = 2011 + index;
    await app.dispatch({ type: 'favoriteSongs', id, items: [catalog], ...patch });
    assert.equal(requests.length, count, 'Invalid favorites must not contact Apple');
    assert(messages.find(event => event.id === (patch.id ?? id)).error);
  }
  const beforeInvalid = requests.length;
  await app.dispatch({ type: 'appendPlaylist', id: 103, playlist: 'p.created/../../', items: [uploaded] });
  assert.equal(requests.length, beforeInvalid);
  assert(messages.find(event => event.id === 103).error);

  // A pending write must not stall play/pause. A rejection must not expose SDK secrets.
  const api = music.api.music;
  let rejectWrite;
  music.api.music = () => new Promise((_, reject) => { rejectWrite = reject; });
  const write = app.dispatch({ type: 'appendPlaylist', id: 104, playlist: 'p.created', items: [uploaded] });
  await new Promise(resolve => setImmediate(resolve));
  let immediatePause = false;
  const pause = music.pause;
  music.pause = async () => { immediatePause = true; };
  await app.dispatch({ type: 'pause' });
  assert(immediatePause);
  rejectWrite(new Error('secret SDK write response'));
  await write;
  assert(messages.find(event => event.id === 104).error);
  assert(!JSON.stringify(messages).includes('secret SDK write response'));
  music.api.music = api;
  music.pause = pause;
  let rejectFavorite;
  music.api.music = () => new Promise((_, reject) => { rejectFavorite = reject; });
  const favoriteWrite = app.dispatch({ type: 'favoriteSongs', id: 2020, items: [uploaded] });
  while (!rejectFavorite) await new Promise(resolve => setImmediate(resolve));
  let favoritePause = false;
  music.pause = async () => { favoritePause = true; };
  await app.dispatch({ type: 'pause' });
  assert(favoritePause, 'A favorites write must not block transport');
  rejectFavorite(new Error('secret SDK favorites response'));
  await favoriteWrite;
  assert(messages.find(event => event.id === 2020).error.includes('favorites'));
  assert(!JSON.stringify(messages).includes('secret SDK favorites response'));
  music.api.music = api;
  music.pause = pause;
  let resolveLibrary;
  music.api.music = () => new Promise(resolve => { resolveLibrary = resolve; });
  const slowLibrary = app.dispatch({ type: 'library', id: 107, next: null });
  let libraryPause = false;
  music.pause = async () => { libraryPause = true; };
  await Promise.race([
    app.dispatch({ type: 'pause' }),
    new Promise((_, reject) => setTimeout(() => reject(new Error('Library blocked pause')), 250))
  ]);
  assert(libraryPause);
  resolveLibrary({ data: { data: [], next: null } });
  await slowLibrary;
  assert.equal(messages.find(event => event.type === 'library' && event.id === 107).items.length, 0);
  music.api.music = async () => { throw new Error('secret SDK library response'); };
  await app.dispatch({ type: 'library', id: 108, next: null });
  assert(messages.find(event => event.type === 'library' && event.id === 108).error);
  assert(!JSON.stringify(messages).includes('secret SDK library response'));
  music.api.music = (_path, _parameters, { fetchOptions }) => new Promise((_resolve, reject) => {
    fetchOptions.signal.addEventListener('abort', () => reject(new Error('secret timeout response')), { once: true });
  });
  const timedOut = app.dispatch({ type: 'library', id: 109, next: null });
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(readTimers.size, 1);
  [...readTimers.values()][0]();
  await timedOut;
  assert(messages.find(event => event.type === 'library' && event.id === 109).error);
  assert.equal(readTimers.size, 0);
  assert(!JSON.stringify(messages).includes('secret timeout response'));
  music.api.music = api;
  music.pause = pause;
  await app.dispatch({ type: 'play', items: [uploaded, catalog, uploaded], index: 0 });
  for (const [sdk, native] of [[0, 0], [1, 1], [2, 2], [3, 3], [4, 0],
    [5, 10], [6, 6], [8, 8], [9, 8], [10, 10]]) {
    music.playbackState = sdk;
    emit('playbackTimeDidChange');
    assert.equal(lastState().status, native, `SDK state ${sdk} must match the native contract`);
  }
  music.playbackState = 2;
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

  libraryReply = { data: { data: [
    { id: 'i.upload', attributes: { name: 'Upload', inFavorites: true } },
    { id: 'i.other', attributes: { name: 'Other', inFavorites: false } },
    { id: 'i.unknown', attributes: { name: 'Unknown' } },
    { id: 'i.invalid', attributes: { name: 'Invalid', inFavorites: 'true' } }],
    next: '/v1/me/library/songs?offset=100' } };
  await app.dispatch({ type: 'library', next: null });
  const firstLibraryURL = new URL(requests.at(-1)[0], 'https://api.music.apple.com');
  assert.equal(firstLibraryURL.pathname, '/v1/me/library/songs');
  assert.equal(firstLibraryURL.searchParams.get('include'), 'albums,artists');
  assert.equal(firstLibraryURL.searchParams.get('extend'), 'inFavorites');
  const page = messages.filter(event => event.type === 'library').at(-1);
  assert.equal(page.items[0].id, 'i.upload');
  assert.equal(page.items[0].playParams, null); // Unavailable metadata stays visible.
  assert.deepEqual(page.items.map(item => item.inFavorites), [true, false, null, null]);
  await app.dispatch({type:'library',next:page.next});
  const nextLibraryURL = new URL(requests.at(-1)[0], 'https://api.music.apple.com');
  assert.equal(nextLibraryURL.searchParams.get('offset'), '100');
  assert.equal(nextLibraryURL.searchParams.get('include'), 'albums,artists');
  assert.deepEqual(nextLibraryURL.searchParams.getAll('extend'), ['inFavorites']);
  await app.dispatch({type:'request',id:201,path:'/v1/me/library/songs/i.upload?extend=inFavorites'});
  const detailURL = new URL(requests.at(-1)[0], 'https://api.music.apple.com');
  assert.deepEqual(detailURL.searchParams.getAll('extend'), ['inFavorites']);
  await app.dispatch({type:'request',id:202,path:'/v1/catalog/us/search?term=test&types=songs'});
  const searchURL = new URL(requests.at(-1)[0], 'https://api.music.apple.com');
  assert.equal(searchURL.searchParams.get('term'), 'test');
  assert.equal(searchURL.searchParams.get('types'), 'songs');
  libraryReply = {data: {data: [
    {id: 'i.upload', type: 'library-songs', attributes: {inFavorites: true, playParams: uploaded.playParams}},
    {id: '123', type: 'songs', attributes: {inFavorites: false, playParams: {id: '123', kind: 'song'}}},
    {id: '456', type: 'songs', attributes: {name: 'Unknown favorite status'}}
  ]}};
  for (const route of [
    '/v1/catalog/us/search?term=test&types=songs',
    '/v1/me/library/search?term=test&types=library-songs',
    '/v1/me/library/playlists/p.test/tracks?offset=100&limit=100',
    '/v1/catalog/us/playlists/pl.test?include=tracks',
    '/v1/me/library/albums/l.test/tracks?limit=100',
    '/v1/catalog/us/albums/123?include=tracks',
    '/v1/catalog/us/artists/123/view/top-songs?limit=20',
    '/v1/me/history/heavy-rotation?limit=10',
    '/v1/me/recommendations?limit=10',
    '/v1/me/library/songs?offset=100&extend%5Blibrary-songs%5D=inFavorites'
  ]) {
    await app.dispatch({type:'request',id:204,path:route});
    const extended = new URL(requests.at(-1)[0], 'https://api.music.apple.com');
    const original = new URL(route, 'https://api.music.apple.com');
    assert.equal(extended.pathname, original.pathname);
    for (const [key, value] of original.searchParams) assert.equal(extended.searchParams.get(key), value);
    assert.deepEqual(extended.searchParams.getAll('extend[songs]'), ['inFavorites']);
    assert.deepEqual(extended.searchParams.getAll('extend[library-songs]'), ['inFavorites']);
    if (!route.startsWith('/v1/me/library/songs')) {
      assert.equal(extended.searchParams.get('include'), original.searchParams.get('include'));
      assert.equal(extended.searchParams.get('extend'), null);
    }
    assert.deepEqual(JSON.parse(JSON.stringify(messages.filter(event => event.id === 204).at(-1).data)), libraryReply.data);
  }
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
  music.playbackState = 5; emit('playbackStateDidChange');
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
  music.playbackState = 5; emit('playbackStateDidChange');
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
  const restoreQueue = music.setQueue, restoreSeek = music.seekToTime;
  const restorePlay = music.play, restorePause = music.pause;
  let restoredSeekTarget;
  music.setQueue = async options => {
    await restoreQueue(options);
    music.currentPlaybackTime = 0;
    music.currentPlaybackDuration = 0;
    music.playbackState = 0;
  };
  music.seekToTime = async seconds => {
    restoredSeekTarget = seconds;
    music.playbackState = 4;
    emit('playbackStateDidChange');
  };
  music.play = async () => {
    played = true;
    music.currentPlaybackTime = restoredSeekTarget;
    music.currentPlaybackDuration = 240;
    music.playbackState = 2;
    emit('playbackStateDidChange');
  };
  music.pause = async () => { music.playbackState = 3; emit('playbackStateDidChange'); };
  const restoredOrder = { upcoming: [2, 1], manualCount: 1, context: [0, 1], history: [] };
  await app.dispatch({ type: 'intent', generation: 12, command: {
    type: 'restore', items: [uploaded, catalog, uploaded], index: 0,
    order: restoredOrder, seconds: 42, shuffle: false, repeat: 0
  } });
  assert(!played);
  assert.equal(lastState().status, 3, 'An unloaded paused seek must not leave restoration loading');
  assert.equal(lastState().actualPosition, 0, 'Do not report the requested position as an actual SDK seek');
  assert.equal(lastState().position, 42);
  assert.equal(lastState().requestGeneration, 12);
  assert.equal(calls.at(-1).items[0].isLibrary, true);
  assert.deepEqual(Array.from(lastState().order.upcoming), [2, 1]);
  await app.dispatch({ type: 'resume' });
  assert(played);
  assert.equal(lastState().status, 2);
  assert.equal(lastState().actualPosition, 42);
  await app.dispatch({ type: 'pause' });
  played = false;
  await app.dispatch({ type: 'next' });
  assert(!played);
  assert.equal(lastState().index, 2);
  assert.equal(lastState().position, 0, 'Next must discard the prior occurrence seek');
  music.playbackState = 1; emit('playbackStateDidChange');
  assert.equal(lastState().status, 1, 'Loading must remain loading');
  music.playbackState = 3;
  music.setQueue = restoreQueue; music.seekToTime = restoreSeek;
  music.play = restorePlay; music.pause = restorePause;
  music.currentPlaybackDuration = 240;

  let resolveRead;
  music.api.music = () => new Promise(resolve => {resolveRead=resolve;});
  const pendingRead=app.dispatch({type:'request',id:42,path:'/v1/me/library/albums?limit=100'});
  while (!resolveRead) await new Promise(resolve => setImmediate(resolve));
  await Promise.race([app.dispatch({type:'pause'}),new Promise((_,reject)=>setTimeout(()=>reject(new Error('Page read blocked pause')),250))]);
  resolveRead({data:{data:[]}});
  await pendingRead;
  assert.equal(messages.filter(event=>event.type==='response').at(-1).id,42);

  let resolveWrite, writeSignal, writeCalls = 0;
  music.api.music = (_, __, options) => { writeCalls++; writeSignal = options.fetchOptions.signal;
    return new Promise(resolve => { resolveWrite = resolve; }); };
  const pendingWrite = app.dispatch({ type: 'favoriteSongs', id: 105, items: [uploaded] });
  while (!resolveWrite) await new Promise(resolve => setImmediate(resolve));
  const queuedWrite = app.dispatch({ type: 'createPlaylist', id: 106, name: 'Canceled', public: false, items: [] });
  const queuedFavorite = app.dispatch({ type: 'favoriteSongs', id: 2021, items: [catalog] });
  const writeAPI = music.api.music, readSignals = [];
  music.api.music = (path, parameters, options) => {
    if (options.fetchOptions.method === 'POST') return writeAPI(path, parameters, options);
    const signal = options.fetchOptions.signal;
    readSignals.push(signal);
    return new Promise((_resolve, reject) => signal.addEventListener('abort',
      () => reject(new Error('secret canceled read')), { once: true }));
  };
  const pendingReads = [app.dispatch({ type: 'library', id: 110, next: null }),
    app.dispatch({ type: 'request', id: 203, path: '/v1/me/library/recently-added' })];
  while (readSignals.length < 2) await new Promise(resolve => setImmediate(resolve));
  const pendingAuthorization = app.dispatch({ type: 'authorize' });
  while (!authorization) await new Promise(resolve => setImmediate(resolve));
  const signOut = app.dispatch({ type: 'signOut' });
  assert(writeSignal.aborted);
  assert(readSignals.every(signal => signal.aborted));
  authorization('late-user-token');
  await Promise.race([signOut, new Promise((_, reject) => setTimeout(() => reject(new Error('Write blocked sign-out')), 250))]);
  resolveWrite({ data: { data: [] } });
  await pendingWrite; await queuedWrite; await queuedFavorite; await pendingAuthorization;
  await Promise.all(pendingReads);
  assert.equal(writeCalls, 1); // Queued mutations cannot execute under a new account.
  assert.equal(readSignals.length, 2, 'A stale favorites POST cannot start confirmation reads');
  assert(!messages.some(event => [105, 106, 2021].includes(event.id))); // Late write replies are stale.
  assert(!messages.some(event => [110, 203].includes(event.id)));
  assert(!JSON.stringify(messages).includes('secret canceled read'));
  assert(!messages.some(event => event.type === 'authorized'));
  assert.equal(messages.at(-1).type, 'signedOut');
  assert.equal(messages.at(-1).session, 2);
  assert.equal(readTimers.size, 0);
  // A fresh host can sign out during confirmation even if the SDK ignores abort.
  vm.runInNewContext(fs.readFileSync(path.join(__dirname, 'bridge.js'), 'utf8'), sandbox);
  const freshApp = sandbox.window.applifast;
  await freshApp.bootstrap({ developerToken: 'mock-only', session: 3 });
  let finishConfirmation, activeConfirmationSignal, confirmationSignOut, confirmationReads = 0;
  music.api.music = (_path, _parameters, options) => {
    if (options.fetchOptions.method === 'POST') return Promise.resolve();
    confirmationReads++;
    activeConfirmationSignal = options.fetchOptions.signal;
    confirmationSignOut = freshApp.dispatch({ type: 'signOut' });
    return new Promise(resolve => { finishConfirmation = resolve; });
  };
  const activeFavorite = freshApp.dispatch({ type: 'favoriteSongs', id: 2113, items: [catalog, uploaded] });
  while (!finishConfirmation) await new Promise(resolve => setImmediate(resolve));
  await Promise.race([confirmationSignOut,
    new Promise((_, reject) => setTimeout(() => reject(new Error('Confirmation blocked sign-out')), 250))]);
  assert(activeConfirmationSignal.aborted);
  assert.equal(confirmationReads, 1, 'Sign-out during one confirmation must prevent starting another');
  finishConfirmation({ data: { data: [{ id: '123', type: 'songs', attributes: { inFavorites: true } }] } });
  await activeFavorite;
  assert(!messages.some(event => event.id === 2113), 'A late confirmation cannot reach a new account');
  assert.equal(readTimers.size, 0);
  console.log('Bridge self-check passed: SDK startup deadline, uploaded IDs, occurrences, pagination, read deadlines, responsive transport, playlist/favorites writes, failures, cancellable and stale authorization.');
})().catch(failure => { console.error(failure); process.exitCode = 1; });
