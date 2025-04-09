CONFIG_FILE ?= config.yaml
GITHUB_ORG ?= harmony-labs
GITHUB_TOKEN ?=
RUST_LOG ?= info

.PHONY: build dry-run help list-repos sync sync-from-github diff

build:
	@cargo build

diff:
	@GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run -- diff $(CONFIG_FILE)

dry-run:
	@GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run -- sync $(CONFIG_FILE) --dry-run

help:
	@cargo run -- --help

list-repos:
	@curl -s -H "Authorization: Bearer $(GITHUB_TOKEN)" \
	"https://api.github.com/orgs/$(GITHUB_ORG)/repos?per_page=100" | jq -r '.[] | .git_url'

sync:
	@GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run -- sync $(CONFIG_FILE)

sync-from-github:
	@GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run -- sync-from-org $(CONFIG_FILE) --org $(GITHUB_ORG)

test:
	cargo test