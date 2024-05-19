# ZenKat Architecture

The following aims to describe the ZenKat architecture **succinctly** for maintainer reference.

## Server

This is responsible for collecting data from the parsers and aggregating them into an AST for a whole Zettelkasten.

It can also be responsible for performing AST manipulations by running in daemon / LSP mode. By keeping it alive, it can also watch the Zettelkasten for updates or even commit updates to disk. I.e. it can maintain a working tree for individual clients and a main tree for what's on disk.

Should be able to maintain multiple ASTs for different knowledge bases.

Needs to be implemented in a compiled language for speed and interoperability with external programs (e.g. nvim plugins).

## Parsers

Parsers are responsible for parsing *single documents*. They're separate from the server so that parsing can be parallelised across a large number of documents.

The other advantage of separating parsers from the server is that it allows different Markdown flavors (e.g. CommonMark, GFM, MMD) to be implemented as different parsers and simply plugged into the server via a common interface.

Ideally implemented in a compiled language for speed, but less essential than the controller.

## Clients

These should access the server to perform operations. Can be written in any language.

Examples:
- Data explorer (basically the same as zenkat-python)
- Task manager (todo list manipulation)
- Vim plugins
