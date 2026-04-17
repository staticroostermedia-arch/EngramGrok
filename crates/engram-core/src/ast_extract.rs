//! AST-aware Rust source extraction for Engram ingestion.
//!
//! Parses a `.rs` file with `syn` and produces one [`AstItem`] per public
//! item (function, struct, enum, trait, impl block). Each item becomes one `.leg`
//! memory block with:
//!
//! - **Concept name** = `{file_stem}__{kind}__{item_name}` (e.g. `encode__fn__from_text`)
//! - **Embed label** = doc comment + signature — the semantic distillation for the embedding
//! - **Full source** = complete item source for the provlog payload
//!
//! # Why this matters
//!
//! A full Rust source file is 500–5000 tokens. Embedding models cap at 512–2048 tokens.
//!
//! The correct architecture:
//! ```text
//! q[0..768].re  = embed("encode__fn__from_text: Converts text → HolographicBlock")
//! provlog       = "pub fn from_text(text: &str) -> Leg3Pointer { ... }"
//! ```

use syn::{
    visit::Visit,
    ItemEnum, ItemFn, ItemImpl, ItemStruct, ItemTrait,
};

/// The kind of AST item extracted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
}

impl ItemKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ItemKind::Function => "fn",
            ItemKind::Struct   => "struct",
            ItemKind::Enum     => "enum",
            ItemKind::Trait    => "trait",
            ItemKind::Impl     => "impl",
        }
    }
}

/// A single extracted item from a Rust source file.
#[derive(Debug, Clone)]
pub struct AstItem {
    /// Short name of the item (e.g. `"from_text"`, `"HolographicBlock"`).
    pub name: String,
    /// What kind of item it is.
    pub kind: ItemKind,
    /// The doc comment lines joined with spaces. Empty if none.
    pub doc_comment: String,
    /// Human-readable signature: `fn name(args) -> Ret` or `struct Name { fields }`.
    pub signature: String,
    /// The complete source lines of this item for storage in the provlog.
    pub full_source: String,
    /// Concept name for the `.leg` block: `{file_stem}__{kind}__{name}`.
    pub concept: String,
}

impl AstItem {
    /// Build the label sent to the embedding model.
    ///
    /// Format: `{concept}: {doc_comment} — {signature}`
    ///
    /// Always < 200 tokens — comfortably within any embedding model's budget.
    pub fn embed_label(&self) -> String {
        if self.doc_comment.is_empty() {
            format!("{}: {}", self.concept, self.signature)
        } else {
            format!("{}: {} — {}", self.concept, self.doc_comment, self.signature)
        }
    }
}

// ── Visitor ──────────────────────────────────────────────────────────────────

struct Extractor<'a> {
    file_stem:  &'a str,
    source_lines: Vec<&'a str>,
    pub items:  Vec<AstItem>,
}

impl<'a> Extractor<'a> {
    fn new(file_stem: &'a str, source: &'a str) -> Self {
        Self {
            file_stem,
            source_lines: source.lines().collect(),
            items: Vec::new(),
        }
    }

    fn concept_name(&self, kind: &ItemKind, name: &str) -> String {
        format!("{}__{}__{}", self.file_stem, kind.as_str(), name)
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
    }

    /// Extract doc-comment text from syn Attributes.
    fn extract_doc(attrs: &[syn::Attribute]) -> String {
        let mut parts = Vec::new();
        for attr in attrs {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(nv) = &attr.meta {
                    if let syn::Expr::Lit(expr) = &nv.value {
                        if let syn::Lit::Str(s) = &expr.lit {
                            let text = s.value();
                            let trimmed = text.trim().to_string();
                            if !trimmed.is_empty() {
                                parts.push(trimmed);
                            }
                        }
                    }
                }
            }
        }
        parts.join(" ")
    }

    /// Extract source lines for a syn item using its byte span.
    /// syn's span gives 1-based line numbers.
    fn lines_for_span(&self, start_line: usize, end_line: usize) -> String {
        let lo = start_line.saturating_sub(1);
        let hi = end_line.min(self.source_lines.len());
        self.source_lines[lo..hi].join("\n")
    }
}

/// Helper: build a human-readable parameter list from syn inputs.
fn format_inputs(inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>) -> String {
    inputs.iter().map(|arg| match arg {
        syn::FnArg::Receiver(r) => {
            if r.reference.is_some() {
                if r.mutability.is_some() { "&mut self".to_string() }
                else { "&self".to_string() }
            } else {
                "self".to_string()
            }
        }
        syn::FnArg::Typed(t) => {
            // Extract just the parameter name (lhs of the pattern)
            let name = match t.pat.as_ref() {
                syn::Pat::Ident(i) => i.ident.to_string(),
                _ => "_".to_string(),
            };
            name
        }
    }).collect::<Vec<_>>().join(", ")
}

impl<'a, 'ast> Visit<'ast> for Extractor<'a> {
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        // Only capture pub functions
        if !matches!(i.vis, syn::Visibility::Public(_)) {
            syn::visit::visit_item_fn(self, i);
            return;
        }

        let name        = i.sig.ident.to_string();
        let doc_comment = Self::extract_doc(&i.attrs);
        let params      = format_inputs(&i.sig.inputs);
        let ret = match &i.sig.output {
            syn::ReturnType::Default  => String::new(),
            syn::ReturnType::Type(_, t) => format!(" -> {}", type_name(t)),
        };
        let signature   = format!("fn {name}({params}){ret}");
        let span        = i.sig.ident.span();
        let full_source = self.lines_for_span(span.start().line, span.end().line + 50);
        let kind        = ItemKind::Function;
        let concept     = self.concept_name(&kind, &name);

        self.items.push(AstItem { name, kind, doc_comment, signature, full_source, concept });
        syn::visit::visit_item_fn(self, i);
    }

    fn visit_item_struct(&mut self, i: &'ast ItemStruct) {
        if !matches!(i.vis, syn::Visibility::Public(_)) {
            syn::visit::visit_item_struct(self, i);
            return;
        }

        let name        = i.ident.to_string();
        let doc_comment = Self::extract_doc(&i.attrs);
        let fields: Vec<String> = i.fields.iter()
            .filter_map(|f| f.ident.as_ref().map(|id| id.to_string()))
            .take(6)  // cap at 6 fields in the signature label
            .collect();
        let ellipsis    = if i.fields.len() > 6 { " .." } else { "" };
        let signature   = if fields.is_empty() {
            format!("struct {name}")
        } else {
            format!("struct {name} {{ {}{ellipsis} }}", fields.join(", "))
        };
        let span        = i.ident.span();
        let full_source = self.lines_for_span(span.start().line, span.end().line + 30);
        let kind        = ItemKind::Struct;
        let concept     = self.concept_name(&kind, &name);

        self.items.push(AstItem { name, kind, doc_comment, signature, full_source, concept });
        syn::visit::visit_item_struct(self, i);
    }

    fn visit_item_enum(&mut self, i: &'ast ItemEnum) {
        if !matches!(i.vis, syn::Visibility::Public(_)) {
            syn::visit::visit_item_enum(self, i);
            return;
        }

        let name        = i.ident.to_string();
        let doc_comment = Self::extract_doc(&i.attrs);
        let variants: Vec<String> = i.variants.iter()
            .map(|v| v.ident.to_string())
            .take(8)
            .collect();
        let signature   = format!("enum {name} {{ {} }}", variants.join(", "));
        let span        = i.ident.span();
        let full_source = self.lines_for_span(span.start().line, span.end().line + 20);
        let kind        = ItemKind::Enum;
        let concept     = self.concept_name(&kind, &name);

        self.items.push(AstItem { name, kind, doc_comment, signature, full_source, concept });
        syn::visit::visit_item_enum(self, i);
    }

    fn visit_item_trait(&mut self, i: &'ast ItemTrait) {
        if !matches!(i.vis, syn::Visibility::Public(_)) {
            syn::visit::visit_item_trait(self, i);
            return;
        }

        let name        = i.ident.to_string();
        let doc_comment = Self::extract_doc(&i.attrs);
        let methods: Vec<String> = i.items.iter().filter_map(|item| {
            if let syn::TraitItem::Fn(m) = item { Some(format!("fn {}", m.sig.ident)) }
            else { None }
        }).collect();
        let signature   = format!("trait {name} {{ {} }}", methods.join(", "));
        let span        = i.ident.span();
        let full_source = self.lines_for_span(span.start().line, span.end().line + 50);
        let kind        = ItemKind::Trait;
        let concept     = self.concept_name(&kind, &name);

        self.items.push(AstItem { name, kind, doc_comment, signature, full_source, concept });
        syn::visit::visit_item_trait(self, i);
    }

    fn visit_item_impl(&mut self, i: &'ast ItemImpl) {
        use syn::spanned::Spanned;

        let type_name_str = match i.self_ty.as_ref() {
            syn::Type::Path(p) => p.path.segments.last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default(),
            _ => return,
        };
        if type_name_str.is_empty() { return; }

        let trait_prefix = i.trait_.as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .map(|s| format!("{} for ", s.ident))
            .unwrap_or_default();

        let methods: Vec<String> = i.items.iter().filter_map(|item| {
            if let syn::ImplItem::Fn(m) = item { Some(format!("fn {}", m.sig.ident)) }
            else { None }
        }).collect();

        let name        = format!("{trait_prefix}{type_name_str}");
        let doc_comment = Self::extract_doc(&i.attrs);
        let signature   = format!("impl {name} {{ {} }}", methods.join(", "));
        let span        = i.span();
        let full_source = self.lines_for_span(span.start().line, span.end().line);
        let kind        = ItemKind::Impl;
        let concept     = self.concept_name(&kind, &name);

        self.items.push(AstItem { name, kind, doc_comment, signature, full_source, concept });
        syn::visit::visit_item_impl(self, i);
    }
}

/// Best-effort extraction of a type's short name for use in signatures.
fn type_name(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(p) => p.path.segments.iter()
            .map(|s| {
                let args = match &s.arguments {
                    syn::PathArguments::AngleBracketed(ab) => {
                        let inner: Vec<String> = ab.args.iter().map(|a| match a {
                            syn::GenericArgument::Type(t) => type_name(t),
                            _ => "_".to_string(),
                        }).collect();
                        if inner.is_empty() { String::new() } else { format!("<{}>", inner.join(", ")) }
                    }
                    _ => String::new(),
                };
                format!("{}{}", s.ident, args)
            })
            .collect::<Vec<_>>()
            .join("::"),
        syn::Type::Reference(r) => {
            let mut_str = if r.mutability.is_some() { "mut " } else { "" };
            format!("&{}{}", mut_str, type_name(&r.elem))
        }
        syn::Type::Tuple(t) if t.elems.is_empty() => "()".to_string(),
        _ => "_".to_string(),
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Parse `source` as a Rust file and extract all public items.
///
/// `file_stem` is used in concept names (e.g. `"encode"` for `encode.rs`).
/// Returns an empty `Vec` if the file fails to parse.
pub fn extract_rust_items(file_stem: &str, source: &str) -> Vec<AstItem> {
    let syntax_tree = match syn::parse_file(source) {
        Ok(f)  => f,
        Err(_) => return Vec::new(),
    };

    let mut extractor = Extractor::new(file_stem, source);
    extractor.visit_file(&syntax_tree);
    extractor.items
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
/// Converts a text string to a HolographicBlock.
pub fn from_text(text: &str) -> Leg3Pointer {
    unimplemented!()
}

/// Core memory block — exactly 256KB.
pub struct HolographicBlock {
    pub q: [f32; 8192],
    pub crs_score: f32,
}

/// Search mode for query dispatch.
pub enum SearchMode {
    Cosine,
    Poincare,
    Int8Poincare,
}

fn private_helper() {}
"#;

    #[test]
    fn extracts_pub_fn() {
        let items = extract_rust_items("encode", SAMPLE);
        let f = items.iter().find(|i| i.kind == ItemKind::Function).unwrap();
        assert_eq!(f.name, "from_text");
        assert!(f.doc_comment.contains("Converts"), "doc: {}", f.doc_comment);
        assert!(f.concept.starts_with("encode__fn__from_text"), "concept: {}", f.concept);
    }

    #[test]
    fn extracts_pub_struct() {
        let items = extract_rust_items("types", SAMPLE);
        let s = items.iter().find(|i| i.kind == ItemKind::Struct).unwrap();
        assert_eq!(s.name, "HolographicBlock");
        assert!(s.signature.contains("q"), "sig: {}", s.signature);
    }

    #[test]
    fn extracts_pub_enum() {
        let items = extract_rust_items("backend", SAMPLE);
        let e = items.iter().find(|i| i.kind == ItemKind::Enum).unwrap();
        assert_eq!(e.name, "SearchMode");
        assert!(e.signature.contains("Cosine"), "sig: {}", e.signature);
    }

    #[test]
    fn skips_private_fn() {
        let items = extract_rust_items("lib", SAMPLE);
        assert!(!items.iter().any(|i| i.name == "private_helper"),
            "private fn must not be extracted");
    }

    #[test]
    fn embed_label_starts_with_concept() {
        let items = extract_rust_items("encode", SAMPLE);
        let f = items.iter().find(|i| i.kind == ItemKind::Function).unwrap();
        let label = f.embed_label();
        assert!(label.starts_with("encode__fn__from_text:"), "label: {label}");
    }
}
