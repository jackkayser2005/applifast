/* Diagnostic playback boundary, not the application's interface. */
(() => {
  'use strict';
  let music, session = 1, queue = [], index = -1, repeat = 0, shuffle = false;
  let chain = Promise.resolve(), initializing, signingOut = false, authorizing = false;
  let transitioning = false, playbackFailed = false;
  let requestGeneration = 0;
  let seekTarget = null;
  let desiredPlaying = false;
  const sdkLoaded = new Promise(resolve => {
    if (window.MusicKit) resolve();
    else document.addEventListener('musickitloaded', resolve, { once: true });
  });
  const send = (type, fields = {}, generation = session) => {
    if (generation !== session) return;
    const event = { type, session: generation, ...fields };
    window.chrome.webview.postMessage(event);
    document.dispatchEvent(new CustomEvent('applifast-event', { detail: event }));
  };
  const error = (code, generation, intent = requestGeneration) => send('error', {
    code, ...(['intent', 'play', 'next', 'previous', 'pause', 'resume', 'seek', 'volume', 'repeat', 'playback'].includes(code) ? { requestGeneration: intent } : {}),
    message: 'Operation failed. Check authorization, subscription, connection, or song availability.'
  }, generation);
  const number = value => Number.isFinite(value) ? value : 0;
  const state = () => {
    const position = number(music.currentPlaybackTime);
    if (seekTarget !== null && Math.abs(position - seekTarget) <= 1) seekTarget = null;
    send('state', {
    status: transitioning ? 1 : number(music.playbackState), position: transitioning ? 0 : (seekTarget ?? position),
    duration: number(music.currentPlaybackDuration), actualPosition: position, index, queueLength: queue.length,
    requestGeneration
  }); };
  function song(item) {
    if (!item) throw new Error('invalidItem');
    const library = item.kind === 'library';
    if (!['library', 'catalog'].includes(item.kind) || typeof item.id !== 'string' ||
        !(library ? /^i\.[\w.-]+$/ : /^\d+$/).test(item.id) || item.id.length > 128) {
      throw new Error('invalidItem');
    }
    if (item.playParams && (typeof item.playParams.id !== 'string' || item.playParams.id.length > 128 ||
        (library ? item.playParams.id !== item.id || item.playParams.isLibrary !== true : !/^\d+$/.test(item.playParams.id)))) throw new Error('invalidItem');
    // Pass Apple's original playback parameters; never replace a library ID with catalogId.
    return item.playParams ? { ...item.playParams } : {
      id: item.id, kind: 'song', ...(library ? { isLibrary: true } : {})
    };
  }
  async function playAt(position, generation, startPlaying = true) {
    if (generation !== session || signingOut) return;
    if (position < 0 || position >= queue.length) return;
    const descriptor = song(queue[position]);
    index = position;
    seekTarget = null;
    transitioning = true;
    playbackFailed = false;
    state();
    // MusicKit's bulk loader keys by song ID. A single item preserves local occurrences.
    try {
      await music.setQueue({ items: [descriptor], startWith: 0, startPlaying: false });
      if (generation !== session) return;
      if (music.queue && music.queue.length !== 1) throw new Error('unavailableItem');
      desiredPlaying = startPlaying;
      if (startPlaying) await music.play();
      else await music.pause();
      if (generation === session) state();
    } catch (failure) {
      playbackFailed = true;
      desiredPlaying = false;
      await music.pause();
      throw failure;
    } finally { transitioning = false; if (generation === session) state(); }
  }
  function nextPosition(direction, ended = false) {
    if (ended && repeat === 1) return index;
    if (shuffle && queue.length > 1) {
      const offset = 1 + Math.floor(Math.random() * (queue.length - 1));
      return (index + offset) % queue.length;
    }
    const candidate = index + direction;
    if (candidate >= 0 && candidate < queue.length) return candidate;
    return repeat === 2 ? (candidate + queue.length) % queue.length : -1;
  }
  async function authorize(generation) {
    if (authorizing || signingOut) return;
    authorizing = true;
    try {
      const token = await music.authorize();
      if (generation !== session) { music.musicUserToken = ''; return; }
      if (typeof token !== 'string' || !token) throw new Error('authorization');
      send('authorized', { token, storefront: String(music.storefrontId || '') }, generation); // Native consumes the token; never renders/logs it.
    } catch { error('authorization', generation); }
    finally { authorizing = false; }
  }
  async function perform(command, generation) {
    if (generation !== session || signingOut) return;
    switch (command.type) {
      case 'intent':
        requestGeneration = command.generation;
        return perform(command.command, generation);
      case 'authorize': return authorize(generation);
      case 'request': {
        // The native boundary validates the relative path before it reaches MusicKit.
        const response = await music.api.music(command.path);
        send('response', { id: command.id, data: response.data }, generation);
        return;
      }
      case 'library': {
        const route = command.next || '/v1/me/library/songs?limit=100&include=albums,artists';
        if (typeof route !== 'string' || !route.startsWith('/v1/me/library/songs?') || route.length > 2048) {
          throw new Error('pagination');
        }
        const response = await music.api.music(route);
        if (generation !== session) return;
        const page = response.data;
        if (!page || !Array.isArray(page.data)) throw new Error('libraryResponse');
        const items = page.data.map(resource => {
          const a = resource.attributes || {}, p = a.playParams || null;
          return { kind: 'library', id: resource.id, title: a.name || '', artist: a.artistName || '',
            album: a.albumName || '', durationMs: number(a.durationInMillis), playParams: p,
            artwork: a.artwork?.url?.replace('{w}', '640').replace('{h}', '640') || null,
            albumId: resource.relationships?.albums?.data?.[0]?.id ? `library.${resource.relationships.albums.data[0].id}` : null,
            artistId: resource.relationships?.artists?.data?.[0]?.id ? `library.${resource.relationships.artists.data[0].id}` : null,
            catalogId: p && p.catalogId || null };
        });
        const next = page.next || null;
        if (next && (!next.startsWith('/v1/me/library/songs?') || next.length > 2048)) {
          throw new Error('pagination');
        }
        send('library', { items, next }, generation);
        return;
      }
      case 'play':
        requestGeneration = command.generation ?? requestGeneration;
        if (!Array.isArray(command.items) || !command.items.length || command.items.length > 1000 ||
            !Number.isInteger(command.index) || command.index < 0 || command.index >= command.items.length) {
          throw new Error('queue');
        }
        command.items.forEach(song);
        queue = structuredClone(command.items);
        return playAt(command.index, generation);
      case 'next': return playAt(nextPosition(1), generation, desiredPlaying);
      case 'previous': return playAt(nextPosition(-1), generation, desiredPlaying);
      case 'pause': desiredPlaying = false; await music.pause(); break;
      case 'resume': desiredPlaying = true; await music.play(); break;
      case 'seek': {
        if (!Number.isFinite(command.seconds) || command.seconds < 0) throw new Error('seek');
        seekTarget = command.seconds;
        // The SDK can report a completed seek while its promise remains pending.
        // Do not let that promise block later pause, next or sign-out commands.
        const intent = requestGeneration;
        void Promise.resolve(music.seekToTime(command.seconds)).catch(() => error('seek', generation, intent));
        break;
      }
      case 'volume':
        if (!Number.isFinite(command.value) || command.value < 0 || command.value > 1) throw new Error('volume');
        music.volume = command.value; break;
      case 'shuffle': shuffle = command.enabled === true; break;
      case 'repeat':
        if (![0, 1, 2].includes(command.mode)) throw new Error('repeat');
        repeat = command.mode; break;
      case 'probe':
        send('probe', { sdkVersion: String(window.MusicKit.version || 'v3'),
          drm: music.browserSupportsVideoDrm === true, secureContext: window.isSecureContext });
        return;
      default: throw new Error('command');
    }
    if (generation === session) state();
  }
  function dispatch(command) {
    if (command.type === 'signOut') {
      const pending = chain;
      const generation = ++session;
      signingOut = true;
      desiredPlaying = false;
      queue = []; index = -1;
      const result = (async () => {
        await initializing;
        await music.pause();
        await pending;
        await music.unauthorize();
        music.musicUserToken = '';
        if (generation === session) { signingOut = false; send('signedOut'); }
      })().catch(() => error('signOut', generation));
      chain = result;
      return result;
    }
    const generation = session;
    if (command.type === 'request') {
      return Promise.resolve(initializing).then(() => perform(command, generation))
        .catch(() => send('response', { id: command.id, error: 'Apple Music could not load this page. Check sign-in and connection, then retry.' }, generation));
    }
    chain = chain.then(() => initializing).then(() => perform(command, generation))
      .catch(() => error(command.type, generation));
    return chain;
  }
  function bootstrap(settings) {
    if (initializing) return initializing;
    session = settings.session;
    initializing = (async () => {
      await sdkLoaded;
      music = await window.MusicKit.configure({ developerToken: settings.developerToken,
        app: { name: 'Applifast Playback Probe', build: '0.1.0' } });
      if (settings.userToken) music.musicUserToken = settings.userToken;
      // Never count a 30-second preview as full-track playback.
      music.playbackMode = window.MusicKit.PlaybackMode.FULL_PLAYBACK_ONLY;
      music.autoplayEnabled = false;
      music.repeatMode = 0;
      const events = window.MusicKit.Events;
      for (const name of ['playbackTimeDidChange', 'playbackStateDidChange', 'nowPlayingItemDidChange']) {
        music.addEventListener(events[name], () => {
          // Seeking may finish by resuming audio after an earlier pause request.
          if (!signingOut && !desiredPlaying && music.playbackState === window.MusicKit.PlaybackStates.playing) void music.pause();
          if (!signingOut && index >= 0) state();
        });
      }
      music.addEventListener(events.mediaPlaybackError, () => {
        playbackFailed = true;
        desiredPlaying = false;
        void music.pause();
        error('playback', session); // Queue/index stay visible. Do not skip an unavailable song.
      });
      music.addEventListener(events.playbackStateDidChange, () => {
        if (!signingOut && !transitioning && !playbackFailed && index >= 0 &&
            music.playbackState === window.MusicKit.PlaybackStates.ended) {
          const generation = session;
          chain = chain.then(() => playAt(nextPosition(1, true), generation)).catch(() => error('playback', generation));
        }
      });
      send('ready', { sdkVersion: String(window.MusicKit.version || 'v3'),
        authorized: music.isAuthorized === true, storefront: String(music.storefrontId || '') });
    })();
    initializing.catch(() => error('configuration', session));
    return initializing;
  }
  // Visible diagnostic controls use the same native command boundary as stdin.
  const request = command => window.chrome.webview.postMessage({ type: 'command', session, command });
  window.applifast = { bootstrap, dispatch, request };
})();
