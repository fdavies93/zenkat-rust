# Zenkat Roadmap

## Working Version 0.1

**Goal:** Fundamental architecture in place. Provide basic input/output functionality. Correctly parse simple markdown documents and provide way to explore or visualise parse tree.

- Separate parsing and control tools for async parsing.
- Transmit data from parsers to control, likely via DB representation or JSON documents. Probably can't rely on this fitting in memory trivially.
- Direct compilation of parser modules into the parser, but leaving the door open to pre-compiled modules by providing a slightly abstracted interface.
  - Some kind of remote tooling could allow for pre-building parsers elsewhere and shipping them down to client machines.
  - This is probably good enough for our purposes.
  - Patching issue still needs resolution. Ideal way probably to write a parser combinator which can be configured via an external file.
  - If properly architected then **the combinator file can be a de-facto config**.
  - https://github.com/rust-bakery/nom
- Default markdown module should work on most simple documents.
- Should allow traversal of parse tree for debug / exploration purposes, from the knowledge base level down to inline elements.

## Icebox
