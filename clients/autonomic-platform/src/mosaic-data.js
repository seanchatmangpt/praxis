/**
 * mosaic-data.js
 * -----------------------------------------------------------------------------
 * Portable Supabase data hooks for Ten Four. These touch ONLY the supabase
 * client and React — zero DOM, zero React Native, zero web. They port to Expo
 * verbatim: copy this file in, swap the one import in mosaic-client, and every
 * call site is unchanged.
 *
 *   const { data } = useQuery('loads', q => q.eq('status','available'));
 *   const runs     = useLiveQuery('runs');           // re-fetches on realtime
 *   const { call } = useEdgeFunction('book-load');
 *   const { call: rpc } = useRpc('fleet_health');
 *   const { session, signIn, signOut } = useAuth();
 *
 * Exposed as window.MosaicData (UMD-ish) so a DC can mount it without a bundler.
 * -----------------------------------------------------------------------------
 */
(function (global) {
  'use strict';
  if (!global.React) {
    var _tries = 0;
    var _wait = setInterval(function () {
      if (global.React) { clearInterval(_wait); build(global.React); }
      else if (++_tries > 200) { clearInterval(_wait); console.error('mosaic-data: React global never appeared'); }
    }, 25);
    return;
  }
  build(global.React);

  function build(React) {

  // ---- the live client singleton (set by initClient) -----------------------
  var _client = null;
  var _clientListeners = [];
  var _initPromise = null;
  function setClient(c) { _client = c; _clientListeners.forEach(function (cb) { cb(c); }); }
  function getClient() { return _client; }

  // Initialize once from the mock (or, in Expo, from @supabase/supabase-js).
  // De-duped: concurrent callers share one in-flight promise → one client.
  async function initClient(url, key, opts) {
    if (_client) return _client;
    if (_initPromise) return _initPromise;
    _initPromise = import('./supabase-mock.js').then(function (mod) {
      if (!_client) setClient(mod.createClient(url || 'https://tenfour.supabase.co', key || 'anon-key', opts || {}));
      return _client;
    });
    return _initPromise;
  }

  // React context so children can read the client once it exists.
  var ClientContext = React.createContext(null);

  function SupabaseProvider(props) {
    var st = React.useState(_client), client = st[0], setLocal = st[1];
    React.useEffect(function () {
      var alive = true;
      if (!_client) initClient(props.url, props.anonKey, props.options).then(function (c) { if (alive) setLocal(c); });
      else setLocal(_client);
      var cb = function (c) { if (alive) setLocal(c); };
      _clientListeners.push(cb);
      return function () { alive = false; _clientListeners = _clientListeners.filter(function (l) { return l !== cb; }); };
    }, []);
    // Withhold children until the client exists — guarantees useSupabase() is
    // never null in any descendant, so hooks can call .rpc / .from freely.
    if (!client) {
      return React.createElement('div', {
        style: { width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', background: '#0E1116' },
      }, React.createElement('div', {
        style: { width: 26, height: 26, borderRadius: 26, border: '3px solid rgba(255,182,39,0.25)', borderTopColor: '#FFB627', animation: 'mn-spin 0.7s linear infinite' },
      }));
    }
    return React.createElement(ClientContext.Provider, { value: client }, props.children);
  }

  function useSupabase() {
    var ctx = React.useContext(ClientContext);
    return ctx || _client;
  }

  /* ============================ useAuth ================================== */
  function useAuth() {
    var sb = useSupabase();
    var s = React.useState(null), session = s[0], setSession = s[1];
    var l = React.useState(true), loading = l[0], setLoading = l[1];
    React.useEffect(function () {
      if (!sb) return;
      var sub = null;
      sb.auth.getSession().then(function (res) { setSession(res.data.session); setLoading(false); });
      var r = sb.auth.onAuthStateChange(function (_event, sess) { setSession(sess); setLoading(false); });
      sub = r && r.data && r.data.subscription;
      return function () { sub && sub.unsubscribe(); };
    }, [sb]);
    return {
      session: session, user: session && session.user, loading: loading,
      signIn: function (email, password) { return sb.auth.signInWithPassword({ email: email, password: password }); },
      signInWithOAuth: function (provider) { return sb.auth.signInWithOAuth({ provider: provider }); },
      signUp: function (email, password, data) { return sb.auth.signUp({ email: email, password: password, options: { data: data } }); },
      signOut: function () { return sb.auth.signOut(); },
    };
  }

  /* ============================ useQuery ================================= */
  // one-shot fetch; `build` lets you chain filters/order/limit on the builder.
  function useQuery(table, build, deps) {
    var sb = useSupabase();
    var d = React.useState(null), data = d[0], setData = d[1];
    var e = React.useState(null), error = e[0], setError = e[1];
    var l = React.useState(true), loading = l[0], setLoading = l[1];
    var tick = React.useState(0), refreshKey = tick[0], setTick = tick[1];
    var refetch = React.useCallback(function () { setTick(function (n) { return n + 1; }); }, []);
    React.useEffect(function () {
      if (!sb) return;
      var alive = true;
      setLoading(true);
      var q = sb.from(table).select('*');
      if (build) q = build(q) || q;
      q.then(function (res) {
        if (!alive) return;
        if (res.error) setError(res.error); else { setData(res.data); setError(null); }
        setLoading(false);
      });
      return function () { alive = false; };
    }, [sb, table, refreshKey].concat(deps || []));
    return { data: data, error: error, loading: loading, refetch: refetch };
  }

  /* ========================== useLiveQuery ============================== */
  // like useQuery but re-fetches whenever a realtime change hits the table.
  function useLiveQuery(table, build, deps) {
    var q = useQuery(table, build, deps);
    var sb = useSupabase();
    React.useEffect(function () {
      if (!sb) return;
      var ch = sb.channel('live:' + table)
        .on('postgres_changes', { event: '*', schema: 'public', table: table }, function () { q.refetch(); })
        .subscribe();
      return function () { sb.removeChannel(ch); };
    }, [sb, table]);
    return q;
  }

  /* ============================= useRpc ================================= */
  function useRpc(fn) {
    var sb = useSupabase();
    var d = React.useState(null), data = d[0], setData = d[1];
    var p = React.useState(false), pending = p[0], setPending = p[1];
    var e = React.useState(null), error = e[0], setError = e[1];
    var call = React.useCallback(function (args) {
      if (!sb) return Promise.resolve({ data: null, error: { message: 'client not ready' } });
      setPending(true);
      return sb.rpc(fn, args || {}).then(function (res) {
        setPending(false);
        if (res.error) { setError(res.error); return res; }
        setData(res.data); setError(null); return res;
      });
    }, [sb, fn]);
    return { call: call, data: data, pending: pending, error: error };
  }

  /* ========================= useEdgeFunction =========================== */
  function useEdgeFunction(name) {
    var sb = useSupabase();
    var d = React.useState(null), data = d[0], setData = d[1];
    var p = React.useState(false), pending = p[0], setPending = p[1];
    var e = React.useState(null), error = e[0], setError = e[1];
    var call = React.useCallback(function (body) {
      if (!sb) return Promise.resolve({ data: null, error: { message: 'client not ready' } });
      setPending(true);
      return sb.functions.invoke(name, { body: body || {} }).then(function (res) {
        setPending(false);
        if (res.error) { setError(res.error); return res; }
        setData(res.data); setError(null); return res;
      });
    }, [sb, name]);
    return { call: call, data: data, pending: pending, error: error };
  }

  /* ========================= usePresence =============================== */
  // shared roster of who's online on a channel (drivers + dispatchers).
  function usePresence(channelName, me) {
    var sb = useSupabase();
    var st = React.useState([]), members = st[0], setMembers = st[1];
    React.useEffect(function () {
      if (!sb) return;
      var ch = sb.channel(channelName, { config: { presence: { key: (me && me.key) || 'me' } } });
      ch.on('presence', { event: 'sync' }, function () {
        var state = ch.presenceState();
        var flat = [];
        Object.keys(state).forEach(function (k) { (state[k] || []).forEach(function (m) { flat.push(m); }); });
        setMembers(flat);
      }).subscribe(function (status) { if (status === 'SUBSCRIBED' && me) ch.track(me); });
      return function () { sb.removeChannel(ch); };
    }, [sb, channelName]);
    return members;
  }

  global.MosaicData = {
    initClient: initClient, getClient: getClient, setClient: setClient,
    SupabaseProvider: SupabaseProvider, ClientContext: ClientContext,
    useSupabase: useSupabase, useAuth: useAuth,
    useQuery: useQuery, useLiveQuery: useLiveQuery,
    useRpc: useRpc, useEdgeFunction: useEdgeFunction, usePresence: usePresence,
  };
  if (typeof module !== 'undefined' && module.exports) module.exports = global.MosaicData;
  }
})(typeof window !== 'undefined' ? window : this);
