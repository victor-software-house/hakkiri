#!/usr/bin/env bash
#MISE description="Fetch the splitter's training text from the pinned sources"
# Reads data/splitter/repositories.tsv, datasets.tsv and licenses.txt and writes
# target/splitter-corpus/{repositories,issues,books}.txt: documents separated by
# a form feed. A repository is fetched without credentials, so a private one
# fails. A run on the same pins writes the same files.
set -euo pipefail
export GIT_TERMINAL_PROMPT=0
export LC_ALL=C
out=target/splitter-corpus
rm -rf "$out"
mkdir -p "$out"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

while IFS=$'\t' read -r name commit _license; do
  dir=$work/${name//\//_}
  echo "fetching $name" >&2
  git init -q "$dir" </dev/null
  git -C "$dir" -c credential.helper= fetch -q --depth 1 "https://github.com/$name.git" "$commit" </dev/null
  git -C "$dir" -c advice.detachedHead=false checkout -q FETCH_HEAD </dev/null
  git -C "$dir" ls-files -z -- '*.md' | sort -z | while IFS= read -r -d '' file; do
    if [ -L "$dir/$file" ] || [ "${file##*/}" = CHANGELOG.md ]; then
      continue
    fi
    cat "$dir/$file"
    printf '\f'
  done
done <data/splitter/repositories.tsv >"$out/repositories.txt"

allowed=$(jq -R -s 'split("\n") | map(select(length > 0) | ascii_downcase)' data/splitter/licenses.txt)
: >"$out/stackv2.txt"
: >"$out/peps.txt"
: >"$out/books.txt"
while IFS=$'\t' read -r dataset revision file bytes _license; do
  url=https://huggingface.co/datasets/$dataset/resolve/$revision/$file
  prefilter=''
  case $dataset in
    common-pile/stackv2)
      target=stackv2.txt
      # A cheap cut before jq: only a line whose path may end in .md or
      # .markdown is parsed. The jq filter below decides.
      prefilter='"path": *"[^"]*\.(md|markdown)"'
      # metadata.license joins the licenses detected in the repository with
      # commas; every one must be on the Blue Oak list.
      filter='select(((.metadata.path // "") | test("\\.(md|markdown)$"; "i")) and (((.metadata.license // "") | ascii_downcase | split(",")) | all(IN($allowed[])))) | .text, "\f"'
      ;;
    common-pile/python_enhancement_proposals)
      target=peps.txt
      filter='select(.metadata.license == "Public Domain") | .text, "\f"'
      ;;
    common-pile/project_gutenberg)
      target=books.txt
      filter='select(.metadata.license == "Public Domain" and .metadata.language == "en") | .text, "\f"'
      ;;
    *)
      echo "unknown dataset $dataset" >&2
      exit 1
      ;;
  esac
  if [ "$bytes" -gt 0 ]; then
    curl -fsSL --retry 3 -r "0-$((bytes - 1))" -o "$work/shard.gz" "$url"
    # The range ends inside the gzip stream. gzip reports the cut and jq drops
    # the line it cut, so both are expected here.
    gzip -dc "$work/shard.gz" 2>/dev/null || true
    rm -f "$work/shard.gz"
  else
    curl -fsSL --retry 3 "$url" | gzip -dc
  fi | grep -aE "$prefilter" | jq -R -j --argjson allowed "$allowed" "fromjson? | $filter" >>"$out/$target"
done <data/splitter/datasets.tsv

for file in repositories stackv2 peps books; do
  if [ ! -s "$out/$file.txt" ]; then
    echo "$out/$file.txt is empty" >&2
    exit 1
  fi
done
