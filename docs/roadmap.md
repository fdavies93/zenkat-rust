# Zenkat Roadmap

## Version 0.1

### Goals

Fundamental architecture in place. Provide basic input/output functionality. Correctly parse simple markdown documents and provide way to explore or visualise parse tree.

### Won't Do

- HTML parsing

### Commitments

#### CLI Tool

- Equivalent to `zenkat list` must work and display some basic information such as word count.
- `zenkat outline` should work *better* than in previous version, since it's about manipulating ASTs.

#### Server

- Must run as a server with defined inputs and outputs in JSON
- Parse queries as JSON and return output
  - load_zk
  - unload_zk
  - load_docs
  - load_doc
  - unload_docs
  - select
- Query parsing
  - Select by node type, `header`
  - Select by data attribute, `header[rank=1]`
  - Descendent operator, `document[id=9001] header`
  - Child operator, `document[id=9001] list > list_item`

#### Parser

- ID should be in data so it's searchable

- Block Parsing
  - Horizontal rules (***, ---, ___)
  - Setext headings
  - Indented code blocks
  - Fenced code blocks
  - Link reference definitions (may delay)

- Container parsing
  - Block quotes
  - Lists
    - List items
      - Ordered list items `1[.)]`
      - Unordered list items `[*-+]`

- Inline parsing
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
