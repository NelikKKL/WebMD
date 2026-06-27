/// Block-level elements of the document tree.
#[derive(Debug, Clone)]
pub enum Block {
    /// ATX heading: `# H1` … `###### H6`
    /// Setext headings are converted to this during parsing.
    Heading {
        level: u8,
        children: Vec<Inline>,
    },
    /// A paragraph of inline content.
    Paragraph(Vec<Inline>),
    /// A block quote: `> ...`
    BlockQuote(Vec<Block>),
    /// Ordered (`1.`) or unordered (`-`, `*`, `+`) list.
    List {
        ordered: bool,
        /// Start number for ordered lists.
        start: u32,
        /// Tight = no blank lines between items.
        tight: bool,
        items: Vec<ListItem>,
    },
    /// Fenced (` ``` `) or indented (4-space) code block.
    CodeBlock {
        lang: Option<String>,
        code: String,
    },
    /// `---`, `***`, `___`
    ThematicBreak,
    /// Raw HTML passthrough block.
    HtmlBlock(String),
    /// GitHub-Flavored Markdown pipe table.
    Table {
        headers: Vec<Vec<Inline>>,
        alignments: Vec<Alignment>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
}

/// A single list item.
#[derive(Debug, Clone)]
pub struct ListItem {
    /// GFM task-list checkbox: `Some(true)` = `[x]`, `Some(false)` = `[ ]`.
    pub checked: Option<bool>,
    pub children: Vec<Block>,
}

/// Column alignment in a GFM table.
#[derive(Debug, Clone, PartialEq)]
pub enum Alignment {
    None,
    Left,
    Center,
    Right,
}

/// Inline elements that appear inside blocks.
#[derive(Debug, Clone)]
pub enum Inline {
    /// Plain text (HTML-escaped by the renderer).
    Text(String),
    /// Rendered as a space (within `<p>` content).
    SoftBreak,
    /// Rendered as `<br />`.
    HardBreak,
    /// Inline code: `` `code` `` — HTML-escaped inside `<code>`.
    Code(String),
    /// `*text*` or `_text_`
    Emphasis(Vec<Inline>),
    /// `**text**` or `__text__`
    Strong(Vec<Inline>),
    /// `***text***` or `___text___`
    StrongEmphasis(Vec<Inline>),
    /// GFM strikethrough: `~~text~~`
    Strikethrough(Vec<Inline>),
    /// `[label](url "title")`
    Link {
        children: Vec<Inline>,
        url: String,
        title: Option<String>,
    },
    /// `![alt](src "title")`
    Image {
        alt: String,
        src: String,
        title: Option<String>,
    },
    /// Passthrough HTML tag, e.g. from autolinks.
    RawHtml(String),

    // ── Custom WebMD extensions ────────────────────────────────────────────────

    /// `#grad[from, to](text)` — CSS linear-gradient text.
    Gradient {
        from: String,
        to: String,
        children: Vec<Inline>,
    },
    /// `#font[family](text)` — custom font-family.
    Font {
        family: String,
        children: Vec<Inline>,
    },
}
