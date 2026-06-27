use crate::ast::{Alignment, Block, ListItem};
use crate::inline_parser::parse_inlines;

// ── Public entry point ────────────────────────────────────────────────────────

pub fn parse_blocks(input: &str) -> Vec<Block> {
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<String> = normalized.lines().map(String::from).collect();
    let mut parser = BlockParser { lines, pos: 0 };
    parser.parse(0)
}

// ── Detection helpers (pure functions on &str) ────────────────────────────────

fn leading_spaces(s: &str) -> usize {
    s.chars().take_while(|c| *c == ' ').count()
}

fn strip_indent(s: &str, n: usize) -> &str {
    let mut count = 0;
    let mut idx = 0;
    for c in s.chars() {
        if count >= n { break; }
        if c == ' ' { count += 1; idx += 1; }
        else if c == '\t' { count += 4; idx += 1; } // treat tab as 4 spaces
        else { break; }
    }
    &s[idx..]
}

/// ATX heading: `# Title` → Some((level, text))
fn detect_atx_heading(line: &str) -> Option<(u8, &str)> {
    let s = line.trim_start_matches(' '); // up to 3 leading spaces
    if leading_spaces(line) > 3 { return None; }
    let hashes = s.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 { return None; }
    let rest = &s[hashes..];
    if !rest.is_empty() && rest.chars().next() != Some(' ') {
        return None; // must be followed by space or end
    }
    let text = rest.trim();
    // Strip trailing #s
    let text = text
        .trim_end_matches('#')
        .trim_end_matches(' ')
        .trim_end_matches('#')
        .trim_end();
    Some((hashes as u8, text))
}

/// Thematic break: `---`, `***`, `___` (with optional spaces)
fn is_thematic_break(line: &str) -> bool {
    if leading_spaces(line) > 3 { return false; }
    let s = line.trim();
    if s.len() < 3 { return false; }
    let ch = s.chars().next().unwrap();
    if !matches!(ch, '-' | '*' | '_') { return false; }
    let count = s.chars().filter(|&c| c == ch).count();
    count >= 3 && s.chars().all(|c| c == ch || c == ' ')
}

/// Fenced code block opening: returns `(fence_char, fence_len, lang)` or None.
fn detect_fence(line: &str) -> Option<(char, usize, String)> {
    if leading_spaces(line) > 3 { return None; }
    let s = line.trim_start();
    let ch = s.chars().next()?;
    if ch != '`' && ch != '~' { return None; }
    let len = s.chars().take_while(|&c| c == ch).count();
    if len < 3 { return None; }
    let lang = s[len..].trim().to_string();
    // Backtick fence cannot have backticks in info string
    if ch == '`' && lang.contains('`') { return None; }
    Some((ch, len, lang))
}

/// Unordered list marker: returns marker length (including trailing space) or None.
fn detect_unordered(line: &str) -> Option<usize> {
    if leading_spaces(line) > 3 { return None; }
    let s = line.trim_start();
    let ch = s.chars().next()?;
    if !matches!(ch, '-' | '*' | '+') { return None; }
    let rest = &s[1..];
    if rest.starts_with(' ') || rest.is_empty() {
        let indent = leading_spaces(line);
        Some(indent + 2) // marker width = 1 bullet + 1 space = 2
    } else {
        None
    }
}

/// Ordered list marker: returns `(start_number, total_marker_width)` or None.
fn detect_ordered(line: &str) -> Option<(u32, usize)> {
    if leading_spaces(line) > 3 { return None; }
    let s = line.trim_start();
    let num_end = s.chars().take_while(|c| c.is_ascii_digit()).count();
    if num_end == 0 || num_end > 9 { return None; }
    let rest = &s[num_end..];
    if !rest.starts_with('.') && !rest.starts_with(')') { return None; }
    let after_dot = &rest[1..];
    if !after_dot.starts_with(' ') && !after_dot.is_empty() { return None; }
    let n: u32 = s[..num_end].parse().ok()?;
    let indent = leading_spaces(line);
    Some((n, indent + num_end + 2)) // number digits + '.' + space
}

/// GFM table separator line: `| --- | :---: |` etc.
fn is_separator_line(line: &str) -> bool {
    let s = line.trim();
    if !s.contains('|') && !s.starts_with('-') && !s.starts_with(':') {
        return false;
    }
    s.split('|')
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .all(|cell| {
            let inner = cell.trim_matches(':').trim_matches('-');
            inner.is_empty() || inner.chars().all(|c| c == '-')
        })
}

fn parse_alignment(cell: &str) -> Alignment {
    let s = cell.trim();
    let left = s.starts_with(':');
    let right = s.ends_with(':');
    match (left, right) {
        (true, true) => Alignment::Center,
        (true, false) => Alignment::Left,
        (false, true) => Alignment::Right,
        _ => Alignment::None,
    }
}

fn split_table_row(line: &str) -> Vec<String> {
    let s = line.trim();
    let s = if s.starts_with('|') { &s[1..] } else { s };
    let s = if s.ends_with('|') { &s[..s.len() - 1] } else { s };
    s.split('|').map(|c| c.trim().to_string()).collect()
}

/// Task list checkbox at start of list item content.
fn detect_checkbox(s: &str) -> Option<(bool, &str)> {
    if s.starts_with("[ ] ") { return Some((false, &s[4..])); }
    if s.starts_with("[x] ") || s.starts_with("[X] ") { return Some((true, &s[4..])); }
    None
}

// ── Block parser ──────────────────────────────────────────────────────────────

struct BlockParser {
    lines: Vec<String>,
    pos: usize,
}

impl BlockParser {
    fn at_end(&self) -> bool {
        self.pos >= self.lines.len()
    }

    fn line(&self) -> Option<&str> {
        self.lines.get(self.pos).map(|s| s.as_str())
    }

    fn is_blank(&self, i: usize) -> bool {
        self.lines.get(i).map(|l| l.trim().is_empty()).unwrap_or(true)
    }

    fn skip_blank_lines(&mut self) {
        while !self.at_end() && self.lines[self.pos].trim().is_empty() {
            self.pos += 1;
        }
    }

    // ── Top-level block dispatch ──────────────────────────────────────────────

    fn parse(&mut self, min_indent: usize) -> Vec<Block> {
        let mut blocks = Vec::new();
        loop {
            self.skip_blank_lines();
            if self.at_end() { break; }

            // If the current line has less indent than required, stop
            if min_indent > 0 && leading_spaces(self.line().unwrap()) < min_indent {
                break;
            }

            match self.parse_one(min_indent) {
                Some(b) => blocks.push(b),
                None => break,
            }
        }
        blocks
    }

    fn parse_one(&mut self, min_indent: usize) -> Option<Block> {
        let raw = self.line()?.to_string();
        let trimmed = raw.trim_start();

        // ATX heading
        if let Some((level, text)) = detect_atx_heading(&raw) {
            self.pos += 1;
            return Some(Block::Heading { level, children: parse_inlines(text) });
        }

        // Thematic break (must check before list, since `---` can be setext)
        if is_thematic_break(trimmed) {
            self.pos += 1;
            return Some(Block::ThematicBreak);
        }

        // Fenced code block
        if let Some((fence_ch, fence_len, lang)) = detect_fence(&raw) {
            return Some(self.parse_fenced(fence_ch, fence_len, lang));
        }

        // Indented code block (4+ spaces, only outside list context)
        if min_indent == 0 && leading_spaces(&raw) >= 4 {
            return Some(self.parse_indented_code());
        }

        // Block quote
        if trimmed.starts_with('>') {
            return Some(self.parse_blockquote());
        }

        // Unordered list
        if let Some(marker_width) = detect_unordered(&raw) {
            return Some(self.parse_list(false, 1, marker_width));
        }

        // Ordered list
        if let Some((start, marker_width)) = detect_ordered(&raw) {
            return Some(self.parse_list(true, start, marker_width));
        }

        // GFM table — peek at next line for separator
        if self.pos + 1 < self.lines.len()
            && is_separator_line(self.lines[self.pos + 1].trim())
            && trimmed.contains('|')
        {
            return Some(self.parse_table());
        }

        // Paragraph (catches setext headings inside)
        Some(self.parse_paragraph(min_indent))
    }

    // ── Fenced code block ─────────────────────────────────────────────────────

    fn parse_fenced(&mut self, fence_ch: char, fence_len: usize, lang: String) -> Block {
        self.pos += 1; // skip opening fence
        let mut code_lines: Vec<String> = Vec::new();

        while !self.at_end() {
            let line = self.lines[self.pos].clone();
            // Closing fence: same char, >= same length, only whitespace after
            let s = line.trim_start();
            let ch_count = s.chars().take_while(|&c| c == fence_ch).count();
            let after = &s[ch_count..];
            if ch_count >= fence_len && after.trim().is_empty() {
                self.pos += 1;
                break;
            }
            code_lines.push(line);
            self.pos += 1;
        }

        Block::CodeBlock {
            lang: if lang.is_empty() { None } else { Some(lang) },
            code: code_lines.join("\n"),
        }
    }

    // ── Indented code block ───────────────────────────────────────────────────

    fn parse_indented_code(&mut self) -> Block {
        let mut lines: Vec<String> = Vec::new();
        loop {
            if self.at_end() { break; }
            let line = &self.lines[self.pos];
            if line.trim().is_empty() {
                // Blank line — include but don't stop yet (check next non-blank)
                lines.push(String::new());
                self.pos += 1;
                // Look ahead: if next non-blank line isn't indented, stop
                let mut la = self.pos;
                while la < self.lines.len() && self.lines[la].trim().is_empty() { la += 1; }
                if la >= self.lines.len() || leading_spaces(&self.lines[la]) < 4 {
                    break;
                }
            } else if leading_spaces(line) >= 4 {
                lines.push(strip_indent(line, 4).to_string());
                self.pos += 1;
            } else {
                break;
            }
        }
        // Trim trailing blank lines
        while lines.last().map(|l: &String| l.trim().is_empty()).unwrap_or(false) {
            lines.pop();
        }
        Block::CodeBlock { lang: None, code: lines.join("\n") }
    }

    // ── Block quote ───────────────────────────────────────────────────────────

    fn parse_blockquote(&mut self) -> Block {
        let mut inner_lines: Vec<String> = Vec::new();

        while !self.at_end() {
            let line = self.lines[self.pos].clone();
            let trimmed = line.trim_start();

            if trimmed.starts_with('>') {
                // Strip the > marker (and one optional space after it)
                let after = &trimmed[1..];
                let content = if after.starts_with(' ') { &after[1..] } else { after };
                inner_lines.push(content.to_string());
                self.pos += 1;
            } else if trimmed.is_empty() {
                // Blank line ends the blockquote
                break;
            } else {
                // Lazy continuation (paragraph continuation inside blockquote)
                inner_lines.push(line.clone());
                self.pos += 1;
            }
        }

        let inner_text = inner_lines.join("\n");
        let children = parse_blocks(&inner_text);
        Block::BlockQuote(children)
    }

    // ── Lists ─────────────────────────────────────────────────────────────────

    fn parse_list(&mut self, ordered: bool, start: u32, marker_width: usize) -> Block {
        let mut items: Vec<ListItem> = Vec::new();
        let mut has_blank_between = false;
        let mut prev_was_blank = false;

        loop {
            self.skip_blank_lines();
            if self.at_end() { break; }

            let line = &self.lines[self.pos];
            let trimmed = line.trim_start();

            // Is this another item in the same list?
            let is_same_list = if ordered {
                detect_ordered(line).is_some()
            } else {
                detect_unordered(line).is_some()
            };

            if !is_same_list && !trimmed.is_empty() {
                break;
            }
            if !is_same_list { break; }

            if prev_was_blank && !items.is_empty() {
                has_blank_between = true;
            }

            items.push(self.parse_list_item(marker_width));

            // Check for blank line after item
            prev_was_blank = !self.at_end() && self.lines[self.pos].trim().is_empty();
        }

        let tight = !has_blank_between;

        // For tight lists, strip paragraph wrappers from single-paragraph items
        if tight {
            for item in &mut items {
                if item.children.len() == 1 {
                    if let Block::Paragraph(inlines) = item.children[0].clone() {
                        item.children = vec![Block::Paragraph(inlines)];
                    }
                }
            }
        }

        Block::List { ordered, start, tight, items }
    }

    fn parse_list_item(&mut self, marker_width: usize) -> ListItem {
        let first_line = self.lines[self.pos].clone();
        self.pos += 1;

        // Content of first line: strip the marker
        let first_content = if first_line.len() >= marker_width {
            first_line[marker_width..].to_string()
        } else {
            String::new()
        };

        // Collect continuation lines (indented by at least marker_width)
        let mut content_lines = vec![first_content.clone()];

        loop {
            if self.at_end() { break; }
            let line = &self.lines[self.pos];
            let trimmed = line.trim_start();

            if trimmed.is_empty() {
                content_lines.push(String::new());
                self.pos += 1;
                // Look ahead to see if next non-blank line continues this item
                let mut la = self.pos;
                while la < self.lines.len() && self.lines[la].trim().is_empty() { la += 1; }
                if la >= self.lines.len() { break; }
                let la_indent = leading_spaces(&self.lines[la]);
                if la_indent < marker_width {
                    break;
                }
            } else if leading_spaces(line) >= marker_width {
                content_lines.push(strip_indent(line, marker_width).to_string());
                self.pos += 1;
            } else {
                break;
            }
        }

        let raw = content_lines.join("\n");
        let raw = raw.trim_end();

        // Detect task-list checkbox
        let (checked, actual_content) = if let Some((chk, rest)) = detect_checkbox(raw.trim_start()) {
            (Some(chk), rest.to_string())
        } else {
            (None, raw.to_string())
        };

        let children = parse_blocks(&actual_content);

        ListItem { checked, children }
    }

    // ── GFM table ─────────────────────────────────────────────────────────────

    fn parse_table(&mut self) -> Block {
        let header_line = self.lines[self.pos].clone();
        let sep_line = self.lines[self.pos + 1].clone();
        self.pos += 2;

        let header_cells = split_table_row(&header_line);
        let alignments: Vec<Alignment> = split_table_row(&sep_line)
            .iter()
            .map(|c| parse_alignment(c))
            .collect();

        let headers: Vec<Vec<_>> = header_cells.iter()
            .map(|c| parse_inlines(c))
            .collect();

        let mut rows = Vec::new();
        while !self.at_end() {
            let line = &self.lines[self.pos];
            if line.trim().is_empty() { break; }
            if !line.trim().contains('|') { break; }
            let row: Vec<Vec<_>> = split_table_row(line)
                .iter()
                .map(|c| parse_inlines(c))
                .collect();
            rows.push(row);
            self.pos += 1;
        }

        Block::Table { headers, alignments, rows }
    }

    // ── Paragraph (also handles setext headings) ──────────────────────────────

    fn parse_paragraph(&mut self, min_indent: usize) -> Block {
        let mut lines: Vec<String> = Vec::new();

        loop {
            if self.at_end() { break; }
            let raw = self.lines[self.pos].clone();
            let trimmed = raw.trim_start();

            // Blank line ends paragraph
            if trimmed.is_empty() { break; }

            // Check indent for nested contexts
            if min_indent > 0 && leading_spaces(&raw) < min_indent { break; }

            // ATX headings interrupt paragraphs
            if detect_atx_heading(&raw).is_some() { break; }

            // Fenced code interrupts paragraphs
            if detect_fence(trimmed).is_some() { break; }

            // Thematic break: but first check if it's a setext heading underline
            if is_thematic_break(trimmed) {
                let all_eq = trimmed.chars().all(|c| c == '=');
                let all_dash = trimmed.chars().all(|c| c == '-');
                if !lines.is_empty() && (all_eq || all_dash) {
                    let level: u8 = if all_eq { 1 } else { 2 };
                    self.pos += 1;
                    let text = lines.join("\n");
                    return Block::Heading { level, children: parse_inlines(text.trim()) };
                }
                break;
            }

            // A list item can interrupt a paragraph only if it's a numbered list starting at 1,
            // or if the paragraph is empty (first line); simplify: never interrupt
            lines.push(raw);
            self.pos += 1;
        }

        let text = lines.join("\n");
        Block::Paragraph(parse_inlines(text.trim()))
    }
}
