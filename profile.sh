#!/bin/bash

if ! test -f "`which flamegraph.pl 2> /dev/null`"; then
    echo 'Error: flamegraph.pl not found'
    echo
    echo 'Clone https://github.com/brendangregg/FlameGraph and place'
    echo 'flamegraph.pl and stackcollapse-perf.pl in your $PATH.'
    exit 1
fi

if test "$1" = --help; then
    echo "Usage: $0 [args...]"
    echo
    echo 'Specify optional arguments for the benchmark binary, such as'
    echo 'the name of the benchmark to run.'
    exit 0
fi

set -e

cargo bench --no-run
sudo perf record -g -- "`ls -tr target/release/benches-* | grep -v \\.d$ | tail -n 1`" "$@"
sudo chmod 644 perf.data
perf script > out.perf
stackcollapse-perf.pl < out.perf > out.folded
flamegraph.pl < out.folded > flamegraph.svg
rm -f perf.data{,.old} out.perf out.folded
echo
echo 'Output placed in flamegraph.svg'
