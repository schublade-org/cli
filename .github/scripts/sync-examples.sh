#!/usr/bin/env bash
# Mirror each examples/<name> folder to schublade-org/examples-<name>.
# examples/ in this repo is the source of truth.
set -euo pipefail

ORG="${EXAMPLES_SYNC_ORG:-schublade-org}"
API="${GITHUB_API_URL:-https://api.github.com}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
EXAMPLES_DIR="$ROOT/examples"
WORKDIR="${EXAMPLES_SYNC_WORKDIR:-$(mktemp -d)}"
DRY_RUN="${EXAMPLES_SYNC_DRY_RUN:-0}"

fail_missing_token() {
  echo "::error::Repository secret EXAMPLES_SYNC_TOKEN is not set."
  echo
  echo "Add a personal access token as the repository secret named EXAMPLES_SYNC_TOKEN"
  echo "(Settings → Secrets and variables → Actions). Do not name it GITHUB_TOKEN —"
  echo "GitHub Actions already injects an automatic token under that name, and that"
  echo "automatic token cannot create or push other repositories in ${ORG}."
  echo
  echo "Required access:"
  echo "  Classic PAT:  repo  (create org repos + push). public_repo is not enough to create repositories."
  echo "  Fine-grained: resource owner ${ORG}; repository access All repositories (including future);"
  echo "                Administration = Read and write (create repos); Contents = Read and write (push)."
  echo "  The token owner must be allowed to create repositories in ${ORG}."
  echo
  echo "See README.md → Example repo sync."
  exit 1
}

if [[ -z "${EXAMPLES_SYNC_TOKEN:-}" ]]; then
  fail_missing_token
fi

if [[ ! -d "$EXAMPLES_DIR" ]]; then
  echo "::error::examples/ directory is missing at $EXAMPLES_DIR"
  exit 1
fi

api() {
  local method="$1"
  local path="$2"
  local body="${3:-}"
  if [[ -n "$body" ]]; then
    curl -sS -X "$method" \
      -H "Authorization: Bearer ${EXAMPLES_SYNC_TOKEN}" \
      -H "Accept: application/vnd.github+json" \
      -H "X-GitHub-Api-Version: 2022-11-28" \
      -H "Content-Type: application/json" \
      --data "$body" \
      "${API}${path}"
  else
    curl -sS -X "$method" \
      -H "Authorization: Bearer ${EXAMPLES_SYNC_TOKEN}" \
      -H "Accept: application/vnd.github+json" \
      -H "X-GitHub-Api-Version: 2022-11-28" \
      "${API}${path}"
  fi
}

repo_exists() {
  local name="$1"
  local code
  code="$(curl -sS -o /dev/null -w "%{http_code}" \
    -H "Authorization: Bearer ${EXAMPLES_SYNC_TOKEN}" \
    -H "Accept: application/vnd.github+json" \
    "${API}/repos/${ORG}/${name}")"
  [[ "$code" == "200" ]]
}

create_repo() {
  local name="$1"
  local example="$2"
  echo "Creating ${ORG}/${name}"
  local payload
  payload="$(cat <<EOF
{
  "name": "${name}",
  "description": "Mirror of ${ORG}/schublade examples/${example}. Source of truth: ${ORG}/schublade.",
  "homepage": "https://github.com/${ORG}/schublade/tree/main/examples/${example}",
  "private": false,
  "has_issues": false,
  "has_projects": false,
  "has_wiki": false,
  "auto_init": false
}
EOF
)"
  local response
  response="$(api POST "/orgs/${ORG}/repos" "$payload")"
  if echo "$response" | grep -q '"full_name"'; then
    return 0
  fi
  echo "::error::Failed to create ${ORG}/${name}"
  echo "$response"
  exit 1
}

push_example() {
  local example="$1"
  local repo="examples-${example}"
  local src="$EXAMPLES_DIR/$example"
  local dest="$WORKDIR/$repo"

  if [[ "$DRY_RUN" == "1" ]]; then
    echo "dry-run: would sync ${src} → ${ORG}/${repo}"
    return 0
  fi

  if ! repo_exists "$repo"; then
    create_repo "$repo" "$example"
  fi

  rm -rf "$dest"
  mkdir -p "$dest"
  tar -C "$src" -cf - . | tar -C "$dest" -xf -

  git -C "$dest" init --initial-branch=main
  git -C "$dest" config user.name "github-actions[bot]"
  git -C "$dest" config user.email "41898282+github-actions[bot]@users.noreply.github.com"
  git -C "$dest" add -A
  if git -C "$dest" diff --cached --quiet; then
    echo "No files to commit for ${repo} (empty example?)"
    exit 1
  fi
  git -C "$dest" commit --quiet -m "Sync examples/${example} from ${ORG}/schublade@${GITHUB_SHA:-local}"
  git -C "$dest" remote add origin "https://x-access-token:${EXAMPLES_SYNC_TOKEN}@github.com/${ORG}/${repo}.git"
  git -C "$dest" push --force origin HEAD:main
  echo "Pushed ${ORG}/${repo}"
}

examples=()
for dir in "$EXAMPLES_DIR"/*/; do
  [[ -d "$dir" ]] || continue
  name="$(basename "$dir")"
  examples+=("$name")
done

if [[ ${#examples[@]} -eq 0 ]]; then
  echo "::error::No example folders found under examples/"
  exit 1
fi

echo "Syncing ${#examples[@]} example(s): ${examples[*]}"
for name in "${examples[@]}"; do
  push_example "$name"
done
