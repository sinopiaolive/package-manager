#!/bin/bash

if ! test -f "`which flamegraph.pl 2> /dev/null`"; then
    echo 'Error: flamegraph.pl not found'
    echo
    echo 'Clone https://github.com/brendangregg/FlameGraph and place'
    echo 'flamegraph.pl and stackcollapse-perf.pl in your $PATH.'
    exit 1
fi

if test "$1" = --help -o "$1" = ""; then
    echo "Usage: $0 args..."
    echo
    echo 'Specify arguments for the test binary, such as the name of the'
    echo 'test case or benchmark to run.'
    echo
    echo 'Examples:'
    echo
    echo "$0 foo_test"
    echo "$0 --bench bar_bench"
    echo
    echo 'Note that if you omit the --bench argument for a benchmark, it will'
    echo 'run the inner loop only once, resulting in inaccurate profiling data.'
    exit 0
fi

set -e

cargo test --release --no-run
sudo perf record -g -- "`ls -tr target/release/package_manager-* | grep -v \\.d$ | tail -n 1`" --test-threads 1 "$@"
sudo chmod 644 perf.data
perf script > out.perf
stackcollapse-perf.pl < out.perf > out.folded
flamegraph.pl < out.folded > flamegraph.svg
rm -f perf.data{,.old} out.perf out.folded
echo
echo 'Output placed in flamegraph.svg'
