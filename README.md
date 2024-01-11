thigpen
==

> Hello and welcome to ACME. I'm the Chief.

`thigpen` is a tool for inspecting rust libraries, seeing the modules that make up the library, and the dependencies between those modules. There are many dependency graphing tools, but none of them inspect a single library's internals. The output of `thigpen` is a diagram that shows the flow of modules, the public interface of each module, and it's dependencies across the crate.

Usage
--

It's recommended to not install `thigpen` globally. Please try it out and report bugs (and request features), and provide patches. These early versions of `thigpen` are experimental pre-alpha at best, so please proceed with caution.

```shell
git clone git@github.com:rockstar/thigpen.git
cd thigpen
cargo build --release
./target/release/thigpen --help
```

By default, `thigpen $PATH` will output a mermaid graph to stdout. Specifying `-o` will specify an output file which you can send to the mermaid cli yourself.

Why thigpen?
--

[Lynne Thigpen](https://en.wikipedia.org/wiki/Lynne_Thigpen) was the voice of the Chief in "Where in the World is Carmen Sandiego?" (the television show as well as games). As she provided clues to help us find the thief, `thigpen` provides clues to help us find our way through rust crates.