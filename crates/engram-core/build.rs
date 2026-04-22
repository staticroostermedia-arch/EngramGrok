//! Build script for engram-core.
//! Compiles the ProvLog Cap'n Proto schema into Rust bindings.
//! This makes engram-core fully self-contained — no dependency on CodeLand/leg_core.

fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file("schema/provlog.capnp")
        .run()
        .expect("capnp schema compilation failed");
}
