CONFIG_FILE ?= config.yaml
GITHUB_ORG ?= harmony-labs
GITHUB_TOKEN ?=
RUST_LOG ?= info

.PHONY: build dry-run help list-repos sync sync-from-github diff

build:
	@cargo build

clean:
	cargo clean

diff:
	@GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run --bin gh-config -- diff $(CONFIG_FILE)

dry-run:
	@GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run --bin gh-config -- sync $(CONFIG_FILE) --dry-run

generate-api-mappings:
	# @GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run --bin generate-api-mappings -- github-openapi.json
	curl -L -o mappings/github-openapi.json https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json
	cargo run --bin generate-api-mappings -- mappings/github-openapi.json > src/github_api_mapping_generated.rs
	@echo "Mappings generated. Integrate as needed."

help:
	@cargo run -- --help

install:
	cargo install --path .

list-repos:
	@curl -s -H "Authorization: Bearer $(GITHUB_TOKEN)" \
	"https://api.github.com/orgs/$(GITHUB_ORG)/repos?per_page=100" | jq -r '.[] | .git_url'

sync:
	@GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run --bin gh-config -- sync $(CONFIG_FILE)

sync-from-github:
	@GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run --bin gh-config -- sync-from-org $(CONFIG_FILE) --org $(GITHUB_ORG)

test:
	cargo test

update-api-mappings: generate-api-mappings
