Hard Link Deduplicator
======================

[![Travis Status](https://api.travis-ci.com/glehmann/hld.svg?branch=master)](https://travis-ci.com/glehmann/hld)

Install
-------

`hld` is currently only available from sources. To install it, you need
a [Rust installation](https://www.rust-lang.org/). `hld` compiles with rust
stable or newer. In general, `hld` tracks the latest stable release of the
Rust compiler.

```
$ git clone https://github.com/glehmann/hld
...
$ cd hld
$ cargo install
...
$ $HOME/.cargo/bin/hld --version
hld 0.1.0
```

Shell completion
----------------

`hld` can generate the completion code for several shells (fish, zsh, bash, …).
Just run it with the `--completion` option followed by the shell type, and save
the produce code in the appropriate location. For example, for fish:

```fish
hld --completion fish > ~/.config/fish/completions/hld.fish
```

The completion is usually activated in the new shell instances, but may be
activated by sourcing the file. Again for fish:

```fish
source ~/.config/fish/completions/hld.fish
```

Building
--------

You need a [Rust installation](https://www.rust-lang.org/). `hld` compiles
with rust stable or newer. In general, `hld` tracks the latest stable release
of the Rust compiler.

To build `hld`:

```
$ git clone https://github.com/glehmann/hld
...
$ cd hld
$ cargo build --release
...
$ ./target/release/hld --version
hld 0.1.0
```

Testing
-------

To run the full test suite, use:

```
$ cargo test
...
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

```

from the repository root.


TODO
----

* complete the README.md
* factorize the computation of the digest in the cached and non cached files
* ensure that the newest date is kept on the hardlinked files (probably)
* find a better way to pass user options without changing the function signature
  at each new option
* code coverage (look at codecov and coveralls)
