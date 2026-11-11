#!/bin/sh
#MISE description="Add a release target's Rust standard library, and musl-tools for a musl target"
#USAGE arg "<triple>" help="The Rust target triple verctl builds"
# assets.prepare in .ctl/ver.yaml runs this once for each target, before the
# build. A declared prepare replaces verctl's stock one, which is the rustup
# line below. cc-rs compiles tree-sitter's C for a musl target with
# musl-gcc, which the runner does not have, so the musl target installs it.
set -eu
rustup target add "${usage_triple}"
case "${usage_triple}" in
  *-linux-musl)
    sudo apt-get update
    sudo apt-get install --yes --no-install-recommends musl-tools
    ;;
esac
