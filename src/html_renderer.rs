use crate::ast::{Alignment, Block, Inline, ListItem};

// ── Public entry point ────────────────────────────────────────────────────────

pub fn render_html(blocks: &[Block]) -> String {
    let mut out = String::new();
    render_blocks(blocks, &mut out);
    out
}

// ── Escape helpers ────────────────────────────────────────────────────────────

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

// ── Block rendering ───────────────────────────────────────────────────────────

fn render_blocks(blocks: &[Block], out: &mut String) {
    for block in blocks {
        render_block(block, out);
    }
}

fn render_block(block: &Block, out: &mut String) {
    match block {
        Block::Heading { level, children } => {
            let tag = format!("h{}", level);
            out.push('<');
            out.push_str(&tag);
            // Auto-generate id from text content for anchor links
            let text_content = plain_text(children);
            if !text_content.is_empty() {
                let slug = slugify(&text_content);
                out.push_str(&format!(" id=\"{}\"", slug));
            }
            out.push('>');
            render_inlines(children, out);
            out.push_str(&format!("</{}>\n", tag));
        }

        Block::Paragraph(children) => {
            out.push_str("<p>");
            render_inlines(children, out);
            out.push_str("</p>\n");
        }

        Block::BlockQuote(children) => {
            out.push_str("<blockquote>\n");
            render_blocks(children, out);
            out.push_str("</blockquote>\n");
        }

        Block::List { ordered, start, tight, items } => {
            if *ordered {
                if *start != 1 {
                    out.push_str(&format!("<ol start=\"{}\">\n", start));
                } else {
                    out.push_str("<ol>\n");
                }
            } else {
                out.push_str("<ul>\n");
            }

            for item in items {
                render_list_item(item, *tight, out);
            }

            if *ordered {
                out.push_str("</ol>\n");
            } else {
                out.push_str("</ul>\n");
            }
        }

        Block::CodeBlock { lang, code } => {
            if let Some(lang_str) = lang {
                out.push_str(&format!(
                    "<pre><code class=\"language-{}\">{}</code></pre>\n",
                    escape_attr(lang_str),
                    escape_html(code)
                ));
            } else {
                out.push_str(&format!("<pre><code>{}</code></pre>\n", escape_html(code)));
            }
        }

        Block::ThematicBreak => {
            out.push_str("<hr />\n");
        }

        Block::HtmlBlock(html) => {
            out.push_str(html);
            out.push('\n');
        }

        Block::Table { headers, alignments, rows } => {
            out.push_str("<table>\n<thead>\n<tr>\n");
            for (i, header) in headers.iter().enumerate() {
                let align = alignments.get(i).unwrap_or(&Alignment::None);
                let style = align_style(align);
                out.push_str(&format!("<th{}>", style));
                render_inlines(header, out);
                out.push_str("</th>\n");
            }
            out.push_str("</tr>\n</thead>\n<tbody>\n");
            for row in rows {
                out.push_str("<tr>\n");
                for (i, cell) in row.iter().enumerate() {
                    let align = alignments.get(i).unwrap_or(&Alignment::None);
                    let style = align_style(align);
                    out.push_str(&format!("<td{}>", style));
                    render_inlines(cell, out);
                    out.push_str("</td>\n");
                }
                out.push_str("</tr>\n");
            }
            out.push_str("</tbody>\n</table>\n");
        }
    }
}

fn render_list_item(item: &ListItem, tight: bool, out: &mut String) {
    // Task list checkbox
    let checkbox = match item.checked {
        Some(true) => "<input type=\"checkbox\" checked disabled /> ",
        Some(false) => "<input type=\"checkbox\" disabled /> ",
        None => "",
    };

    out.push_str("<li>");
    if !checkbox.is_empty() {
        out.push_str(checkbox);
    }

    if tight {
        // Tight list: render paragraph children directly (no <p> wrapper)
        for child in &item.children {
            match child {
                Block::Paragraph(inlines) => render_inlines(inlines, out),
                other => render_block(other, out),
            }
        }
    } else {
        out.push('\n');
        render_blocks(&item.children, out);
    }

    out.push_str("</li>\n");
}

fn align_style(a: &Alignment) -> &'static str {
    match a {
        Alignment::Left => " style=\"text-align:left\"",
        Alignment::Center => " style=\"text-align:center\"",
        Alignment::Right => " style=\"text-align:right\"",
        Alignment::None => "",
    }
}

// ── Inline rendering ──────────────────────────────────────────────────────────

fn render_inlines(inlines: &[Inline], out: &mut String) {
    for inline in inlines {
        render_inline(inline, out);
    }
}

fn render_inline(inline: &Inline, out: &mut String) {
    match inline {
        Inline::Text(s) => out.push_str(&escape_html(s)),

        Inline::SoftBreak => out.push('\n'),

        Inline::HardBreak => out.push_str("<br />\n"),

        Inline::Code(s) => {
            out.push_str("<code>");
            out.push_str(&escape_html(s));
            out.push_str("</code>");
        }

        Inline::Emphasis(children) => {
            out.push_str("<em>");
            render_inlines(children, out);
            out.push_str("</em>");
        }

        Inline::Strong(children) => {
            out.push_str("<strong>");
            render_inlines(children, out);
            out.push_str("</strong>");
        }

        Inline::StrongEmphasis(children) => {
            out.push_str("<strong><em>");
            render_inlines(children, out);
            out.push_str("</em></strong>");
        }

        Inline::Strikethrough(children) => {
            out.push_str("<del>");
            render_inlines(children, out);
            out.push_str("</del>");
        }

        Inline::Link { children, url, title } => {
            out.push_str("<a href=\"");
            out.push_str(&escape_attr(url));
            out.push('"');
            if let Some(t) = title {
                out.push_str(" title=\"");
                out.push_str(&escape_attr(t));
                out.push('"');
            }
            out.push('>');
            render_inlines(children, out);
            out.push_str("</a>");
        }

        Inline::Image { alt, src, title } => {
            out.push_str("<img src=\"");
            out.push_str(&escape_attr(src));
            out.push_str("\" alt=\"");
            out.push_str(&escape_attr(alt));
            out.push('"');
            if let Some(t) = title {
                out.push_str(" title=\"");
                out.push_str(&escape_attr(t));
                out.push('"');
            }
            out.push_str(" />");
        }

        Inline::RawHtml(html) => out.push_str(html),

        // ── Custom extensions ──────────────────────────────────────────────────

        Inline::Gradient { from, to, children } => {
            out.push_str(&format!(
                "<span style=\"background: linear-gradient(to right, {}, {}); \
                 -webkit-background-clip: text; -webkit-text-fill-color: transparent; \
                 background-clip: text;\">",
                escape_attr(from),
                escape_attr(to)
            ));
            render_inlines(children, out);
            out.push_str("</span>");
        }

        Inline::Font { family, children } => {
            out.push_str(&format!(
                "<span style=\"font-family: '{}'\">",
                escape_attr(family)
            ));
            render_inlines(children, out);
            out.push_str("</span>");
        }
    }
}

// ── Utilities ─────────────────────────────────────────────────────────────────

/// Extract plain text from a list of inlines (for heading id generation).
fn plain_text(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(s) => out.push_str(s),
            Inline::Code(s) => out.push_str(s),
            Inline::Emphasis(c) | Inline::Strong(c) | Inline::StrongEmphasis(c)
            | Inline::Strikethrough(c) => out.push_str(&plain_text(c)),
            Inline::Link { children, .. } => out.push_str(&plain_text(children)),
            Inline::Gradient { children, .. } | Inline::Font { children, .. } => {
                out.push_str(&plain_text(children))
            }
            Inline::SoftBreak | Inline::HardBreak => out.push(' '),
            _ => {}
        }
    }
    out
}

/// GitHub-style heading slug: lowercase, spaces → `-`, remove non-alphanumeric.
fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else if c == ' ' { '-' } else { '\0' })
        .filter(|&c| c != '\0')
        .collect()
}
