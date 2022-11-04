Solutions to Advent of Code 2021.

This year, in an attempt to better learn what's in (and not in) Rust's standard
library, I imposed upon myself the arbitary restriction of using no external
crates other than ones I'd written. And aside from really missing `anyhow` and
occasionally wishing I had access to `itertools`, the experience was quite
nice.

Test cases and (my) problem inputs can be found under `data/`. Note that the
naming format of the files under `data/` is relied upon by some utility
functions.

Test with `cargo test --release`.
