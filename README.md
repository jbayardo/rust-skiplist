Skip List implementation for Rust. Currently **under very active development**; please do NOT use (this is not in crates.io for a reason!).

# Code Status [![Build Status](https://travis-ci.org/jbayardo/rust-skiplist.svg?branch=master)](https://travis-ci.org/jbayardo/rust-skiplist) [![Windows Build Status](https://ci.appveyor.com/api/projects/status/5wd0sbesdncwp80d?svg=true)](https://ci.appveyor.com/project/jbayardo/rust-skiplist) 

[![Test coverage](https://codecov.io/gh/jbayardo/rust-skiplist/branch/master/graph/badge.svg)](https://codecov.io/gh/jbayardo/rust-skiplist)

[![Percentage of issues still open](http://isitmaintained.com/badge/open/jbayardo/rust-skiplist.svg)](http://isitmaintained.com/project/jbayardo/rust-skiplist "Percentage of issues still open") [![Average time to resolve an issue](http://isitmaintained.com/badge/resolution/jbayardo/rust-skiplist.svg)](http://isitmaintained.com/project/jbayardo/rust-skiplist "Average time to resolve an issue") 

Missing work:
* Mutable range iterators (easy)
* Tests for all iterators (easy)
* More testing would do great. Node is an easy example. The linked list needs more tests too
* Testing for memory leaks would be good too.
* You can try compiling on stable and testing what needs to be done to make it compatible
* It would be good to add some statistical testing to the HeighControl to ensure output is distributed as expected
* This can be turned into a lock-free dictionary, just need proper atomics support and some work (hard)

# Releases

Library releases follow [semantic versioning](http://semver.org/). Versioned releases can be found in Github's [release manager](https://github.com/jbayardo/rust-skiplist/releases).

Notice that this project **requires** Rust nightly to work. This is due to unavailable primitives in stable and beta channels; work is underway to make it compatible with beta and stable channels, but may take a while. 

# Reporting issues

Please report all issues on the Github [issue tracker](https://github.com/jbayardo/rust-skiplist/issues). [Due dilligence](https://contribution-guide-org.readthedocs.io/#due-diligence) is expected, and please include [all relevant information](https://contribution-guide-org.readthedocs.io/#what-to-put-in-your-bug-report).

# Contributing

All code contributed must pass through [Clippy](https://github.com/rust-lang-nursery/rust-clippy) and [Format](https://github.com/rust-lang-nursery/rustfmt), and no code will be merged unless it is thoroughly tested. Please look at the [Rust Book](https://doc.rust-lang.org/book/second-edition/ch11-03-test-organization.html) if you are not sure how to do this.

Changes for performance improvement must include benchmark results to back the claim. New dependencies are to be avoided; this library is expected to be as dependency-free as possible. 

# License

See [LICENSE.md](https://github.com/jbayardo/rust-skiplist/blob/master/LICENSE).