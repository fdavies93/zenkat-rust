# ZenKat
*(Rocket Powered Crab Edition)*

*We haven't reached v0.1 release, so this is basically a developer preview.*

## Dependencies

Once the project is more mature, it would be good to reduce dependencies on
these where practical.

- clap for argument parsing
- nom for creating AST from markdown
- crossterm for rendering to terminal
- tokio for async support
- reqwest for making requests
- axum for web server handling
- serde / serde_json for json parsing

## Architecture

Zenkat ships as a *collection* of utilities. Once we reach a v0.1 release these should be neatly wrapped as the `zenkat` utility, in a similar way to how `apt` wraps `apt-get`, `apt-cache` and others.

This simplifies parallelising parsing, but also has benefits for code structure.

The following are the utilities targeted for the v0.1 release.

### zenkat

A wrapper for the other utilities in the bundle designed to make the apps easier to use.

### zk-serve

The main server utility for Zenkat. Allows for querying and manipulation of documents in one or more ZKs / vaults.

For v0.1 release this will be a HTTP server taking JSON requests, but we'd like to allow other I/O methods such as LSP-mode in the future.

### zk-cmd

A CLI client utility which allows testing the capabilities of `zk-serve` using the same interface as other client utilities. This should be considered similar to the `mysql` client and other database interfaces.

### md-parse

The main parser for Zenkat. Takes a **single** markdown document and parses it into a n-tree structure.

For v0.1 the goal is to support basic Markdown features listed in [CommonMark](https://commonmark.org/), although not to perfectly implement the CommonMark spec.

## FAQs

### Why is this called ZenKat?

Because I couldn't remember the word ZEttelKAsTen.
