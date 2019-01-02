Hard Link Deduplicator
======================

[![Travis Status](https://api.travis-ci.com/glehmann/hld.svg?branch=master)](https://travis-ci.com/glehmann/hld)

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
* bash/zsh/fish completion
