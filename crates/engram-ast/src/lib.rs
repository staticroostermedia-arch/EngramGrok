//! Universal AST-aware source extraction for Engram ingestion via tree-sitter.
//!
//! Parses source code (Rust, Python, TS, JS, Go, Java, C, C++) and produces
//! one [`AstItem`] per public semantic block (function, class, struct, etc.).
//!
//! - **Concept name** = `{file_stem}__{kind}__{item_name}`
//! - **Embed label** = First 2 lines of the block (signature) as semantic distillation
//! - **Full source** = Complete node source for the provlog

use tree_sitter::{Language, Parser, Query, QueryCursor, Node};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Class,
    Method,
    Interface,
    Unknown(String),
}

impl ItemKind {
    pub fn as_str(&self) -> &str {
        match self {
            ItemKind::Function => "fn",
            ItemKind::Struct   => "struct",
            ItemKind::Enum     => "enum",
            ItemKind::Trait    => "trait",
            ItemKind::Impl     => "impl",
            ItemKind::Class    => "class",
            ItemKind::Method   => "method",
            ItemKind::Interface=> "interface",
            ItemKind::Unknown(s) => s.as_str(),
        }
    }
}

/// A single extracted semantic item.
#[derive(Debug, Clone)]
pub struct AstItem {
    pub name: String,
    pub kind: ItemKind,
    pub doc_comment: String,
    pub signature: String,
    pub full_source: String,
    pub concept: String,
    // --- PHASE 8.2: EMBODIMENT ---
    pub start_pos: (usize, usize),
    pub end_pos: (usize, usize),
}

impl AstItem {
    pub fn embed_label(&self) -> String {
        if self.doc_comment.is_empty() {
            format!("{}: {}", self.concept, self.signature)
        } else {
            format!("{}: {} — {}", self.concept, self.doc_comment, self.signature)
        }
    }
}

/// Helper to extract preceding contiguous comments.
fn extract_preceding_comments(source_bytes: &[u8], mut node: Node) -> String {
    let mut comments = Vec::new();
    while let Some(prev) = node.prev_sibling() {
        if prev.kind().contains("comment") {
            if let Ok(text) = std::str::from_utf8(&source_bytes[prev.start_byte()..prev.end_byte()]) {
                comments.push(text.trim().to_string());
            }
            node = prev;
        } else {
            break;
        }
    }
    comments.reverse();
    comments.join(" ")
}

/// Common structure to hold parsing configuration for a language
struct LangConfig {
    language: Language,
    query_str: &'static str,
}

impl LangConfig {
    fn new(language: Language, query_str: &'static str) -> Self {
        Self { language, query_str }
    }
}

fn get_config(ext: &str) -> Option<LangConfig> {
    match ext {
        "rs" => Some(LangConfig::new(
            tree_sitter_rust::LANGUAGE.into(),
            r#"
            (function_item name: (identifier) @name) @fn
            (struct_item name: (type_identifier) @name) @struct
            (enum_item name: (type_identifier) @name) @enum
            (trait_item name: (type_identifier) @name) @trait
            (impl_item type: (type_identifier) @name) @impl
            "#
        )),
        "py" => Some(LangConfig::new(
            tree_sitter_python::LANGUAGE.into(),
            r#"
            (function_definition name: (identifier) @name) @fn
            (class_definition name: (identifier) @name) @class
            "#
        )),
        "ts" | "tsx" => Some(LangConfig::new(
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            r#"
            (function_declaration name: (identifier) @name) @fn
            (class_declaration name: (type_identifier) @name) @class
            (interface_declaration name: (type_identifier) @name) @interface
            (method_definition name: (property_identifier) @name) @method
            (lexical_declaration (variable_declarator name: (identifier) @name value: (arrow_function))) @fn
            "#
        )),
        "js" | "jsx" => Some(LangConfig::new(
            tree_sitter_javascript::LANGUAGE.into(),
            r#"
            (function_declaration name: (identifier) @name) @fn
            (class_declaration name: (identifier) @name) @class
            (method_definition name: (property_identifier) @name) @method
            (lexical_declaration (variable_declarator name: (identifier) @name value: (arrow_function))) @fn
            "#
        )),
        "go" => Some(LangConfig::new(
            tree_sitter_go::LANGUAGE.into(),
            r#"
            (function_declaration name: (identifier) @name) @fn
            (method_declaration name: (field_identifier) @name) @method
            (type_declaration (type_spec name: (type_identifier) @name type: (struct_type))) @struct
            (type_declaration (type_spec name: (type_identifier) @name type: (interface_type))) @interface
            "#
        )),
        "java" => Some(LangConfig::new(
            tree_sitter_java::LANGUAGE.into(),
            r#"
            (class_declaration name: (identifier) @name) @class
            (interface_declaration name: (identifier) @name) @interface
            (method_declaration name: (identifier) @name) @method
            "#
        )),
        "c" => Some(LangConfig::new(
            tree_sitter_c::LANGUAGE.into(),
            r#"
            (function_definition declarator: (function_declarator declarator: (identifier) @name)) @fn
            (struct_specifier name: (type_identifier) @name) @struct
            (enum_specifier name: (type_identifier) @name) @enum
            "#
        )),
        "cpp" | "cc" | "cxx" | "h" | "hpp" => Some(LangConfig::new(
            tree_sitter_cpp::LANGUAGE.into(),
            r#"
            (function_definition declarator: (function_declarator declarator: (identifier) @name)) @fn
            (function_definition declarator: (function_declarator declarator: (field_identifier) @name)) @method
            (class_specifier name: (type_identifier) @name) @class
            (struct_specifier name: (type_identifier) @name) @struct
            (enum_specifier name: (type_identifier) @name) @enum
            "#
        )),
        _ => None,
    }
}

/// Extract all supported AST items from the given source text based on the file extension.
pub fn extract_ast_items(file_path: &str, source: &str) -> Vec<AstItem> {
    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let config = match get_config(ext) {
        Some(c) => c,
        None => return Vec::new(),
    };

    let mut parser = Parser::new();
    if parser.set_language(&config.language).is_err() {
        return Vec::new();
    }

    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let query = match Query::new(&config.language, config.query_str) {
        Ok(q) => q,
        Err(_) => return Vec::new(),
    };

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    let mut items = Vec::new();
    let file_stem = std::path::Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric(), "_");

    let source_bytes = source.as_bytes();

    while let Some(m) = matches.next() {
        let mut name = String::new();
        let mut kind = ItemKind::Unknown("item".to_string());
        let mut node_opt = None;

        for capture in m.captures {
            let capture_name = query.capture_names()[capture.index as usize];
            if capture_name == "name" {
                if let Ok(n) = std::str::from_utf8(&source_bytes[capture.node.start_byte()..capture.node.end_byte()]) {
                    name = n.to_string();
                }
            } else {
                // Must be one of the item captures: @fn, @class, @struct, etc.
                kind = match capture_name {
                    "fn" => ItemKind::Function,
                    "class" => ItemKind::Class,
                    "struct" => ItemKind::Struct,
                    "enum" => ItemKind::Enum,
                    "trait" => ItemKind::Trait,
                    "impl" => ItemKind::Impl,
                    "method" => ItemKind::Method,
                    "interface" => ItemKind::Interface,
                    s => ItemKind::Unknown(s.to_string()),
                };
                node_opt = Some(capture.node);
            }
        }

        if let Some(node) = node_opt {
            if !name.is_empty() {
                let start = node.start_byte();
                let end = node.end_byte();
                
                // Extract 2D coordinates for the AABB mapping
                let start_pos = (node.start_position().row, node.start_position().column);
                let end_pos = (node.end_position().row, node.end_position().column);

                let full_source = std::str::from_utf8(&source_bytes[start..end]).unwrap_or("").to_string();

                // Signature is roughly the first line or two of the node
                let first_newline = full_source.find('\n').unwrap_or(full_source.len());
                let mut sig_end = first_newline;
                if sig_end < full_source.len() - 1 && full_source.chars().filter(|c| *c == '{').count() > 0 {
                    if let Some(brace) = full_source.find('{') {
                         sig_end = sig_end.max(brace + 1);
                    }
                }
                let signature = full_source[0..sig_end.min(full_source.len())].replace('\n', " ").trim().to_string();

                let doc_comment = extract_preceding_comments(source_bytes, node);

                let concept = format!("{}__{}__{}", file_stem, kind.as_str(), name)
                    .to_lowercase()
                    .replace(|c: char| !c.is_alphanumeric() && c != '_', "_");

                items.push(AstItem {
                    name,
                    kind,
                    doc_comment,
                    signature,
                    full_source,
                    concept,
                    start_pos,
                    end_pos,
                });
            }
        }
    }

    items
}
