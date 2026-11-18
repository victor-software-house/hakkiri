#!/bin/sh
#MISE description="Add a release target's Rust standard library"
#USAGE arg "<triple>" help="The Rust target triple verctl builds"
# assets.prepare in .ctl/ver.yaml runs this once for each target, before the
# build. A declared prepare replaces verctl's stock one, which is the rustup
# line below. The C compiler for a musl target is zig, which mise.toml
# installs and passes to cc-rs. The runner's apt mirrors play no part.
set -eu
rustup target add "${usage_triple}"
