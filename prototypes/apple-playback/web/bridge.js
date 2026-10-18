/* Diagnostic playback boundary, not the application's interface. */
(() => {
  'use strict';
  let music, session = 1, queue = [], index = -1, repeat = 0, shuffle = false;
  let order = { upcoming: [], manualCount: 0, context: [], history: [] };
  let nativeQueue = false;
  let compactQueue = false, nativeItem = null, nativeLength = 0;
  const queueLength = () => compactQueue ? nativeLength : queue.length;
  const currentItem = () => compactQueue ? nativeItem : queue[index];
  let chain = Promise.resolve(), initializing, signingOut = false, authorization = null;
  // The SDK can mutate its token after cancellation; retain only the accepted grant.
  let userToken = '';
  let writes = Promise.resolve();
  const apiControllers = new Set();
  let transitioning = false, playbackFailed = false;
  let requestGeneration = 0;
  let seekTarget = null;
  let desiredPlaying = false;
  const sdkLoaded = () => {
    if (window.MusicKit) return Promise.resolve();
    return new Promise((resolve, reject) => {
      const cleanup = () => {
        clearTimeout(timeout);
        document.removeEventListener('musickitloaded', loaded);
      };
      const loaded = () => { cleanup(); resolve(); };
      const timeout = setTimeout(() => {
        cleanup();
        reject(new Error('MusicKit did not load before the startup deadline.'));
      }, 45_000);
      document.addEventListener('musickitloaded', loaded, { once: true });
    });
  };
  const send = (type, fields = {}, generation = session) => {
    if (generation !== session) return;
    const event = { type, session: generation, ...fields };
    window.chrome.webview.postMessage(event);
    document.dispatchEvent(new CustomEvent('applifast-event', { detail: event }));
  };
  const apiFailure = failure => {
    const status = failure?.data?.status;
    return Number.isInteger(status) && status >= 400 && status < 600 ? { httpStatus: status } : {};
  };
  const error = (code, generation, intent = requestGeneration, failure) => send('error', {
    code, ...apiFailure(failure), ...(['intent', 'play', 'nativeQueue', 'restore', 'next', 'previous', 'pause', 'resume', 'seek', 'volume', 'repeat', 'playback'].includes(code) ? { requestGeneration: intent } : {}),
    message: 'Operation failed. Check authorization, subscription, connection, or song availability.'
  }, generation);
  const number = value => Number.isFinite(value) ? value : 0;
  async function requestMusic(path, parameters = {}, options = {}) {
    const controller = new AbortController();
    apiControllers.add(controller);
    let abort;
    const canceled = new Promise((_, reject) => {
      abort = () => reject(new Error('Request canceled.'));
      controller.signal.addEventListener('abort', abort, { once: true });
    });
    // Reads have a deadline. A timed-out write could have applied at Apple,
    // so mutations retain their existing acceptance/confirmation behavior.
    const timeout = options.method === 'POST' ? null : setTimeout(() => controller.abort(), 20_000);
    try {
      // The SDK may ignore abort; release our wait and ignore its late result.
      return await Promise.race([canceled, Promise.resolve().then(() => {
        if (controller.signal.aborted) throw new Error('Request canceled.');
        return music.api.music(path, parameters, {
          fetchOptions: { ...options, signal: controller.signal }
        });
      })]);
    } finally {
      if (timeout !== null) clearTimeout(timeout);
      controller.signal.removeEventListener('abort', abort);
      apiControllers.delete(controller);
    }
  }
  const songAttributes = path => {
    const [bare, query = ''] = path.split('?');
    const params = new URLSearchParams(query);
    params.set('extend[songs]', 'inFavorites');
    params.set('extend[library-songs]', 'inFavorites');
    if (/^\/v1\/me\/library\/songs(?:\/[\w.-]+)?$/.test(bare)) {
      params.set('extend', 'inFavorites');
      if (!params.has('include')) params.set('include', 'albums,artists');
    }
    return `${bare}?${params}`;
  };
  const state = () => {
    const position = number(music.currentPlaybackTime);
    if (seekTarget !== null && Math.abs(position - seekTarget) <= 1) seekTarget = null;
    // Normalize SDK states to the native transport contract, including unloaded pauses.
    const states = window.MusicKit.PlaybackStates;
    let status = number(music.playbackState);
    if (status === states.stopped) status = !desiredPlaying && index >= 0 ? 3 : 0;
    else if (status === states.ended || status === states.completed) status = 10;
    else if (status === states.stalled) status = 8;
    send('state', {
    status: transitioning ? 1 : status, position: transitioning ? 0 : (seekTarget ?? position),
    duration: number(music.currentPlaybackDuration), actualPosition: position, index, queueLength: queueLength(),
    requestGeneration, ...(!compactQueue ? { order: structuredClone(order) } : {})
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
    if (position < 0 || position >= queueLength()) return;
    const descriptor = song(compactQueue ? nativeItem : queue[position]);
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
  function queueOrder(value, items, current) {
    value = { ...value, history: value?.history || [] };
    const valid = indices => Array.isArray(indices) && indices.length <= items.length &&
      indices.every(at => Number.isInteger(at) && at >= 0 && at < items.length) &&
      new Set(indices).size === indices.length;
    if (!value || !valid(value.upcoming) || !valid(value.context) ||
        !Number.isInteger(value.manualCount) || value.manualCount < 0 || value.manualCount > value.upcoming.length ||
        value.upcoming.includes(current) ||
        value.upcoming.slice(0, value.manualCount).some(at => value.context.includes(at)) ||
        value.upcoming.slice(value.manualCount).some(at => !value.context.includes(at)) ||
        !Array.isArray(value.history) || value.history.length > 64 ||
        value.history.some(at => !Number.isInteger(at) || at < 0 || at >= items.length)) throw new Error('queue');
    return structuredClone(value);
  }
  function rememberCurrent() {
    order.history.push(index);
    if (order.history.length > 64) order.history.shift();
  }
  function shuffled(indices) {
    for (let i = indices.length - 1; i > 0; i--) {
      const at = Math.floor(Math.random() * (i + 1));
      [indices[i], indices[at]] = [indices[at], indices[i]];
    }
    return indices;
  }
  function jump(position) {
    if (position < 0 || position >= order.upcoming.length) return -1;
    if (index >= 0) rememberCurrent();
    const target = order.upcoming[position];
    order.upcoming.splice(0, position + 1);
    order.manualCount = Math.max(0, order.manualCount - position - 1);
    return target;
  }
  function nextPosition(direction, ended = false) {
    if (ended && repeat === 1) return index;
    if (direction > 0) {
      if (order.upcoming.length) return jump(0);
      if (repeat === 2 && order.context.length) {
        if (index >= 0) rememberCurrent();
        order.upcoming = order.context.slice(1);
        return order.context[0];
      }
    } else if (order.history.length) {
      const target = order.history.pop();
      if (index >= 0) {
        const manual = !order.context.includes(index);
        order.upcoming.splice(manual ? 0 : order.manualCount, 0, index);
        order.manualCount += Number(manual);
      }
      const at = order.upcoming.indexOf(target);
      if (at >= 0) { order.upcoming.splice(at, 1); order.manualCount -= Number(at < order.manualCount); }
      return target;
    }
    return -1;
  }
  function cancelAuthorization() {
    const attempt = authorization;
    authorization = null;
    attempt?.cancel();
    if (attempt) music.musicUserToken = userToken;
  }
  async function authorize(generation) {
    if (authorization || signingOut) return;
    const attempt = {};
    const canceled = new Promise(resolve => { attempt.cancel = () => resolve(null); });
    authorization = attempt;
    try {
      const requested = Promise.resolve(music.authorize()).then(token => {
        if (generation !== session || authorization !== attempt) {
          music.musicUserToken = userToken;
          return null;
        }
        return token;
      });
      const token = await Promise.race([requested, canceled]);
      if (token === null || generation !== session || authorization !== attempt) return;
      if (typeof token !== 'string' || !token) throw new Error('authorization');
      userToken = token;
      music.musicUserToken = token;
      send('authorized', { token, storefront: String(music.storefrontId || '') }, generation); // Native consumes the token; never renders/logs it.
    } catch { if (authorization === attempt) error('authorization', generation); }
    finally { if (authorization === attempt) authorization = null; }
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
        const response = await requestMusic(songAttributes(command.path));
        send('response', { id: command.id, data: response.data }, generation);
        return;
      }
      case 'favoriteSongs': {
        if (!Number.isSafeInteger(command.id) || command.id < 0 ||
            !Array.isArray(command.items) || !command.items.length || command.items.length > 100) {
          throw new Error('favorites');
        }
        const parameters = {}, groups = [];
        for (const kind of ['catalog', 'library']) {
          const ids = new Set();
          for (const item of command.items) {
            song(item);
            if (item.kind === kind) ids.add(item.id);
          }
          if (ids.size) {
            const type = kind === 'library' ? 'library-songs' : 'songs';
            if (kind === 'catalog' && (typeof music.storefrontId !== 'string' || !/^[a-z]{2}$/.test(music.storefrontId))) throw new Error('storefront');
            parameters[`ids[${type}]`] = [...ids].join(',');
            groups.push({ type, ids: [...ids], path: kind === 'library'
              ? '/v1/me/library/songs' : `/v1/catalog/${music.storefrontId}/songs` });
          }
        }
        await requestMusic('/v1/me/favorites', parameters, { method: 'POST' });
        if (generation !== session || signingOut) return;
        // Apple's 202 may ignore IDs. Confirm exact typed resources, never catalog matches.
        const results = await Promise.all(groups.map(async group => {
          if (generation !== session || signingOut) return [];
          let rows = [];
          try {
            const response = await requestMusic(group.path, { ids: group.ids.join(','), extend: 'inFavorites' }, { cache: 'no-store' });
            if (Array.isArray(response?.data?.data)) rows = response.data.data;
          } catch { /* An accepted mutation with a failed read remains unconfirmed. */ }
          return group.ids.map(id => {
            const matches = rows.filter(row => row?.id === id && row.type === group.type);
            const flag = matches.length === 1 ? matches[0].attributes?.inFavorites : undefined;
            return { id, type: group.type, attributes: typeof flag === 'boolean' ? { inFavorites: flag } : {} };
          });
        }));
        const data = results.flat();
        send('response', { id: command.id, data: { data },
          ...(data.every(row => row.attributes.inFavorites === true) ? {} : {
            error: 'Apple Music has not confirmed every favorite. Refresh your library before retrying.'
          }) }, generation);
        return;
      }
      case 'createPlaylist':
      case 'appendPlaylist': {
        const create = command.type === 'createPlaylist';
        if (!Number.isSafeInteger(command.id) || command.id < 0 ||
            !Array.isArray(command.items) || command.items.length > 1000 || (!create && !command.items.length) ||
            (create ? typeof command.name !== 'string' || !command.name.trim() ||
              new TextEncoder().encode(command.name).length > 1024 || typeof command.public !== 'boolean' :
              typeof command.playlist !== 'string' || command.playlist.length > 128 || !/^p\.[\w.-]+$/.test(command.playlist))) {
          throw new Error('playlist');
        }
        // Resource IDs identify playlist members. Playback IDs can differ and must not replace them.
        const data = command.items.map(item => {
          song(item);
          return { id: item.id, type: item.kind === 'library' ? 'library-songs' : 'songs' };
        });
        const body = create ? {
          attributes: { name: command.name.trim(), isPublic: command.public },
          ...(data.length ? { relationships: { tracks: { data } } } : {})
        } : { data };
        const path = '/v1/me/library/playlists' + (create ? '' : `/${command.playlist}/tracks`);
        const response = await requestMusic(path, {}, {
          method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body)
        });
        send('response', { id: command.id, data: response?.data ?? null }, generation);
        return;
      }
      case 'library': {
        if (command.id != null && (!Number.isSafeInteger(command.id) || command.id < 0)) {
          throw new Error('libraryRequest');
        }
        const route = command.next || '/v1/me/library/songs?limit=100&include=albums,artists';
        if (typeof route !== 'string' || !route.startsWith('/v1/me/library/songs?') || route.length > 2048) {
          throw new Error('pagination');
        }
        const response = await requestMusic(songAttributes(route));
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
            catalogId: p && p.catalogId || null,
            inFavorites: typeof a.inFavorites === 'boolean' ? a.inFavorites : null };
        });
        const next = page.next || null;
        if (next && (!next.startsWith('/v1/me/library/songs?') || next.length > 2048)) {
          throw new Error('pagination');
        }
        send('library', { id: command.id, items, next }, generation);
        return;
      }
      case 'play':
        requestGeneration = command.generation ?? requestGeneration;
        if (!Array.isArray(command.items) || !command.items.length || command.items.length > 1000 ||
            !Number.isInteger(command.index) || command.index < 0 || command.index >= command.items.length) {
          throw new Error('queue');
        }
        command.items.forEach(song);
        order = queueOrder(command.order || {upcoming: Array.from({length: command.items.length - command.index - 1}, (_, at) => command.index + at + 1),
          manualCount: 0, context: command.items.map((_, at) => at)}, command.items, command.index);
        queue = structuredClone(command.items);
        compactQueue = false; nativeItem = null; nativeLength = 0;
        nativeQueue = !!command.order;

        return playAt(command.index, generation);
      case 'restore': {
        if (!Array.isArray(command.items) || command.items.length > 1000 ||
            (command.index !== null && (!Number.isInteger(command.index) || command.index < 0 || command.index >= command.items.length)) ||
            !Number.isFinite(command.seconds) || command.seconds < 0 || typeof command.shuffle !== 'boolean' ||
            ![0, 1, 2].includes(command.repeat)) throw new Error('restore');
        command.items.forEach(song);
        order = queueOrder(command.order, command.items, command.index);
        queue = structuredClone(command.items);
        compactQueue = false; nativeItem = null; nativeLength = 0;
        index = command.index ?? -1;
        nativeQueue = true;
        shuffle = command.shuffle;
        repeat = command.repeat;
        desiredPlaying = false;
        if (index >= 0) {
          await playAt(index, generation, false);
          if (generation !== session) return;
          return perform({ type: 'seek', seconds: command.seconds }, generation);
        }
        break;
      }
      case 'queue': {
        if (!Array.isArray(command.items) || command.items.length > 1000 ||
            (command.index !== null && (!Number.isInteger(command.index) || command.index < 0 || command.index >= command.items.length))) throw new Error('queue');
        command.items.forEach(song);
        const current = command.index ?? -1;
        if ((current < 0) !== (index < 0) || (current >= 0 && JSON.stringify(command.items[current]) !== JSON.stringify(currentItem()))) throw new Error('queue');
        order = queueOrder(command.order, command.items, current);
        queue = structuredClone(command.items);
        compactQueue = false; nativeItem = null; nativeLength = 0;
        index = current;
        nativeQueue = true;
        break;
      }
      case 'select': {
        if (!Number.isInteger(command.index) || command.index < 0 || command.index >= queue.length || typeof command.playing !== 'boolean') throw new Error('queue');
        order = queueOrder(command.order, queue, command.index);
        nativeQueue = true;
        return playAt(command.index, generation, command.playing);
      }
      case 'nativeQueue': {
        if (!Number.isInteger(command.length) || command.length < 0 || command.length > 100_000 ||
            !(command.index === null || Number.isInteger(command.index) && command.index >= 0 && command.index < command.length) ||
            (command.item === null) !== (command.index === null) ||
            !(command.playing === null || typeof command.playing === 'boolean') ||
            (command.playing !== null && command.index === null) ||
            !(command.seconds === null || Number.isFinite(command.seconds) && command.seconds >= 0 && command.playing === false) ||
            typeof command.shuffle !== 'boolean' || ![0, 1, 2].includes(command.repeat) ||
            !(command.generation === undefined || Number.isSafeInteger(command.generation) && command.generation >= 0)) throw new Error('queue');
        if (command.item !== null) song(command.item);
        if (command.playing === null && ((command.index === null) !== (index < 0) ||
            command.item !== null && JSON.stringify(command.item) !== JSON.stringify(currentItem()))) throw new Error('queue');
        requestGeneration = command.generation ?? requestGeneration;
        nativeItem = structuredClone(command.item);
        nativeLength = command.length;
        index = command.index ?? -1;
        queue = [];
        order = { upcoming: [], manualCount: 0, context: [], history: [] };
        compactQueue = nativeQueue = true;
        shuffle = command.shuffle; repeat = command.repeat;
        if (command.playing !== null) {
          await playAt(index, generation, command.playing);
          if (generation !== session) return;
          if (command.seconds !== null) return perform({ type: 'seek', seconds: command.seconds }, generation);
        }
        break;
      }
      case 'jump':
        if (compactQueue) throw new Error('queue');
        return playAt(jump(command.position), generation, desiredPlaying);
      case 'next':
        if (compactQueue) throw new Error('queue');
        return playAt(nextPosition(1), generation, desiredPlaying);
      case 'previous':
        if (compactQueue) throw new Error('queue');
        return playAt(nextPosition(-1), generation, desiredPlaying);
      case 'pause': desiredPlaying = false; await music.pause(); break;
      case 'resume':
        desiredPlaying = true;
        if (index < 0) return playAt(nextPosition(1), generation);
        await music.play(); break;
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
      case 'shuffle':
        shuffle = command.enabled === true;
        if (compactQueue) break;
        const rest = order.upcoming.splice(order.manualCount);
        order.upcoming.push(...(shuffle ? shuffled(rest) : rest.sort((a, b) => a - b)));
        if (shuffle) shuffled(order.context); else order.context.sort((a, b) => a - b);
        break;
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
      userToken = '';
      cancelAuthorization();
      for (const controller of apiControllers) controller.abort();
      writes = Promise.resolve();
      desiredPlaying = false;
      queue = []; index = -1; order = { upcoming: [], manualCount: 0, context: [], history: [] };
      compactQueue = false; nativeItem = null; nativeLength = 0;
      const result = (async () => {
        await initializing;
        await music.pause();
        await pending;
        await music.unauthorize();
        userToken = '';
        music.musicUserToken = '';
        if (generation === session) { signingOut = false; send('signedOut'); }
      })().catch(() => error('signOut', generation));
      chain = result;
      return result;
    }
    const generation = session;
    if (['createPlaylist', 'appendPlaylist', 'favoriteSongs'].includes(command.type)) {
      // Writes serialize independently: a slow library mutation cannot stall playback controls.
      writes = writes.then(() => initializing).then(() => perform(command, generation))
        .catch(failure => send('response', { id: command.id, ...apiFailure(failure),
          error: command.type === 'favoriteSongs'
            ? 'Apple Music could not confirm these favorites. Check sign-in and connection, then refresh your library before retrying.'
            : 'Apple Music could not save this playlist. Check sign-in, connection and playlist permissions, then retry.' }, generation));
      return writes;
    }
    if (command.type === 'request') {
      return Promise.resolve(initializing).then(() => perform(command, generation))
        .catch(failure => send('response', { id: command.id, ...apiFailure(failure), error: 'Apple Music could not load this page. Check sign-in and connection, then retry.' }, generation));
    }
    if (command.type === 'library') {
      // Metadata reads must not make transport commands wait for the network.
      return Promise.resolve(initializing).then(() => perform(command, generation))
        .catch(failure => Number.isSafeInteger(command.id) && command.id >= 0
          ? send('library', { id: command.id, items: [], next: null, ...apiFailure(failure),
            error: 'Apple Music could not load songs. Check sign-in and connection, then refresh Songs.' }, generation)
          : error('library', generation, undefined, failure));
    }
    chain = chain.then(() => initializing).then(() => perform(command, generation))
      .catch(() => error(command.type, generation));
    return chain;
  }
  function bootstrap(settings) {
    if (initializing) return initializing;
    session = settings.session;
    initializing = (async () => {
      await sdkLoaded();
      music = await window.MusicKit.configure({ developerToken: settings.developerToken,
        app: { name: 'Applifast Playback Probe', build: '0.1.0' } });
      userToken = settings.userToken || '';
      if (userToken) music.musicUserToken = userToken;
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
        if (!nativeQueue && !signingOut && !transitioning && !playbackFailed && index >= 0 &&
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
  window.applifast = { bootstrap, dispatch, request, cancelAuthorization };
})();
