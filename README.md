Hard Link Deduplicator
======================

`hld` finds the duplicated files and hardlinks them together in order to save
some disk space. And it's made to be fast!

Here is an example session on a modern (2017) laptop:

```fish
$ du -sh myproject ~/.m2
896M    myproject
912M    .m2
$ time hld -r -c ~/.m2 myproject
420.23 MB saved in the deduplication of 675 files
real 0.47
user 1.17
sys 0.22
```

420MB — 46% of the build directory size — saved in just 0.5 seconds :-)

[![CI Status](https://github.com/glehmann/hld/actions/workflows/ci.yml/badge.svg)]([text](https://github.com/glehmann/hld/actions))

Features
--------

It works with all the available core by default and uses the [BLAKE3](https://blake3.io/)
hashing function in order to be both very fast and with an extremely low
chance of collision.

Because of its caching feature, it is an efficient way to deduplicate files
that might have been copied by some automated process — for example a maven
build.

Usage
-----

#### globs

`hld` takes a set of globs as argument. The globs are used to find the
candidate files for deduplication. They support the `**` notation to traverse
any number of directories. For example:

* `hld "target/*.jar"` deduplicates all the `jar` files directly in the `target`
  directory;
* `hld "target/**/*.jar"` deduplicates all the `jar` files in the `target`
  directory and its subdirectories.

Several globs may be passed on the command line in order to work with
several directories and/or several file name patterns. For example:
`hld "target/*.jar" "images/**/*.png"`.

Note: the quotes are important to avoid the glob expansion by the shell.
In case of large directories, the shell may not be able to pass all the
files contained there.

#### caching

In addition to the raw globs of the previous chapter, some cached globs may
be used. They act all the same than the raw globs, but their BLAKE3 digest
value is saved for a latter reuse. They must be used on files that are
guarenteed to *not* change. Cached globs are passed with a `--cache`,
or `-c` option.

For example: `hld "target/*" --cache "stable/*"` will deduplicate
all the files in both `target` and `stable`, and will also cache the
digests of the files in `stable`. The cached digests of `stable` will
then be reused at a latter `hld` call, in order to speed up the execution.

The quotes are very important in this case: without them, the globs would
be expanded by the shell, and only the first file of the set would be
cached.

The cache path may be specified with the `--cache-path` option or `-C`,
in order to deal with several sets of caches, depending on the execution
context.

The cache may be cleared with the option `--clear-cache`.

#### recursive

The `--recursive` or `-r` option simplify the command line usage when working
with all the files in some directories. For example, the two following
commands are strictly equivalents:

```fish
hld -r -c ~/.m2 myproject
```

```fish
hld -c "$HOME/.m2/**/*" "myproject/**/*"
```

#### dry run

Using the option `--dry-run` or `-n` prevents `hld` to modify anytring on
the disk, cache included.

For example: `hld "target/*" --cache "stable/*" --dry-run` only show how many
files would be deduplicated and how much space would be saved, but actually
does nothing.

#### log level

The amount of output displayed by `hld` can be controlled by the `--log-level`
or `-l` option. It accepts the following values, from the most verbose to
the most quiet: `trace`, `debug`, `info` (the default level), `warn`, `error`.

#### parallelism

By default `hld` maximize the number of cores it is working on, in order to
complete its task as fast of possible. The `--parallel` or `-j` options let
you change the number of threads to run in parallel.

For example, `hld -j1 "myproject/*"` forces `hld` to run single threaded.

#### shell completion

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

Releasing
---------

In order to produce a small easy to download executable, just do a release
build followed by:

```
$ strip target/release/hld
$ upx --ultra-brute target/release/hld
```

Code coverage
-------------

The code coverage may be computed with [kcov](https://simonkagstrom.github.io/kcov/).
Make sure the `kcov` executable is in the `PATH` then run:

```fish
$ cargo test --features kcov -- --test-threads 1
```

The report is available in `target/x86_64-unknown-linux-gnu/debug/coverage/index.html`.

TODO
----

* factorize the computation of the digest in the cached and non cached files
* which duplicate do we keep when symlinking? The first one? From the caches if possible?
