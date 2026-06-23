//! Language Server Protocol (LSP) server backend for `{{project-name}}`.
//!
//! Enforces typestate-driven configurations and `RulePackServer` structures
//! adhering to the Post-Chatman Equation A = \mu(O^*).

use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::Mutex;

use lsp_max::lsp_types_max::*;
use lsp_max::rule_pack_server::{RulePackServer, ValidatedRulePackSet, WorkspaceIndex, RulePack};
use lsp_max::ast::AutoLspAdapter;
use lsp_max::primitives::{SpcMonitor, RuleLatencyTracker, CircuitBreaker};
use lsp_max::{Client, LanguageServer, async_trait};

/// The `AppLspServer` structure implementing `RulePackServer` from `lsp-max`.
///
/// This server skeleton aligns with the Post-Chatman equation, providing scaffolding for
/// rule-pack verification, grammar matching, and telemetry monitoring.
pub struct AppLspServer {
    client: Client,
    packs: ValidatedRulePackSet,
    adapter: AutoLspAdapter,
    index: WorkspaceIndex,
    spc_monitor: std::sync::Mutex<SpcMonitor>,
    latency_trackers: Arc<DashMap<String, RuleLatencyTracker>>,
    rule_circuit_breaker: Arc<Mutex<CircuitBreaker>>,
}

impl AppLspServer {
    /// Create a new `AppLspServer` instance bound to the given client.
    pub fn new(client: Client) -> Self {
        // Post-Chatman Equation: A = \mu(O^*)
        // Scaffolding for loading rules from file or default rule pack.
        let rules = Vec::new();
        let rule_pack = RulePack {
            id: "{{project-name}}-pack".to_string(),
            version: "1.0.0".to_string(),
            rules,
            depends_on: Vec::new(),
        };

        let packs = ValidatedRulePackSet::new(&[rule_pack]).unwrap_or_default();

        Self {
            client,
            packs,
            adapter: AutoLspAdapter::new_default(),
            index: WorkspaceIndex::new(),
            spc_monitor: std::sync::Mutex::new(SpcMonitor::default()),
            latency_trackers: Arc::new(DashMap::new()),
            rule_circuit_breaker: Arc::new(Mutex::new(CircuitBreaker::default())),
        }
    }
}

impl RulePackServer for AppLspServer {
    fn rule_packs(&self) -> &ValidatedRulePackSet {
        &self.packs
    }

    fn grammar(&self) -> tree_sitter::Language {
        // Return tree-sitter-rust language as standard
        tree_sitter_rust::LANGUAGE.into()
    }

    fn server_name(&self) -> &'static str {
        "{{project-name}}-lsp"
    }

    fn client(&self) -> &Client {
        &self.client
    }

    fn adapter(&self) -> &AutoLspAdapter {
        &self.adapter
    }

    fn workspace_index(&self) -> Option<&WorkspaceIndex> {
        Some(&self.index)
    }

    fn spc_monitor(&self) -> Option<&std::sync::Mutex<SpcMonitor>> {
        Some(&self.spc_monitor)
    }

    fn latency_trackers(&self) -> Option<&Arc<DashMap<String, RuleLatencyTracker>>> {
        Some(&self.latency_trackers)
    }

    fn rule_circuit_breaker(&self) -> Option<&Arc<Mutex<CircuitBreaker>>> {
        Some(&self.rule_circuit_breaker)
    }
}

#[async_trait]
impl LanguageServer for AppLspServer {
    async fn initialize(&self, _: InitializeParams) -> lsp_max::jsonrpc::Result<InitializeResult> {
        use lsp_max::rule_pack_server::RulePackServer;
        Ok(self.build_initialize_result())
    }

    async fn shutdown(&self) -> lsp_max::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        use lsp_max::rule_pack_server::RulePackServer;
        self.handle_did_open(params).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        use lsp_max::rule_pack_server::RulePackServer;
        self.handle_did_change(params).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        use lsp_max::rule_pack_server::RulePackServer;
        self.handle_did_close(params);
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        use lsp_max::rule_pack_server::RulePackServer;
        let uri = &params.text_document.uri;

        // Optimization: only re-analyze if workspace index exists
        if let Some(index) = self.workspace_index() {
            if let Some(doc) = index.get(uri.as_str()) {
                // Avoid cloning document content: pass by reference
                self.publish_findings_classified(uri, &doc.content).await;
            }
        }
    }
}
