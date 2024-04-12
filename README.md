# ZenKat
*(Rocket Powered Crab Edition)*

[Find design notes here](dev-docs/design.md).

## Dependencies

- clap for argument parsing
- nom for parser combinator framework
- crossterm for rendering to terminal
- tokio for async support

## Architecture

Zenkat-rs is split into two processes: the **controller** and the **parser**. Currently the parser is in `/src/zk-parse` and the controller in `/src/zenkat`.
When you give ZenKat a command, it creates one or multiple instances of the parser and collects their output, then processes the output to form the data model.

This allows the parser to run asynchronously across many files at once. It also means that once the data transfer format between the controller and the parser becomes more stable, it should be possible to provide parser implementations which are written in completely different stacks.

## FAQs

### Why is this called ZenKat?

Because I couldn't remember the word ZEttelKAsTen.
