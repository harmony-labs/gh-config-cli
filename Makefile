GITHUB_ORG ?= harmony-labs
GITHUB_TOKEN ?=
RUST_LOG ?= info

.PHONY: build dry-run help list-repos sync sync-from-github

build:
	@cargo build

dry-run:
	@GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run -- --config config.yaml --dry-run

help:
	@cargo run -- --help

list-repos:
	@curl -s -L \
	-H "Accept: application/vnd.github+json" \
	-H "Authorization: Bearer $(GITHUB_TOKEN)" \
	-H "X-GitHub-Api-Version: 2022-11-28" \
	"https://api.github.com/orgs/$(GITHUB_ORG)/repos?per_page=100" | jq -r '.[] | .git_url'

sync:
	@GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run -- --config config.yaml

sync-from-github:
	@GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run -- --config config.yaml --sync-from-org $(GITHUB_ORG) $(if $(filter true,$(DRY_RUN)),--dry-run,)