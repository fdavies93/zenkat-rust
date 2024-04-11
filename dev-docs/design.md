# Zenkat Design Notes

## View and Model Separation

The Python version of Zenkat suffered from closely associating its data models with how things were rendered. This was particularly the case with templates and formatting on the view layer, which drew on the Rich console library.

The Rust version should achieve a full separation of the data model from the presentation model, and the presentation model itself should be layered such that templates, styling, and console manipulation are separated.

## Parsing System

Zenkat-Rust should support parsing Markdown in a variety of dialects, with users able to both select and extend the dialects supported by the parser.

This parser should emit an AST for defined dialects and be able to incorporate extended dialects into the normal parsing process using the "New Jersey" model. Ideally the baseline parser should just be another dialect, likely a CommonMark implementation or close to CommonMark. GFM extensions would be the highest priority subsequent dialect, with others to follow.

This presents technical challenges:

- How can we run a plugin-based parser while maintaining good performance?
- Should we handwrite our parser or generate it from a PEG?
- How can dialects successfully stack on top of each other and maintain a coherent structure?
- How can we ensure that dialects are compatible, or where they're incompatible at least have a good way to check this?

**Plugins with good performance.** Likely the best solution is to compile dynamic modules which have some mechanism to patch into a core interface which supports them via dependency injection or similar mechanisms. Dialects can be authored as config files and a compiled module with a standard interface.

Solutions based on transmitting data via JSON or pipes have an appeal in terms of code hygiene and separation of concerns, but as the parser should work well over thousands of files at once, interprocess communication at this low a level is likely to lead to unacceptable performance issues.

Solutions based on shared memory space would likely offer good performance, but would be extremely challenging to maintain versus using dynamic loading.

Another interesting possibility is simply *compiling modules directly into the Rust parser* using a simple module system, since the parser is likely to be a separate process to the coordinating program anyway. However this requires end users to have the Rust compiler toolchain installed.

**Handwritten or generated parser?** As per [this discussion](https://talk.commonmark.org/t/commonmark-formal-grammar/46/18), it's at a minimum extremely challenging and perhaps impossible to write a formal specification for CommonMark. Markdown in general is often ambiguous and most dialects are "good enough" implementations in practice.

Handwritten parsers are also substantially easier to reason about and debug than those generated from PEGs, so as a possible community project it makes good sense to go for the option that's more subject to code review and more readable.

**Dialect stacking** requires some thought and experimenting. Fundamentally, there are only a few operations that need to be extended:

- Possibilities and priorities for parsing at a given level (blocks, inline)
- What's emitted from given blocks to the AST

Both should be able to be added to and 

## Asynchronous Parsing

Ideally different files would be able to be parsed asynchronously for good performance over large knowledge bases. This is unlikely to be a priority in early versions, but it would be good to build with it as an eventual goal.

It's likely that the implementation of this is also not going to be particularly challenging, in contrast to the module system.

Since a single file can easily be parsed by a single process with little overhead (barring synthetically long or convoluted files), parsing can just be a subprocess which is collected by the parent process.

## References

- [Plugin Tech in Rust](https://nullderef.com/blog/plugin-tech/)
