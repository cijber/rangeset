[![Workflow Status](https://github.com/cijber/rangeset/workflows/rust%2Eyml/badge.svg)](https://github.com/cijber/rangeset/actions?query=workflow%3A%22rust%2Eyml%22)
[![Coverage Status](https://codecov.io/gh/cijber/rangeset/branch/master/graph/badge.svg)](https://codecov.io/gh/cijber/rangeset)

# eater_rangeset

A simple library with some boilerplate code to work with Range's and RangeSet's

Currently every set operation depends on a mix of [`RangeSet::union`](RangeSet::union) and
[`RangeSet::invert`](RangeSet::invert) as those are the only 2 operations needed to implement
all other operations.

This also reduces surface for errors, as only 2 functions have to be proven correct.

This will change in the future
