use crate::ast::Inline;

// ── Public entry point ────────────────────────────────────────────────────────

pub fn parse_inlines(input: &str) -> Vec<Inline> {
    InlineParser::new(input).parse()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn flush(buf: &mut String, out: &mut Vec<Inline>) {
    if !buf.is_empty() {
        out.push(Inline::Text(std::mem::take(buf)));
    }
}

fn is_ascii_punct(c: char) -> bool {
    matches!(
        c,
        '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+'
        | ',' | '-' | '.' | '/' | ':' | ';' | '<' | '=' | '>' | '?' | '@'
        | '[' | '\\' | ']' | '^' | '_' | '`' | '{' | '|' | '}' | '~'
    )
}

/// Parse `url "optional title"` from inside `(…)`.
fn split_url_title(content: &str) -> (String, Option<String>) {
    let s = content.trim();

    // Look for a title delimited by " or '
    for &delim in &['"', '\''] {
        if let Some(ti) = s.rfind(delim) {
            if ti > 0 {
                let before = s[..ti].trim();
                // Make sure there's also an opening delimiter
                if let Some(open) = s[..ti].rfind(delim) {
                    if open < ti.saturating_sub(0) {
                        let title = &s[open + 1..ti];
                        let url = s[..open].trim();
                        return (url.to_string(), Some(title.to_string()));
                    }
                }
                // Single quoted segment at the end
                if let Some(start) = s.rfind(|c| c == delim) {
                    if start < ti {
                        let title = &s[start + 1..ti];
                        let url = s[..start].trim();
                        return (url.to_string(), Some(title.to_string()));
                    }
                }
                // Fallback: title is everything after last space
                if let Some(sp) = before.rfind(' ') {
                    let url = &before[..sp];
                    let raw = &before[sp + 1..];
                    if raw.starts_with(delim) {
                        let title = &s[sp + 2..ti];
                        return (url.trim().to_string(), Some(title.to_string()));
                    }
                }
            }
        }
    }

    (s.to_string(), None)
}

// ── Parser struct ─────────────────────────────────────────────────────────────

struct InlineParser {
    chars: Vec<char>,
    pos: usize,
}

impl InlineParser {
    fn new(input: &str) -> Self {
        InlineParser {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn current(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.chars.len());
    }

    /// Returns how many times `ch` repeats starting at `self.pos`.
    fn count_run(&self, ch: char) -> usize {
        self.chars[self.pos..].iter().take_while(|&&c| c == ch).count()
    }

    /// `self.chars[self.pos..]` starts with the characters of `s`.
    fn starts_with(&self, s: &str) -> bool {
        let sc: Vec<char> = s.chars().collect();
        let rem = self.chars.len() - self.pos;
        rem >= sc.len() && self.chars[self.pos..self.pos + sc.len()] == sc[..]
    }

    /// Find the next occurrence of `needle` (as char slice) from `self.pos`.
    /// Returns absolute index of the first character of the match.
    fn find_from(&self, needle: &[char]) -> Option<usize> {
        if needle.is_empty() {
            return Some(self.pos);
        }
        let end = self.chars.len().saturating_sub(needle.len() - 1);
        for i in self.pos..end {
            if self.chars[i..i + needle.len()] == *needle {
                return Some(i);
            }
        }
        None
    }

    /// Collect `self.chars[self.pos..end]` into a String, then set `self.pos = end + skip`.
    fn slice_and_advance(&mut self, end: usize, skip_after: usize) -> String {
        let s: String = self.chars[self.pos..end].iter().collect();
        self.pos = end + skip_after;
        s
    }

    // ── Main parsing loop ─────────────────────────────────────────────────────

    fn parse(mut self) -> Vec<Inline> {
        self.parse_until(None)
    }

    /// Parse inlines, stopping when `stop` matches at the current position
    /// (the stop sequence is NOT consumed).
    fn parse_until(&mut self, stop: Option<&[char]>) -> Vec<Inline> {
        let mut out: Vec<Inline> = Vec::new();
        let mut buf = String::new();

        loop {
            // Check stop condition
            if let Some(s) = stop {
                if !s.is_empty()
                    && self.pos + s.len() <= self.chars.len()
                    && self.chars[self.pos..self.pos + s.len()] == *s
                {
                    break;
                }
            }
            if self.at_end() {
                break;
            }

            let ch = self.current().unwrap();

            match ch {
                // ── Escape sequence ──────────────────────────────────────────
                '\\' if self.peek(1).map(is_ascii_punct).unwrap_or(false) => {
                    self.skip(1);
                    buf.push(self.advance().unwrap());
                }

                // ── Inline code span ─────────────────────────────────────────
                '`' => {
                    let ticks = self.count_run('`');
                    let saved = self.pos;
                    self.skip(ticks);
                    let needle: Vec<char> = std::iter::repeat('`').take(ticks).collect();
                    if let Some(close) = self.find_from(&needle) {
                        let raw = self.slice_and_advance(close, ticks);
                        // Collapse whitespace per CommonMark §6.1
                        let code = if raw.starts_with(' ')
                            && raw.ends_with(' ')
                            && raw.chars().any(|c| c != ' ')
                        {
                            raw[1..raw.len() - 1].to_string()
                        } else {
                            raw
                        };
                        flush(&mut buf, &mut out);
                        out.push(Inline::Code(code));
                    } else {
                        // Not a code span — output raw backticks
                        self.pos = saved;
                        for _ in 0..ticks {
                            buf.push('`');
                        }
                        self.skip(ticks);
                    }
                }

                // ── Emphasis / Strong / StrongEmphasis with * ────────────────
                '*' => {
                    if let Some(node) = self.try_delimited('*', &mut buf, &mut out) {
                        out.push(node);
                    }
                }

                // ── Emphasis / Strong / StrongEmphasis with _ ────────────────
                '_' => {
                    // _ only opens emphasis when not surrounded by word chars
                    let left_ok = self.pos == 0
                        || !self.chars[self.pos - 1].is_alphanumeric();
                    if left_ok {
                        if let Some(node) = self.try_delimited('_', &mut buf, &mut out) {
                            out.push(node);
                            continue;
                        }
                    }
                    buf.push('_');
                    self.skip(1);
                }

                // ── Strikethrough ~~ ─────────────────────────────────────────
                '~' if self.peek(1) == Some('~') => {
                    let saved = self.pos;
                    self.skip(2);
                    let needle = ['~', '~'];
                    if let Some(close) = self.find_from(&needle) {
                        let inner = self.slice_and_advance(close, 2);
                        let children = InlineParser::new(&inner).parse();
                        flush(&mut buf, &mut out);
                        out.push(Inline::Strikethrough(children));
                    } else {
                        self.pos = saved;
                        buf.push('~');
                        buf.push('~');
                        self.skip(2);
                    }
                }

                // ── Image ![alt](src) ────────────────────────────────────────
                '!' if self.peek(1) == Some('[') => {
                    let saved = self.pos;
                    self.skip(2); // skip ![
                    if let Some(img) = self.try_parse_bracket_link(true) {
                        flush(&mut buf, &mut out);
                        out.push(img);
                    } else {
                        self.pos = saved;
                        buf.push('!');
                        self.skip(1);
                    }
                }

                // ── Link [text](url) ─────────────────────────────────────────
                '[' => {
                    let saved = self.pos;
                    self.skip(1); // skip [
                    if let Some(link) = self.try_parse_bracket_link(false) {
                        flush(&mut buf, &mut out);
                        out.push(link);
                    } else {
                        self.pos = saved;
                        buf.push('[');
                        self.skip(1);
                    }
                }

                // ── Autolinks / raw HTML <…> ─────────────────────────────────
                '<' => {
                    let saved = self.pos;
                    if let Some(node) = self.try_parse_angle_bracket() {
                        flush(&mut buf, &mut out);
                        out.push(node);
                    } else {
                        self.pos = saved;
                        buf.push('<');
                        self.skip(1);
                    }
                }

                // ── Custom: #grad[from, to](text) ────────────────────────────
                '#' if self.starts_with("#grad[") => {
                    let saved = self.pos;
                    if let Some(node) = self.try_parse_gradient() {
                        flush(&mut buf, &mut out);
                        out.push(node);
                    } else {
                        self.pos = saved;
                        buf.push('#');
                        self.skip(1);
                    }
                }

                // ── Custom: #font[family](text) ──────────────────────────────
                '#' if self.starts_with("#font[") => {
                    let saved = self.pos;
                    if let Some(node) = self.try_parse_font() {
                        flush(&mut buf, &mut out);
                        out.push(node);
                    } else {
                        self.pos = saved;
                        buf.push('#');
                        self.skip(1);
                    }
                }

                // ── Hard break (two spaces + newline) or Soft break ──────────
                '\n' => {
                    if buf.ends_with("  ") {
                        while buf.ends_with(' ') {
                            buf.pop();
                        }
                        flush(&mut buf, &mut out);
                        out.push(Inline::HardBreak);
                    } else {
                        let trimmed = buf.trim_end_matches(' ').to_string();
                        buf = trimmed;
                        flush(&mut buf, &mut out);
                        out.push(Inline::SoftBreak);
                    }
                    self.skip(1);
                }

                // ── Regular character ─────────────────────────────────────────
                _ => {
                    buf.push(ch);
                    self.skip(1);
                }
            }
        }

        flush(&mut buf, &mut out);
        out
    }

    // ── Delimiter helpers ─────────────────────────────────────────────────────

    /// Try to parse `***`, `**`, or `*` (or `_` variant).
    /// Returns `None` and does NOT advance on failure.
    fn try_delimited(&mut self, delim: char, buf: &mut String, out: &mut Vec<Inline>) -> Option<Inline> {
        let count = self.count_run(delim).min(3);
        let saved = self.pos;
        self.skip(count);

        // Try from longest to shortest so `***` matches before `**`
        for try_count in (1..=count).rev() {
            let needle: Vec<char> = std::iter::repeat(delim).take(try_count).collect();
            if let Some(close) = self.find_from(&needle) {
                // Make sure close delimiter is not immediately followed by same delim
                // (avoids `**a**b**` greedily grabbing too much — simple heuristic)
                let inner = self.slice_and_advance(close, try_count);
                let children = InlineParser::new(&inner).parse();
                // If we consumed fewer than `count` delimiters, put extras back as text
                let extra = count - try_count;
                if extra > 0 {
                    flush(buf, out);
                    for _ in 0..extra {
                        out.push(Inline::Text(delim.to_string()));
                    }
                } else {
                    flush(buf, out);
                }
                return match try_count {
                    3 => Some(Inline::StrongEmphasis(children)),
                    2 => Some(Inline::Strong(children)),
                    _ => Some(Inline::Emphasis(children)),
                };
            }
        }

        // Nothing matched — restore position
        self.pos = saved;
        // Output the raw delimiter characters as text
        for _ in 0..count {
            buf.push(delim);
        }
        self.skip(count);
        None
    }

    // ── Link / Image ──────────────────────────────────────────────────────────

    /// Caller has already consumed `[` (link) or `![` (image).
    /// Parses `text](url "title")`.
    fn try_parse_bracket_link(&mut self, is_image: bool) -> Option<Inline> {
        // Find matching ] respecting nesting
        let bracket_start = self.pos;
        let mut depth = 1i32;
        let mut i = self.pos;
        while i < self.chars.len() && depth > 0 {
            match self.chars[i] {
                '\\' => i += 1, // skip escaped char
                '[' => depth += 1,
                ']' => depth -= 1,
                _ => {}
            }
            if depth > 0 { i += 1; }
        }
        if depth != 0 { return None; }
        let bracket_end = i; // index of ]

        // Require ( immediately after ]
        if self.chars.get(bracket_end + 1) != Some(&'(') {
            return None;
        }

        // Find matching )
        let paren_start = bracket_end + 2;
        let mut depth2 = 1i32;
        let mut j = paren_start;
        while j < self.chars.len() && depth2 > 0 {
            match self.chars[j] {
                '\\' => j += 1,
                '(' => depth2 += 1,
                ')' => depth2 -= 1,
                _ => {}
            }
            if depth2 > 0 { j += 1; }
        }
        if depth2 != 0 { return None; }
        let paren_end = j; // index of )

        let inner_text: String = self.chars[bracket_start..bracket_end].iter().collect();
        let dest: String = self.chars[paren_start..paren_end].iter().collect();
        let (url, title) = split_url_title(&dest);

        self.pos = paren_end + 1;

        if is_image {
            Some(Inline::Image { alt: inner_text, src: url, title })
        } else {
            let children = InlineParser::new(&inner_text).parse();
            Some(Inline::Link { children, url, title })
        }
    }

    // ── Angle bracket: autolink or raw HTML ──────────────────────────────────

    fn try_parse_angle_bracket(&mut self) -> Option<Inline> {
        let saved = self.pos;
        self.skip(1); // skip <

        // Find >
        let offset = self.chars[self.pos..].iter().position(|&c| c == '>')?;
        let inner: String = self.chars[self.pos..self.pos + offset].iter().collect();

        // Autolink: contains :// (URL) or @ without spaces (email)
        if inner.contains("://") || (inner.contains('@') && !inner.contains(' ') && !inner.is_empty()) {
            self.pos += offset + 1;
            let url = inner.clone();
            return Some(Inline::Link {
                children: vec![Inline::Text(inner)],
                url,
                title: None,
            });
        }

        // Raw HTML tag: starts with / (closing tag) or alpha (opening tag)
        let first = inner.chars().next();
        if first.map(|c| c.is_ascii_alphabetic() || c == '/').unwrap_or(false) {
            self.pos += offset + 1;
            return Some(Inline::RawHtml(format!("<{}>", inner)));
        }

        self.pos = saved;
        None
    }

    // ── Custom extensions ─────────────────────────────────────────────────────

    fn try_parse_gradient(&mut self) -> Option<Inline> {
        self.skip("#grad[".len());

        let bracket_end = self.chars[self.pos..].iter().position(|&c| c == ']')?;
        let colors: String = self.chars[self.pos..self.pos + bracket_end].iter().collect();
        let comma = colors.find(',')?;
        let from = colors[..comma].trim().to_string();
        let to = colors[comma + 1..].trim().to_string();
        self.skip(bracket_end + 1);

        if self.current() != Some('(') { return None; }
        self.skip(1);
        let paren_end = self.chars[self.pos..].iter().position(|&c| c == ')')?;
        let inner: String = self.chars[self.pos..self.pos + paren_end].iter().collect();
        self.skip(paren_end + 1);

        let children = InlineParser::new(&inner).parse();
        Some(Inline::Gradient { from, to, children })
    }

    fn try_parse_font(&mut self) -> Option<Inline> {
        self.skip("#font[".len());

        let bracket_end = self.chars[self.pos..].iter().position(|&c| c == ']')?;
        let family: String = self.chars[self.pos..self.pos + bracket_end]
            .iter()
            .collect::<String>()
            .trim()
            .to_string();
        self.skip(bracket_end + 1);

        if self.current() != Some('(') { return None; }
        self.skip(1);
        let paren_end = self.chars[self.pos..].iter().position(|&c| c == ')')?;
        let inner: String = self.chars[self.pos..self.pos + paren_end].iter().collect();
        self.skip(paren_end + 1);

        let children = InlineParser::new(&inner).parse();
        Some(Inline::Font { family, children })
    }
}
