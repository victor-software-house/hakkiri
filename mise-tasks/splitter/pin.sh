#!/usr/bin/env bash
#MISE description="Pin the owner's public repositories for the splitter corpus"
# Lists the public repositories of the owner that are not forks, not archived
# and at most 10 MB, and writes each one's name, default-branch commit and
# license to data/splitter/repositories.tsv. The listing needs no credentials,
# so it holds only what anyone can read.
set -euo pipefail
export GIT_TERMINAL_PROMPT=0
owner=victor-software-house
listing=$(mktemp)
trap 'rm -f "$listing"' EXIT
page=1
while true; do
  rows=$(curl -fsSL "https://api.github.com/orgs/$owner/repos?type=public&per_page=100&page=$page")
  if [ "$(jq length <<<"$rows")" -eq 0 ]; then
    break
  fi
  jq -r '.[] | select(.visibility == "public" and .fork == false and .archived == false and .size <= 10240) | [.full_name, (.license.spdx_id // "none")] | @tsv' <<<"$rows" >>"$listing"
  page=$((page + 1))
done
LC_ALL=C sort "$listing" | while IFS=$'\t' read -r name license; do
  commit=$(git ls-remote "https://github.com/$name.git" HEAD | cut -f1)
  if [ -z "$commit" ]; then
    echo "no commit for $name" >&2
    exit 1
  fi
  printf '%s\t%s\t%s\n' "$name" "$commit" "$license"
done >data/splitter/repositories.tsv
