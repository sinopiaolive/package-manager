#!/bin/bash

if ! test -f "`which flamegraph.pl 2> /dev/null`"; then
    echo 'Error: flamegraph.pl not found'
    echo
    echo 'Clone https://github.com/brendangregg/FlameGraph and place'
    echo 'flamegraph.pl and stackcollapse-perf.pl in your $PATH.'
    exit 1
fi

if test "$1" = --help -o "$1" = ""; then
    echo "Usage: $0 <client|server|lib> args..."
    echo
    echo 'Specify arguments for the test binary, such as the name of the'
    echo 'test case or benchmark to run.'
    echo
    echo 'Examples:'
    echo
    echo "$0 client foo_test"
    echo "$0 lib --bench --ignored bar_bench"
    echo
    echo 'Note that if you omit the --bench argument for a benchmark, it will'
    echo 'run the inner loop only once, resulting in inaccurate profiling data.'
    exit 0
fi

# From https://gist.github.com/cstorey/f7ccbc4b3e67fccdaf85158cce6ec811
function rust_demangle() {
    sed -e '
        s!\$C\$!,!g;
        s!\$SP\$!@!g;
        s!\$BP\$!*!g;
        s!\$RF\$!\&!g;
        s!\$LT\$!<!g;
        s!\$GT\$!>!g;
        s!\$LP\$!(!g;
        s!\$RP\$!)!g;
        s!\$u20\$! !g;
        s!\$u27\$!'\''!g;
        s!\$u2b\$!+!g;
        s!\$u5b\$![!g;
        s!\$u5d\$!]!g;
        s!\$u7e\$!~!g;
        s!\.\.!::!g;
    '
}

function disable_kptr_restrict() {
    # This is merely hardening against some forms of exploits, so it appears to
    # be OK to disable: https://wiki.ubuntu.com/Security/Features
    #
    # We could also run 'perf script' as root instead of doing this.
    if test -f /proc/sys/kernel/kptr_restrict; then
        kptr_restrict_orig="`cat /proc/sys/kernel/kptr_restrict`"
        sudo sh -c "echo 0 > /proc/sys/kernel/kptr_restrict"
    fi
}

function restore_kptr_restrict() {
    if test -f /proc/sys/kernel/kptr_restrict; then
        sudo sh -c "echo $kptr_restrict_orig > /proc/sys/kernel/kptr_restrict"
    fi
}

set -e

dir="$1" # client, server or lib
shift
if [ "$dir" = lib ]; then
    binary="pm_lib"
elif [ "$dir" = server ]; then
    binary="pm_server"
else # client
    binary="pm"
fi

cd "$dir"

cargo test --release --no-run

sudo perf record -g -- "$(ls -tr ../target/release/"$binary"-* | grep -v \\.d$ | tail -n 1)" --test-threads 1 "$@"
sudo chmod 644 perf.data
disable_kptr_restrict
perf script > out.perf
restore_kptr_restrict
stackcollapse-perf.pl < out.perf > out.folded
rust_demangle < out.folded > out.folded.demangled
flamegraph.pl < out.folded.demangled > ../flamegraph.svg
rm -f perf.data{,.old} out.perf out.folded{,.demangled}
echo
echo 'Output placed in flamegraph.svg'
