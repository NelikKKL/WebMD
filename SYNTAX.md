# WebMD Syntax Reference

WebMD is a [CommonMark](https://spec.commonmark.org/)-compatible Markdown renderer with [GFM](https://github.github.com/gfm/) (GitHub-Flavored Markdown) extensions and a small set of its own inline extensions. This document covers every supported construct.

---

## Table of Contents

- [Headings](#headings)
- [Paragraphs](#paragraphs)
- [Line Breaks](#line-breaks)
- [Emphasis](#emphasis)
- [Strikethrough](#strikethrough)
- [Code](#code)
- [Links](#links)
- [Images](#images)
- [Lists](#lists)
- [Task Lists](#task-lists)
- [Block Quotes](#block-quotes)
- [Tables](#tables)
- [Thematic Breaks](#thematic-breaks)
- [Raw HTML](#raw-html)
- [Escape Sequences](#escape-sequences)
- [WebMD Extensions](#webmd-extensions)

---

## Headings

### ATX-style headings

Use one to six `#` characters followed by a space.

```markdown
# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6
```

### Setext-style headings

Underline the text with `=` (level 1) or `-` (level 2).

```markdown
Heading 1
=========

Heading 2
---------
```

---

## Paragraphs

A paragraph is any block of text not preceded by a special marker. Separate paragraphs with a blank line.

```markdown
This is the first paragraph.

This is the second paragraph.
```

---

## Line Breaks

**Soft break** — a single newline inside a paragraph becomes a space in the output.

**Hard break** — end a line with two or more spaces, or a backslash `\`, to produce a `<br />`.

```markdown
Line one  
Line two (two trailing spaces above)

Line one\
Line two (backslash above)
```

---

## Emphasis

| Syntax | Result |
|---|---|
| `*italic*` or `_italic_` | *italic* |
| `**bold**` or `__bold__` | **bold** |
| `***bold italic***` or `___bold italic___` | ***bold italic*** |

```markdown
*italic text*
**bold text**
***bold and italic***
```

---

## Strikethrough

GFM extension. Wrap text in double tildes.

```markdown
~~strikethrough~~
```

---

## Code

### Inline code

Wrap text in backticks.

```markdown
Use `const x = 1;` inline.
```

Wrap in double backticks if the code contains a single backtick.

```markdown
`` There is a `backtick` inside ``
```

### Fenced code blocks

Open and close with three or more backticks (or tildes). Optionally specify a language after the opening fence.

````markdown
```javascript
function greet(name) {
  return `Hello, ${name}!`;
}
```
````

````markdown
~~~python
def greet(name):
    return f"Hello, {name}!"
~~~
````

### Indented code blocks

Indent every line with 4 spaces (or 1 tab).

```
    four-space indented code
    second line
```

---

## Links

### Inline links

```markdown
[link text](https://example.com)
[link with title](https://example.com "My title")
```

### Autolinks

Angle-bracket-wrapped URLs are passed through as raw HTML.

```markdown
<https://example.com>
```

---

## Images

```markdown
![alt text](image.png)
![alt text](image.png "Optional title")
```

---

## Lists

### Unordered lists

Use `-`, `*`, or `+` as the bullet character.

```markdown
- First item
- Second item
  - Nested item
  - Another nested
- Third item
```

### Ordered lists

Use a number followed by `.`. The start number is respected; subsequent numbers are ignored.

```markdown
1. First item
2. Second item
   1. Nested ordered
3. Third item
```

Starting from a specific number:

```markdown
7. Seventh item
8. Eighth item
```

### Loose lists

A blank line between list items produces a *loose* list — each item is wrapped in a `<p>`.

```markdown
- Item one

- Item two

- Item three
```

---

## Task Lists

GFM extension. Prefix a list item with `[ ]` (unchecked) or `[x]` (checked).

```markdown
- [x] Write the parser
- [x] Write the renderer
- [ ] Write the tests
- [ ] Publish the release
```

Works inside ordered lists too.

```markdown
1. [x] Design the API
2. [ ] Implement #grad extension
3. [ ] Write documentation
```

---

## Block Quotes

Prefix lines with `>`. Block quotes can be nested and can contain any block-level content.

```markdown
> This is a block quote.
>
> It can span multiple paragraphs.

> Outer quote
>
> > Inner (nested) quote
```

---

## Tables

GFM extension. Columns are separated by `|`. The second row defines alignment with colons.

```markdown
| Column A | Column B | Column C |
|----------|:--------:|---------:|
| left     | center   | right    |
| foo      | bar      | baz      |
```

| Separator | Alignment |
|-----------|-----------|
| `---` | None (default left) |
| `:---` | Left |
| `:---:` | Center |
| `---:` | Right |

Outer pipes are optional.

---

## Thematic Breaks

Three or more `-`, `*`, or `_` characters on a line (with optional spaces) produce a horizontal rule.

```markdown
---

***

___
```

---

## Raw HTML

HTML tags inside Markdown are passed through to the output. Block-level HTML tags start a raw HTML block.

```markdown
<div class="custom">
  Raw HTML block
</div>
```

Inline HTML also works inside paragraphs.

```markdown
This has <em>inline HTML</em> inside it.
```

---

## Escape Sequences

Prefix any ASCII punctuation character with `\` to output it literally.

```markdown
\*not italic\*
\# not a heading
\[not a link\](url)
```

Escapable characters: `! " # $ % & ' ( ) * + , - . / : ; < = > ? @ [ \ ] ^ _ ` { | } ~`

---

## WebMD Extensions

These extensions are unique to WebMD and are not part of CommonMark or GFM.

### Gradient text — `#grad`

Applies a horizontal CSS `linear-gradient` to text using `-webkit-background-clip`.

**Syntax**

```
#grad[<from>, <to>](text)
```

- `<from>` — any valid CSS color value for the left side of the gradient
- `<to>` — any valid CSS color value for the right side
- `text` — the visible label; may contain other inline markup

**Examples**

```markdown
#grad[red, blue](Rainbow heading)

#grad[#ff6b6b, #feca57](Sunset orange to yellow)

#grad[oklch(60% 0.25 280), oklch(60% 0.25 180)](Purple to teal)

This paragraph has #grad[tomato, dodgerblue](**bold gradient**) inside it.
```

**Rendered HTML**

```html
<span style="background: linear-gradient(to right, red, blue);
             -webkit-background-clip: text;
             -webkit-text-fill-color: transparent;
             background-clip: text;">Rainbow heading</span>
```

**Notes**

- Color values are passed directly to CSS — any valid CSS color syntax works: named colors, hex, `rgb()`, `hsl()`, `oklch()`, etc.
- The gradient direction is always left-to-right (`to right`).
- Nesting `#grad` inside another `#grad` is technically parsed but the inner fill-color will be invisible.

---

### Custom font — `#font`

Sets the `font-family` CSS property on the wrapped text.

**Syntax**

```
#font[<family>](text)
```

- `<family>` — any CSS font family name (quote it in CSS if it contains spaces, but no quotes are needed inside the brackets)
- `text` — the visible label; may contain other inline markup

**Examples**

```markdown
#font[Georgia](Rendered in Georgia)

#font[Comic Sans MS](Whimsical text)

#font[JetBrains Mono](Monospaced without a code block)

#font[system-ui](System default font)
```

**Rendered HTML**

```html
<span style="font-family: 'Comic Sans MS'">Whimsical text</span>
```

**Notes**

- Font family names with spaces do not need quoting inside the brackets — the renderer adds the CSS single quotes automatically.
- The font must be available in the browser (system font, web font, or previously loaded via `<link>`).
- `#font` and `#grad` can be combined by nesting:
  ```markdown
  #font[Georgia](#grad[gold, crimson](Styled and coloured))
  ```

---

## AST / Parse Tree

The `WebMD` class exposes `parse_to_json(markdown)`, which returns the document as a JSON array of block nodes. Useful for building custom renderers or tooling.

```javascript
import init, { WebMD } from './webmd.js';
await init();

const md = new WebMD();
const ast = JSON.parse(md.parse_to_json('# Hello\n\nWorld'));
// [
//   { type: "heading", level: 1, children: [{ type: "text", value: "Hello" }] },
//   { type: "paragraph", children: [{ type: "text", value: "World" }] }
// ]
```

### Block node types

| `type` | Extra fields |
|---|---|
| `heading` | `level` (1–6), `children` |
| `paragraph` | `children` |
| `blockquote` | `children` (blocks) |
| `list` | `ordered`, `start`, `tight`, `items` |
| `code_block` | `lang` (string or null), `code` |
| `table` | `headers`, `alignments`, `rows` |
| `thematic_break` | — |
| `html_block` | `html` |

Each `items` entry has `checked` (`true`, `false`, or `null`) and `children` (blocks).

`alignments` values: `"left"`, `"center"`, `"right"`, or `null`.

### Inline node types

| `type` | Extra fields |
|---|---|
| `text` | `value` |
| `soft_break` | — |
| `hard_break` | — |
| `code` | `value` |
| `emphasis` | `children` |
| `strong` | `children` |
| `strong_emphasis` | `children` |
| `strikethrough` | `children` |
| `link` | `url`, `title` (or null), `children` |
| `image` | `src`, `alt`, `title` (or null) |
| `raw_html` | `html` |
| `gradient` | `from`, `to`, `children` |
| `font` | `family`, `children` |

---

## Quick Reference

```markdown
# H1  ## H2  ### H3

*italic*  **bold**  ***bold italic***  ~~strike~~

`inline code`

```lang
fenced code block
```

[link](url)        [link](url "title")
![img](src)

- bullet  * bullet  + bullet
1. ordered

- [ ] unchecked task
- [x] checked task

> blockquote

| A | B |
|---|---|
| 1 | 2 |

---  ***  ___   ← thematic break

#grad[red, blue](gradient text)
#font[Georgia](custom font)
```
