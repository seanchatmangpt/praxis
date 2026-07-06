/**
 * SupabaseProvider.js
 * -----------------------------------------------------------------------------
 * React abstraction layer over a Supabase client. Works with the real
 * @supabase/supabase-js client OR the bundled `supabase-mock.js` — pass whichever
 * into <SupabaseProvider client={...}>. If you pass nothing, it spins up the mock
 * so the whole app runs with zero backend.
 *
 * Hooks:
 *   useSupabaseClient()                      → the raw client
 *   useAuth()                                → { session, user, loading, signIn,
 *                                               signUp, signInWithOAuth, signOut,
 *                                               providers, lastEvent }
 *   useTable(table, build?, { realtime })    → { data, loading, error, count,
 *                                               refetch, insert, update, remove }
 *   useRealtimeChannel(name, setup, deps)    → the live channel
 *   usePresence(name, userState)             → { online, channel }
 *   useBroadcast(name, event)                → { messages, send }
 *   useRpc(fn)                               → [invoke, { data, loading, error }]
 *   useEdgeFunction(name)                    → [invoke, { data, loading, error }]
 *   useStorage(bucket)                       → { files, loading, refresh, upload,
 *                                               remove, publicUrl }
 * -----------------------------------------------------------------------------
 */

import React, {
  createContext, useContext, useEffect, useState, useCallback, useRef, useMemo,
} from 'react';
import { createClient } from './supabase-mock.js';

const SupabaseContext = createContext(null);

export function SupabaseProvider({ client, url, anonKey, children }) {
  const ref = useRef(null);
  if (!ref.current) ref.current = client || createClient(url, anonKey);
  return <SupabaseContext.Provider value={ref.current}>{children}</SupabaseContext.Provider>;
}

export function useSupabaseClient() {
  const c = useContext(SupabaseContext);
  if (!c) throw new Error('useSupabaseClient must be used within <SupabaseProvider>');
  return c;
}

/* ----------------------------------------------------------------------------
 * Auth
 * --------------------------------------------------------------------------*/
export function useAuth() {
  const supabase = useSupabaseClient();
  const [session, setSession] = useState(null);
  const [loading, setLoading] = useState(true);
  const [lastEvent, setLastEvent] = useState(null);

  useEffect(() => {
    let active = true;
    supabase.auth.getSession().then(({ data }) => { if (active) { setSession(data.session); setLoading(false); } });
    const { data: { subscription } } = supabase.auth.onAuthStateChange((event, sess) => {
      if (!active) return;
      setLastEvent(event);
      setSession(sess);
      setLoading(false);
    });
    return () => { active = false; subscription.unsubscribe(); };
  }, [supabase]);

  const signIn = useCallback((email, password) => supabase.auth.signInWithPassword({ email, password }), [supabase]);
  const signUp = useCallback((email, password, data) => supabase.auth.signUp({ email, password, options: { data } }), [supabase]);
  const signInWithOAuth = useCallback((provider) => supabase.auth.signInWithOAuth({ provider }), [supabase]);
  const signOut = useCallback(() => supabase.auth.signOut(), [supabase]);

  return {
    session, user: session?.user ?? null, loading, lastEvent,
    signIn, signUp, signInWithOAuth, signOut,
    providers: supabase.auth.providers || [],
  };
}

/* ----------------------------------------------------------------------------
 * Database table — query + optional realtime subscription + mutations
 * --------------------------------------------------------------------------*/
export function useTable(table, build, { realtime = true, deps = [] } = {}) {
  const supabase = useSupabaseClient();
  const [data, setData] = useState([]);
  const [count, setCount] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const buildRef = useRef(build);
  buildRef.current = build;

  const refetch = useCallback(async () => {
    setLoading(true);
    let q = supabase.from(table).select('*', { count: 'exact' });
    if (buildRef.current) q = buildRef.current(q) || q;
    const res = await q;
    if (res.error) setError(res.error);
    else { setData(Array.isArray(res.data) ? res.data : res.data ? [res.data] : []); setCount(res.count ?? null); setError(null); }
    setLoading(false);
    return res;
  }, [supabase, table]);

  useEffect(() => { refetch(); /* eslint-disable-next-line */ }, [refetch, ...deps]);

  // realtime: refetch on any change to the table (simple + correct).
  useEffect(() => {
    if (!realtime) return undefined;
    const ch = supabase
      .channel(`realtime:${table}`)
      .on('postgres_changes', { event: '*', schema: 'public', table }, () => refetch())
      .subscribe();
    return () => { supabase.removeChannel(ch); };
  }, [supabase, table, realtime, refetch]);

  const insert = useCallback((rows) => supabase.from(table).insert(rows).select(), [supabase, table]);
  const update = useCallback((patch, match) => { let q = supabase.from(table).update(patch); Object.entries(match || {}).forEach(([k, v]) => { q = q.eq(k, v); }); return q.select(); }, [supabase, table]);
  const remove = useCallback((match) => { let q = supabase.from(table).delete(); Object.entries(match || {}).forEach(([k, v]) => { q = q.eq(k, v); }); return q; }, [supabase, table]);

  return { data, count, loading, error, refetch, insert, update, remove };
}

/* ----------------------------------------------------------------------------
 * Realtime channel (raw)
 * --------------------------------------------------------------------------*/
export function useRealtimeChannel(name, setup, deps = []) {
  const supabase = useSupabaseClient();
  const [channel, setChannel] = useState(null);
  useEffect(() => {
    const ch = supabase.channel(name);
    setup && setup(ch);
    ch.subscribe();
    setChannel(ch);
    return () => { supabase.removeChannel(ch); };
    // eslint-disable-next-line
  }, [supabase, name, ...deps]);
  return channel;
}

export function usePresence(name, userState) {
  const supabase = useSupabaseClient();
  const [online, setOnline] = useState([]);
  useEffect(() => {
    const ch = supabase.channel(name);
    ch.on('presence', { event: 'sync' }, () => {
      const state = ch.presenceState();
      setOnline(Object.entries(state).flatMap(([key, metas]) => metas.map((m) => ({ key, ...m }))));
    });
    ch.subscribe((status) => { if (status === 'SUBSCRIBED') ch.track(userState || { key: 'me', online_at: new Date().toISOString() }); });
    return () => { ch.untrack(); supabase.removeChannel(ch); };
    // eslint-disable-next-line
  }, [supabase, name]);
  return { online };
}

export function useBroadcast(name, event = 'message') {
  const supabase = useSupabaseClient();
  const [messages, setMessages] = useState([]);
  const chRef = useRef(null);
  useEffect(() => {
    const ch = supabase.channel(name);
    ch.on('broadcast', { event }, (msg) => setMessages((m) => [...m, msg].slice(-50)));
    ch.subscribe();
    chRef.current = ch;
    return () => { supabase.removeChannel(ch); };
    // eslint-disable-next-line
  }, [supabase, name, event]);
  const send = useCallback((payload) => chRef.current?.send({ type: 'broadcast', event, payload }), [event]);
  return { messages, send };
}

/* ----------------------------------------------------------------------------
 * RPC + Edge functions
 * --------------------------------------------------------------------------*/
export function useRpc(fn) {
  const supabase = useSupabaseClient();
  const [state, setState] = useState({ data: null, loading: false, error: null });
  const invoke = useCallback(async (args) => {
    setState({ data: null, loading: true, error: null });
    const res = await supabase.rpc(fn, args);
    setState({ data: res.data, loading: false, error: res.error });
    return res;
  }, [supabase, fn]);
  return [invoke, state];
}

export function useEdgeFunction(name) {
  const supabase = useSupabaseClient();
  const [state, setState] = useState({ data: null, loading: false, error: null });
  const invoke = useCallback(async (body) => {
    setState({ data: null, loading: true, error: null });
    const res = await supabase.functions.invoke(name, { body });
    setState({ data: res.data, loading: false, error: res.error });
    return res;
  }, [supabase, name]);
  return [invoke, state];
}

/* ----------------------------------------------------------------------------
 * Storage
 * --------------------------------------------------------------------------*/
export function useStorage(bucket) {
  const supabase = useSupabaseClient();
  const [files, setFiles] = useState([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    setLoading(true);
    const { data } = await supabase.storage.from(bucket).list();
    setFiles(data || []);
    setLoading(false);
  }, [supabase, bucket]);

  useEffect(() => { refresh(); }, [refresh]);

  const upload = useCallback(async (path, file, opts) => { const r = await supabase.storage.from(bucket).upload(path, file, opts); await refresh(); return r; }, [supabase, bucket, refresh]);
  const remove = useCallback(async (paths) => { const r = await supabase.storage.from(bucket).remove(paths); await refresh(); return r; }, [supabase, bucket, refresh]);
  const publicUrl = useCallback((path) => supabase.storage.from(bucket).getPublicUrl(path).data.publicUrl, [supabase, bucket]);

  return { files, loading, refresh, upload, remove, publicUrl };
}

export default SupabaseProvider;
