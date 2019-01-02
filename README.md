Hard Link Deduplicator
======================

[![Travis Status](https://api.travis-ci.com/glehmann/hld.svg?branch=master)](https://travis-ci.com/glehmann/hld)

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

TODO
----

* complete the README.md
* factorize the computation of the digest in the cached and non cached files
* test the --recursive option
* test the --dry-run option
* test the --parallel option
* ensure that the newest date is kept on the hardlinked files (probably)
* find a better way to pass user options without changing the function signature
  at each new option
* code coverage (look at codecov and coveralls)
