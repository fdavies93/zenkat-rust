# Zenkat API Surface

All operations should be accessible via the API, including loading, unloading, deletion, etc.

## As a CLI tool

Either the `--query` parameter or the `--server` flag **must** be supplied.

`--query` takes input from one JSON file and processes it, outputting the result to `stdout` (or whatever it's piped into). Per UNIX standards, it can also use `--` to take input from `stdin`.

## As a HTTP Server

Zenkat has one POST route: `query`. This takes requests in the same format as the CLI `--query` parameter, This is to avoid a lot of boilerplate from transforming API routes into queries.

## Entity Types

**The store** is the top-level data structure which holds all trees and to which queries are directed.

**ZKs** are roots of a parsing tree, located at a given filesystem path. Typically they are directories or symlinks to directories.

**Documents** are specific `.md` files (and others may be allowable in future).

**Headers, paragraphs, lists, etc** are as defined in Markdown specifications. Note that ZenKat tends to prefer combining element types (e.g. ATX and Setext style headers) for the sake of making querying straightforward; therefore we prefer the abstract `header` over the more specific `h1`, `h2`, et al.

## Query Format

Queries to locate nodes or sets of nodes are given in a format similar to CSS selectors.

`header` selects all headers.

`header[rank=1]` selects all `h1` elements. Note that like XML / HTML, all arguments are parsed as strings.

`document[id=9001] > header` selects all headers which are children of document 9001.

## API Details

### JSON Format

### Operations

`load_zk` loads a new zk into memory from a given file path. This doesn't load the contents of files to increase flexibility.

`unload_zk` unloads a zk with the given path or id. It will also unload any documents that are part of the zk.

`load_docs` loads all documents which match the path selector. **This is the preferred way to load documents because it uses concurrency.** The most common use is `load_docs(document)` (i.e. load all).

`load_doc` loads a single document by id. **This doesn't use concurrency.**

`unload_docs` drops documents from memory.

`select` queries the store and retrieves matching nodes.
