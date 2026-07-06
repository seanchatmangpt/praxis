//! example — shows the #[verb] proc-macro, fn()-pointer distributed_slice
//! registration, and Box::leak for 'static clap strings.
//!
//! The filename stem (`example`) becomes the noun; each `#[verb]` function
//! becomes a sub-command under that noun.

use clap_noun_verb::error::Result;
use clap_noun_verb_macros::verb;

/// Show an object by ID.
#[verb]
pub fn show(
    #[arg(help = "Object identifier")] id: String,
    #[arg(short, help = "Enable verbose output")] verbose: bool,
) -> Result<()> {
    println!("show id={id} verbose={verbose}");
    Ok(())
}

// The #[verb] macro expands to a linkme distributed_slice registration roughly
// equivalent to the following (shown here for documentation purposes only):
//
// use linkme::distributed_slice;
// use clap_noun_verb::cli::registry::__VERB_REGISTRY;
//
// #[distributed_slice(__VERB_REGISTRY)]
// static _REGISTER_SHOW: fn(&mut clap_noun_verb::CommandRegistry) = |registry| {
//     // Box::leak is required so the clap strings live for 'static
//     let noun: &'static str = Box::leak("example".to_owned().into_boxed_str());
//     let verb: &'static str = Box::leak("show".to_owned().into_boxed_str());
//     registry.register(noun, verb, show_command());
// };
