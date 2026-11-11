#!/bin/sh
#MISE description="Add a release target's Rust standard library"
#USAGE arg "<triple>" help="The Rust target triple verctl builds"
# assets.prepare in .ctl/ver.yaml runs this once for each target, before the
# build. A declared prepare replaces verctl's stock one, which is the rustup
# line below. The C compiler for a musl target is zig, from mise.toml, which
# mise.toml hands to cc-rs; the runner's apt mirrors are not involved.
set -eu
rustup target add "${usage_triple}"
