#!/usr/bin/env bash
#MISE description="Train the sentence-splitter model and write the source list"
#MISE depends=["splitter:fetch"]
# Writes data/splitter/en.tsv and data/splitter/SOURCES.md from the corpus that
# splitter:fetch wrote. Release mode: the trainer reads hundreds of megabytes.
exec cargo run --release --example train_splitter -- \
  --markdown target/splitter-corpus/repositories.txt \
  --markdown target/splitter-corpus/stackv2.txt \
  --text target/splitter-corpus/peps.txt \
  --text target/splitter-corpus/books.txt \
  --sources data/splitter \
  --model data/splitter/en.tsv \
  --report data/splitter/SOURCES.md
