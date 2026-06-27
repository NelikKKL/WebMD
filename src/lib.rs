mod ast;
mod block_parser;
mod html_renderer;
mod inline_parser;

use wasm_bindgen::prelude::*;

/// One-shot render: Markdown (+ WebMD extensions) → HTML string.
///
/// Extensions supported on top of CommonMark + GFM:
/// - `#grad[red, blue](text)`  — gradient text via CSS linear-gradient
/// - `#font[Comic Sans MS](text)` — custom font-family
#[wasm_bindgen]
pub fn render(markdown: &str) -> String {
    let blocks = block_parser::parse_blocks(markdown);
    html_renderer::render_html(&blocks)
}

/// Stateful renderer object (useful when you need to reuse options later).
#[wasm_bindgen]
pub struct WebMD {
    /// Reserved for future per-instance configuration (e.g. custom extensions,
    /// syntax-highlight callbacks, link rewriters).
    _private: (),
}

#[wasm_bindgen]
impl WebMD {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WebMD {
        WebMD { _private: () }
    }

    /// Render Markdown to HTML.
    pub fn render(&self, markdown: &str) -> String {
        render(markdown)
    }

    /// Parse Markdown and return the AST as a JSON string.
    /// Useful for tooling, syntax trees, or custom renderers written in JS.
    pub fn parse_to_json(&self, markdown: &str) -> String {
        let blocks = block_parser::parse_blocks(markdown);
        blocks_to_json(&blocks)
    }
}

// ── Minimal AST → JSON serialisation (no serde dependency) ───────────────────

fn blocks_to_json(blocks: &[ast::Block]) -> String {
    let items: Vec<String> = blocks.iter().map(block_to_json).collect();
    format!("[{}]", items.join(","))
}

fn block_to_json(block: &ast::Block) -> String {
    match block {
        ast::Block::Heading { level, children } => {
            format!(
                "{{\"type\":\"heading\",\"level\":{},\"children\":{}}}",
                level,
                inlines_to_json(children)
            )
        }
        ast::Block::Paragraph(children) => {
            format!("{{\"type\":\"paragraph\",\"children\":{}}}", inlines_to_json(children))
        }
        ast::Block::BlockQuote(children) => {
            format!("{{\"type\":\"blockquote\",\"children\":{}}}", blocks_to_json(children))
        }
        ast::Block::List { ordered, start, tight, items } => {
            let items_json: Vec<String> = items.iter().map(|it| {
                format!(
                    "{{\"checked\":{},\"children\":{}}}",
                    match it.checked {
                        Some(v) => if v { "true" } else { "false" },
                        None => "null",
                    },
                    blocks_to_json(&it.children)
                )
            }).collect();
            format!(
                "{{\"type\":\"list\",\"ordered\":{},\"start\":{},\"tight\":{},\"items\":[{}]}}",
                ordered, start, tight,
                items_json.join(",")
            )
        }
        ast::Block::CodeBlock { lang, code } => {
            format!(
                "{{\"type\":\"code_block\",\"lang\":{},\"code\":{}}}",
                json_string_or_null(lang.as_deref()),
                json_string(code)
            )
        }
        ast::Block::ThematicBreak => r#"{"type":"thematic_break"}"#.to_string(),
        ast::Block::HtmlBlock(html) => {
            format!("{{\"type\":\"html_block\",\"html\":{}}}", json_string(html))
        }
        ast::Block::Table { headers, alignments, rows } => {
            let headers_json: Vec<String> = headers.iter().map(|h| inlines_to_json(h)).collect();
            let aligns_json: Vec<&str> = alignments.iter().map(|a| match a {
                ast::Alignment::Left => "\"left\"",
                ast::Alignment::Center => "\"center\"",
                ast::Alignment::Right => "\"right\"",
                ast::Alignment::None => "null",
            }).collect();
            let rows_json: Vec<String> = rows.iter().map(|row| {
                let cells: Vec<String> = row.iter().map(|c| inlines_to_json(c)).collect();
                format!("[{}]", cells.join(","))
            }).collect();
            format!(
                "{{\"type\":\"table\",\"headers\":[{}],\"alignments\":[{}],\"rows\":[{}]}}",
                headers_json.join(","),
                aligns_json.join(","),
                rows_json.join(",")
            )
        }
    }
}

fn inlines_to_json(inlines: &[ast::Inline]) -> String {
    let items: Vec<String> = inlines.iter().map(inline_to_json).collect();
    format!("[{}]", items.join(","))
}

fn inline_to_json(inline: &ast::Inline) -> String {
    match inline {
        ast::Inline::Text(s) => format!("{{\"type\":\"text\",\"value\":{}}}", json_string(s)),
        ast::Inline::SoftBreak => r#"{"type":"soft_break"}"#.to_string(),
        ast::Inline::HardBreak => r#"{"type":"hard_break"}"#.to_string(),
        ast::Inline::Code(s) => format!("{{\"type\":\"code\",\"value\":{}}}", json_string(s)),
        ast::Inline::Emphasis(c) => format!("{{\"type\":\"emphasis\",\"children\":{}}}", inlines_to_json(c)),
        ast::Inline::Strong(c) => format!("{{\"type\":\"strong\",\"children\":{}}}", inlines_to_json(c)),
        ast::Inline::StrongEmphasis(c) => format!("{{\"type\":\"strong_emphasis\",\"children\":{}}}", inlines_to_json(c)),
        ast::Inline::Strikethrough(c) => format!("{{\"type\":\"strikethrough\",\"children\":{}}}", inlines_to_json(c)),
        ast::Inline::Link { children, url, title } => format!(
            "{{\"type\":\"link\",\"url\":{},\"title\":{},\"children\":{}}}",
            json_string(url), json_string_or_null(title.as_deref()), inlines_to_json(children)
        ),
        ast::Inline::Image { alt, src, title } => format!(
            "{{\"type\":\"image\",\"src\":{},\"alt\":{},\"title\":{}}}",
            json_string(src), json_string(alt), json_string_or_null(title.as_deref())
        ),
        ast::Inline::RawHtml(html) => format!("{{\"type\":\"raw_html\",\"html\":{}}}", json_string(html)),
        ast::Inline::Gradient { from, to, children } => format!(
            "{{\"type\":\"gradient\",\"from\":{},\"to\":{},\"children\":{}}}",
            json_string(from), json_string(to), inlines_to_json(children)
        ),
        ast::Inline::Font { family, children } => format!(
            "{{\"type\":\"font\",\"family\":{},\"children\":{}}}",
            json_string(family), inlines_to_json(children)
        ),
    }
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_string_or_null(s: Option<&str>) -> String {
    match s {
        Some(v) => json_string(v),
        None => "null".to_string(),
    }
}
