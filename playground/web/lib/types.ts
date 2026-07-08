/**
 * Type definitions matching the Praxis WASM DTO layer.
 * These mirror src/dto.rs structures for the GraphLaw engine.
 */

export type Status =
  | "ADMITTED"
  | "REFUSED"
  | "UNSUPPORTED"
  | "REPLAY_MISMATCH"
  | "HASH_MISMATCH"
  | "PROFILE_NOT_ADMITTED";

export interface DialectResult {
  dialect: string;
  status: Status;
  detail: string;
  triples_out: number;
}

export type HookVerdict = "Fired" | "NotFired" | "Gated";

export interface TriggerDiagnostic {
  hook_iri: string;
  conforms: boolean;
  details: DiagnosticDetail[];
}

export interface DiagnosticDetail {
  focus_node?: string;
  result_path?: string;
  value?: string;
  severity?: string;
  message: string;
}

export interface HookVerdictRecord {
  hook_id: number;
  hook_iri: string;
  hook_name: string;
  condition_kind: string;
  condition_hash: string;
  verdict: HookVerdict;
  effect: string;
  action_iri?: string;
  diagnostics?: TriggerDiagnostic;
  delta_hash?: string;
  idempotency_key?: string;
}

export interface HookReceipt {
  hook_name: string;
  delta_hash: string;
  idempotency_key: string;
  delta_quads: string;
}

export interface HookRunResult {
  status: Status;
  verdicts: HookVerdictRecord[];
  receipts: HookReceipt[];
  schedule: string[];
}

export interface ReplayResult {
  status: Status;
  first_hash: string;
  second_hash: string;
}

export interface PlaygroundResult {
  graph_hash: string;
  profile_hash: string;
  dialects: DialectResult[];
  hooks: HookRunResult;
  replay: ReplayResult;
  hash_algorithms: Record<string, string>;
}

export interface EditorFile {
  name: string;
  content: string;
  language: "turtle" | "sparql" | "shacl" | "shex" | "n3";
}

export interface AppState {
  files: EditorFile[];
  activeFileIndex: number;
  result: PlaygroundResult | null;
  isLoading: boolean;
  error: string | null;
}
