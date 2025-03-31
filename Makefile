GITHUB_ORG ?= harmony-labs
GITHUB_TOKEN ?=

build:
	cargo build

dry-run:
	RUST_LOG=info cargo run -- --config config.yaml --token $(GITHUB_TOKEN) --dry-run

help:
	cargo run -- --help

list-repos:
	curl -L \
		-H "Accept: application/vnd.github+json" \
		-H "Authorization: Bearer $$GITHUB_TOKEN" \
		-H "X-GitHub-Api-Version: 2022-11-28" \
		"https://api.github.com/orgs/$(GITHUB_ORG)/repos?per_page=100" | jq -r '.[] | .git_url'

sync:
	cargo run -- --config config.yaml --token $(GITHUB_TOKEN)