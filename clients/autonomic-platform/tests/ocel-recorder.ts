/**
 * OCEL 2.0 Shape-A builder (v26.7.6 evidence pass).
 *
 * Emits exactly the wire shape `wasm4pm_compat::ocel::OCEL` parses:
 *   {
 *     "eventTypes":  [{ name, attributes: [{ name, type }] }],
 *     "objectTypes": [{ name, attributes: [{ name, type }] }],
 *     "events":  [{ id, type, time, attributes: [{ name, value }], relationships: [{ objectId, qualifier }] }],
 *     "objects": [{ id, type, attributes, relationships }]
 *   }
 *
 * Times are ISO-8601 UTC Z strings. This is evidence time (OCEL event
 * timestamps), never a hash input — the no-wall-clock-in-hash-paths
 * invariant applies to receipts, not to the evidence log.
 */

export interface OcelRelationship {
  objectId: string;
  qualifier: string;
}

export interface OcelEventInput {
  id?: string;
  type: string;
  time?: string;
  attributes?: Record<string, unknown>;
  relationships?: OcelRelationship[];
}

export interface OcelObjectInput {
  id: string;
  type: string;
  attributes?: Record<string, unknown>;
  relationships?: OcelRelationship[];
}

interface AttrEntry {
  name: string;
  value: string | number | boolean;
}

/**
 * OCEL 2.0 object attribute values are timestamped (wire shape
 * `wasm4pm_compat::ocel::OCELObjectAttribute { name, value, time }`);
 * event attributes are not (the event's own `time` covers them).
 */
interface ObjAttrEntry extends AttrEntry {
  time: string;
}

interface TypeDecl {
  name: string;
  attributes: { name: string; type: string }[];
}

interface OcelEvent {
  id: string;
  type: string;
  time: string;
  attributes: AttrEntry[];
  relationships: OcelRelationship[];
}

interface OcelObject {
  id: string;
  type: string;
  attributes: ObjAttrEntry[];
  relationships: OcelRelationship[];
}

function ocelValueType(v: unknown): string {
  if (typeof v === 'boolean') return 'boolean';
  if (typeof v === 'number') return Number.isInteger(v) ? 'integer' : 'float';
  return 'string';
}

function normalizeValue(v: unknown): string | number | boolean {
  if (typeof v === 'string' || typeof v === 'number' || typeof v === 'boolean') return v;
  return JSON.stringify(v);
}

function toAttrEntries(attrs: Record<string, unknown> | AttrEntry[] | undefined): AttrEntry[] {
  if (!attrs) return [];
  if (Array.isArray(attrs)) {
    return attrs.map((a) => ({ name: a.name, value: normalizeValue(a.value) }));
  }
  return Object.entries(attrs).map(([name, value]) => ({ name, value: normalizeValue(value) }));
}

function toObjAttrEntries(
  attrs: Record<string, unknown> | (AttrEntry & { time?: string })[] | undefined,
  fallbackTime: string,
): ObjAttrEntry[] {
  if (!attrs) return [];
  if (Array.isArray(attrs)) {
    return attrs.map((a) => ({
      name: a.name,
      value: normalizeValue(a.value),
      time: a.time ?? fallbackTime,
    }));
  }
  return Object.entries(attrs).map(([name, value]) => ({
    name,
    value: normalizeValue(value),
    time: fallbackTime,
  }));
}

export class OcelRecorder {
  private eventTypes = new Map<string, Map<string, string>>();
  private objectTypes = new Map<string, Map<string, string>>();
  private events: OcelEvent[] = [];
  private objects: OcelObject[] = [];
  private eventSeq = 0;
  readonly runId: string;
  readonly releaseId: string;

  constructor(opts: { runId: string; releaseId: string }) {
    this.runId = opts.runId;
    this.releaseId = opts.releaseId;
  }

  addEventType(name: string, attributes: AttrEntry[] = []): void {
    const decl = this.eventTypes.get(name) ?? new Map<string, string>();
    for (const a of attributes) {
      if (!decl.has(a.name)) decl.set(a.name, ocelValueType(a.value));
    }
    this.eventTypes.set(name, decl);
  }

  addObjectType(name: string, attributes: AttrEntry[] = []): void {
    const decl = this.objectTypes.get(name) ?? new Map<string, string>();
    for (const a of attributes) {
      if (!decl.has(a.name)) decl.set(a.name, ocelValueType(a.value));
    }
    this.objectTypes.set(name, decl);
  }

  /** Add an event; id auto-increments, time defaults to now (UTC Z). */
  addEvent(input: OcelEventInput): OcelEvent {
    this.eventSeq += 1;
    const attributes = toAttrEntries({
      run_id: this.runId,
      release_id: this.releaseId,
      actor: 'playwright',
      ...input.attributes,
    });
    const ev: OcelEvent = {
      id: input.id ?? `pw_e${this.eventSeq}`,
      type: input.type,
      time: input.time ?? new Date().toISOString(),
      attributes,
      relationships: input.relationships ?? [],
    };
    this.addEventType(ev.type, ev.attributes);
    this.events.push(ev);
    return ev;
  }

  addObject(input: OcelObjectInput): OcelObject {
    // Observation instant for the OCEL-v2 timestamped object attributes.
    const observedAt = new Date().toISOString();
    const obj: OcelObject = {
      id: input.id,
      type: input.type,
      attributes: toObjAttrEntries(input.attributes, observedAt),
      relationships: input.relationships ?? [],
    };
    this.addObjectType(obj.type, obj.attributes);
    this.objects.push(obj);
    return obj;
  }

  /**
   * Merge the CLI driver's intermediate log (already Shape-A event/object
   * entries, emitted by tests/run-evidence-pass.mjs) so the browser pass
   * produces ONE final OCEL log.
   */
  merge(intermediate: { events?: OcelEvent[]; objects?: OcelObject[] }): void {
    for (const ev of intermediate.events ?? []) {
      const entry: OcelEvent = {
        id: ev.id,
        type: ev.type,
        time: ev.time,
        attributes: toAttrEntries(ev.attributes),
        relationships: ev.relationships ?? [],
      };
      this.addEventType(entry.type, entry.attributes);
      this.events.push(entry);
    }
    for (const obj of intermediate.objects ?? []) {
      const entry: OcelObject = {
        id: obj.id,
        type: obj.type,
        // Driver entries carry their own attribute `time`; stamp merge time
        // only if an entry predates the driver-side fix.
        attributes: toObjAttrEntries(obj.attributes, new Date().toISOString()),
        relationships: obj.relationships ?? [],
      };
      this.addObjectType(entry.type, entry.attributes);
      this.objects.push(entry);
    }
  }

  toJSON(): {
    eventTypes: TypeDecl[];
    objectTypes: TypeDecl[];
    events: OcelEvent[];
    objects: OcelObject[];
  } {
    const declList = (m: Map<string, Map<string, string>>): TypeDecl[] =>
      [...m.entries()].map(([name, attrs]) => ({
        name,
        attributes: [...attrs.entries()].map(([n, t]) => ({ name: n, type: t })),
      }));
    // Stable evidence ordering: ascending event time (driver ran first).
    const events = [...this.events].sort((a, b) => a.time.localeCompare(b.time));
    return {
      eventTypes: declList(this.eventTypes),
      objectTypes: declList(this.objectTypes),
      events,
      objects: this.objects,
    };
  }

  async save(path: string): Promise<void> {
    const fs = await import('node:fs');
    const pathMod = await import('node:path');
    fs.mkdirSync(pathMod.dirname(path), { recursive: true });
    fs.writeFileSync(path, JSON.stringify(this.toJSON(), null, 2) + '\n');
  }
}
