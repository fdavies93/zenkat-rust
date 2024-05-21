# Zenkat Roadmap

## Version 0.1

### Goals

Fundamental architecture in place. Provide basic input/output functionality. Correctly parse simple markdown documents and provide way to explore or visualise parse tree.

### Won't Do

- HTML parsing

### Commitments

#### Server

- Must run as a server with defined inputs and outputs in JSON

- `.` (select child) operator
- `*` (all) operator
- `**` (traverse) operator
- `[]` (data selector) operator

**Do we actually need a query language, or would it be simpler to just pass queries in as JSON?**

#### Block Parsing

- Horizontal rules (***, ---, ___)
- Setext headings (low)
- Indented code blocks (low)
- Fenced code blocks
- Link reference definitions (low)

#### Inline Parsing
- Emphasis and strong emphasis
- Links
- Backslash escapes
- Inline code
- Image embedding
- Hard line breaks (low)

### Bugs

- `parse_at_paths` crashes with paths involving `.`

### Complete

#### Server

- Async parsing of documents

#### Block Parsing

- ATX headings
- Paragraphs

#### Inline Parsing

#### Bugs

- Parser crashes when characters are not a single byte width (i.e. non-ascii)
- `walk` follows symlinks like normal directories (resolved in refactoring)

## Icebox
