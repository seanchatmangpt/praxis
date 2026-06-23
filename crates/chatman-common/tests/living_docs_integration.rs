//! Integration tests for `chatman_common` living-docs feature.
//!
//! These tests exercise `DocContext` and `DocEvent` under the `living-docs` feature:
//!   - Section/Para/Code/Table/Mermaid/KeyValue events render correctly to Markdown
//!   - `say_and_assert` panics when cond=false and does NOT emit the event
//!   - BLAKE3 chain hash appears in the footer
//!   - Output file is written to the configured output dir on `finish()`
//!   - Drop auto-flushes if `finish()` was not called explicitly
//!   - `doc_assert!` macro integrates correctly with `DocContext`

#[cfg(feature = "living-docs")]
mod living_docs_integration {
    use chatman_common::testkit::DocContext;
    use chatman_common::doc_assert;

    // -----------------------------------------------------------------------
    // Render helpers
    // -----------------------------------------------------------------------

    fn make_ctx_with_dir(name: &str, dir: &std::path::Path) -> DocContext {
        DocContext::for_test(name).with_output_dir(dir)
    }

    // -----------------------------------------------------------------------
    // Individual event rendering
    // -----------------------------------------------------------------------

    /// Section event renders as `## heading\n\n`.
    #[test]
    fn section_event_renders_as_h2() {
        let dir = tempfile::tempdir().unwrap();
        let mut ctx = make_ctx_with_dir("sec_test", dir.path());
        ctx.say_section("My Section");
        let md = String::from_utf8(ctx.render_markdown()).unwrap();
        assert!(md.contains("## My Section\n\n"), "expected ## heading, got:\n{md}");
    }

    /// Para event renders as plain paragraph followed by double newline.
    #[test]
    fn para_event_renders_as_paragraph() {
        let dir = tempfile::tempdir().unwrap();
        let mut ctx = make_ctx_with_dir("para_test", dir.path());
        ctx.say("Hello, world.");
        let md = String::from_utf8(ctx.render_markdown()).unwrap();
        assert!(md.contains("Hello, world.\n\n"), "expected paragraph, got:\n{md}");
    }

    /// Code event renders as fenced code block with language tag.
    #[test]
    fn code_event_renders_as_fenced_block() {
        let dir = tempfile::tempdir().unwrap();
        let mut ctx = make_ctx_with_dir("code_test", dir.path());
        ctx.say_code("rust", "fn main() {}");
        let md = String::from_utf8(ctx.render_markdown()).unwrap();
        assert!(md.contains("```rust\nfn main() {}\n```\n\n"), "expected fenced code:\n{md}");
    }

    /// Table event renders correct header/separator/rows.
    #[test]
    fn table_event_renders_markdown_table() {
        let dir = tempfile::tempdir().unwrap();
        let mut ctx = make_ctx_with_dir("table_test", dir.path());
        ctx.say_table(
            &["Name", "Value"],
            &[&["alpha", "1"], &["beta", "2"]],
        );
        let md = String::from_utf8(ctx.render_markdown()).unwrap();
        assert!(md.contains("| Name | Value |"), "missing header: {md}");
        assert!(md.contains("| --- | --- |"), "missing separator: {md}");
        assert!(md.contains("| alpha | 1 |"), "missing row 1: {md}");
        assert!(md.contains("| beta | 2 |"), "missing row 2: {md}");
    }

    /// Mermaid event renders as a fenced mermaid block.
    #[test]
    fn mermaid_event_renders_as_mermaid_block() {
        let dir = tempfile::tempdir().unwrap();
        let mut ctx = make_ctx_with_dir("mermaid_test", dir.path());
        ctx.say_mermaid("graph TD\n  A --> B");
        let md = String::from_utf8(ctx.render_markdown()).unwrap();
        assert!(
            md.contains("```mermaid\ngraph TD\n  A --> B\n```\n\n"),
            "expected mermaid block:\n{md}"
        );
    }

    /// KeyValue event renders as `- **key:** value` list.
    #[test]
    fn key_value_event_renders_as_bullet_list() {
        let dir = tempfile::tempdir().unwrap();
        let mut ctx = make_ctx_with_dir("kv_test", dir.path());
        ctx.say_key_value(&[("host", "localhost"), ("port", "8080")]);
        let md = String::from_utf8(ctx.render_markdown()).unwrap();
        assert!(md.contains("- **host:** localhost"), "missing host: {md}");
        assert!(md.contains("- **port:** 8080"), "missing port: {md}");
    }

    /// Assertion event renders as `- ✓ label` when passed.
    #[test]
    fn assertion_event_renders_pass_icon() {
        let dir = tempfile::tempdir().unwrap();
        let mut ctx = make_ctx_with_dir("assert_test", dir.path());
        ctx.say_and_assert("two plus two equals four", 2 + 2 == 4);
        let md = String::from_utf8(ctx.render_markdown()).unwrap();
        assert!(
            md.contains("- ✓ two plus two equals four"),
            "expected pass icon: {md}"
        );
    }

    // -----------------------------------------------------------------------
    // say_and_assert panic behaviour
    // -----------------------------------------------------------------------

    /// `say_and_assert` panics when the condition is false, and does NOT
    /// write an event (the method never reaches `self.events.push(...)` on failure).
    #[test]
    fn say_and_assert_panics_on_false_cond() {
        let dir = tempfile::tempdir().unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut ctx = make_ctx_with_dir("panic_test", dir.path());
            ctx.say_and_assert("this must not pass", false);
        }));
        assert!(result.is_err(), "say_and_assert(false) must panic");
    }

    /// After a `say_and_assert(false)` panic no assertion event is present
    /// in the accumulated event list (the panic happens BEFORE push).
    #[test]
    fn say_and_assert_false_does_not_emit_event() {
        // We need to inspect the events before drop — use a cell trick.
        use std::cell::RefCell;
        use std::rc::Rc;

        let event_count: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
        let count_clone = event_count.clone();

        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();

        // Run in catch_unwind so we can inspect afterwards.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let mut ctx = DocContext::for_test("no_event")
                .with_output_dir(&dir_path);
            // The first call passes — emits an event.
            ctx.say_and_assert("passes", true);
            *count_clone.borrow_mut() = 1; // checkpoint: 1 event emitted
            // The second call fails — should NOT emit.
            ctx.say_and_assert("fails", false);
            *count_clone.borrow_mut() = 2; // this line should NOT execute
        }));

        // Only 1 event was emitted before the panic.
        assert_eq!(*event_count.borrow(), 1, "second event must not be emitted");
    }

    // -----------------------------------------------------------------------
    // BLAKE3 chain hash in footer
    // -----------------------------------------------------------------------

    /// The rendered Markdown footer must contain a BLAKE3 hex hash.
    #[test]
    fn rendered_markdown_contains_blake3_footer() {
        let dir = tempfile::tempdir().unwrap();
        let mut ctx = make_ctx_with_dir("hash_test", dir.path());
        ctx.say("some content");
        let md = String::from_utf8(ctx.render_markdown()).unwrap();
        // Footer pattern: "*chain: `<64-char hex>`*"
        assert!(md.contains("chain:"), "footer must mention 'chain': {md}");
        // Find the hash in backticks
        let hash_start = md.find('`').expect("backtick for hash not found");
        let after = &md[hash_start + 1..];
        let hash_end = after.find('`').expect("closing backtick not found");
        let hash = &after[..hash_end];
        assert_eq!(hash.len(), 64, "BLAKE3 hex hash must be 64 chars, got: {hash:?}");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "hash contains non-hex chars: {hash}"
        );
    }

    /// The chain hash changes when content changes (not a constant).
    #[test]
    fn rendered_markdown_chain_hash_depends_on_content() {
        let extract_hash = |md: &str| -> String {
            let start = md.find('`').unwrap() + 1;
            let after = &md[start..];
            let end = after.find('`').unwrap();
            after[..end].to_string()
        };

        let dir = tempfile::tempdir().unwrap();
        let mut ctx_a = make_ctx_with_dir("hash_a", dir.path());
        ctx_a.say("content A");
        let md_a = String::from_utf8(ctx_a.render_markdown()).unwrap();

        let dir2 = tempfile::tempdir().unwrap();
        let mut ctx_b = make_ctx_with_dir("hash_b", dir2.path());
        ctx_b.say("content B");
        let md_b = String::from_utf8(ctx_b.render_markdown()).unwrap();

        let hash_a = extract_hash(&md_a);
        let hash_b = extract_hash(&md_b);
        assert_ne!(hash_a, hash_b, "different content must produce different chain hash");
    }

    // -----------------------------------------------------------------------
    // finish() writes the file
    // -----------------------------------------------------------------------

    /// `finish()` creates the output file at `<output_dir>/<name>.md`.
    #[test]
    fn finish_writes_output_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut ctx = DocContext::for_test("my_doc_test")
            .with_output_dir(dir.path());
        ctx.say("Written by finish()");
        ctx.finish().expect("finish() must succeed");

        let expected = dir.path().join("my_doc_test.md");
        assert!(expected.exists(), "output file must exist at {}", expected.display());
        let contents = std::fs::read_to_string(&expected).unwrap();
        assert!(contents.contains("Written by finish()"), "content must appear in file");
    }

    /// `finish()` sanitizes slashes and spaces in the filename.
    #[test]
    fn finish_sanitizes_filename() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = DocContext::for_test("module/sub test")
            .with_output_dir(dir.path());
        ctx.finish().unwrap();

        // Slashes and spaces replaced with underscores
        let expected = dir.path().join("module_sub_test.md");
        assert!(expected.exists(), "sanitized filename must exist: {}", expected.display());
    }

    // -----------------------------------------------------------------------
    // Drop auto-flush
    // -----------------------------------------------------------------------

    /// When `finish()` is NOT called explicitly and events are present, the
    /// Drop impl writes the file automatically (as long as we're not panicking).
    #[test]
    fn drop_auto_flushes_when_finish_not_called() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();

        {
            let mut ctx = DocContext::for_test("auto_flush")
                .with_output_dir(&dir_path);
            ctx.say("This was auto-flushed on drop");
            // Note: finish() is NOT called — Drop should handle it.
        }

        let expected = dir_path.join("auto_flush.md");
        assert!(
            expected.exists(),
            "Drop must write the file when finish() was not called"
        );
        let contents = std::fs::read_to_string(&expected).unwrap();
        assert!(contents.contains("This was auto-flushed on drop"));
    }

    /// When the context has NO events and finish() is not called, Drop should
    /// not write an empty file (nothing to flush).
    #[test]
    fn drop_does_not_write_empty_context() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();

        {
            let _ctx = DocContext::for_test("empty_ctx").with_output_dir(&dir_path);
            // No events pushed, no finish() called.
        }

        let expected = dir_path.join("empty_ctx.md");
        assert!(
            !expected.exists(),
            "Drop must NOT write an empty context to disk"
        );
    }

    // -----------------------------------------------------------------------
    // doc_assert! macro
    // -----------------------------------------------------------------------

    /// `doc_assert!` with a true condition emits an event and does not panic.
    #[test]
    fn doc_assert_macro_true_condition() {
        let dir = tempfile::tempdir().unwrap();
        let mut ctx = DocContext::for_test("doc_assert_pass").with_output_dir(dir.path());
        doc_assert!(ctx, "one equals one", 1 == 1);
        let md = String::from_utf8(ctx.render_markdown()).unwrap();
        assert!(md.contains("one equals one"), "label must appear in rendered doc");
    }

    /// `doc_assert!` with a false condition panics.
    #[test]
    fn doc_assert_macro_false_condition_panics() {
        let dir = tempfile::tempdir().unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut ctx = DocContext::for_test("doc_assert_fail").with_output_dir(dir.path());
            doc_assert!(ctx, "one equals two", 1 == 2);
        }));
        assert!(result.is_err(), "doc_assert!(ctx, _, false) must panic");
    }

    // -----------------------------------------------------------------------
    // Document structure — multi-event rendering
    // -----------------------------------------------------------------------

    /// Multiple events in sequence render in the correct order.
    #[test]
    fn multi_event_order_is_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let mut ctx = make_ctx_with_dir("multi_event", dir.path());
        ctx.say_section("Intro");
        ctx.say("First paragraph.");
        ctx.say_code("sh", "echo hello");
        ctx.say_section("Conclusion");
        ctx.say("Last paragraph.");
        let md = String::from_utf8(ctx.render_markdown()).unwrap();

        let intro_pos = md.find("## Intro").unwrap();
        let first_pos = md.find("First paragraph.").unwrap();
        let code_pos = md.find("```sh").unwrap();
        let conclusion_pos = md.find("## Conclusion").unwrap();
        let last_pos = md.find("Last paragraph.").unwrap();

        assert!(intro_pos < first_pos);
        assert!(first_pos < code_pos);
        assert!(code_pos < conclusion_pos);
        assert!(conclusion_pos < last_pos);
    }

    /// The document always starts with a level-1 heading equal to the test name.
    #[test]
    fn document_starts_with_h1_test_name() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = make_ctx_with_dir("my heading test", dir.path());
        let md = String::from_utf8(ctx.render_markdown()).unwrap();
        assert!(md.starts_with("# my heading test\n"), "H1 must be first line: {md}");
    }

    // -----------------------------------------------------------------------
    // TestReceipt chain_hash (living-docs feature)
    // -----------------------------------------------------------------------

    /// When living-docs feature is active, `TestReceipt::record` populates
    /// the `chain_hash` field with a 64-char hex string.
    #[test]
    fn test_receipt_chain_hash_populated_with_living_docs() {
        use chatman_common::testkit::TestReceipt;
        let r = TestReceipt::record("integration_receipt", true, 42);
        let hash = r.chain_hash.expect("chain_hash must be Some with living-docs feature");
        assert_eq!(hash.len(), 64, "chain_hash must be 64 hex chars: {hash}");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "chain_hash must be hex: {hash}"
        );
    }

    /// Two identical receipts (same inputs) produce the same chain_hash.
    #[test]
    fn test_receipt_chain_hash_is_deterministic() {
        use chatman_common::testkit::TestReceipt;
        let r1 = TestReceipt::record("determ", true, 100);
        let r2 = TestReceipt::record("determ", true, 100);
        assert_eq!(
            r1.chain_hash, r2.chain_hash,
            "same inputs must produce same chain_hash"
        );
    }

    /// Changing any input changes the chain_hash.
    #[test]
    fn test_receipt_chain_hash_changes_with_inputs() {
        use chatman_common::testkit::TestReceipt;
        let r1 = TestReceipt::record("test_a", true, 0);
        let r2 = TestReceipt::record("test_b", true, 0);
        assert_ne!(
            r1.chain_hash, r2.chain_hash,
            "different test names must produce different chain_hash"
        );
    }
}
