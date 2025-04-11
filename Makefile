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
	@GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run -- diff $(CONFIG_FILE)

dry-run:
	@GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run -- sync $(CONFIG_FILE) --dry-run

help:
	@cargo run -- --help

install:
	cargo install --path .

list-repos:
	@curl -s -H "Authorization: Bearer $(GITHUB_TOKEN)" \
	"https://api.github.com/orgs/$(GITHUB_ORG)/repos?per_page=100" | jq -r '.[] | .git_url'

sync:
	@GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run -- sync $(CONFIG_FILE)

sync-from-github:
	@GITHUB_TOKEN=$(GITHUB_TOKEN) RUST_LOG=$(RUST_LOG) cargo run -- sync-from-org $(CONFIG_FILE) --org $(GITHUB_ORG)

test:
	cargo test

# Update the API mapping table from the latest GitHub OpenAPI spec
update-github-api-mappings:
	curl -L -o github-openapi.json https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json
	cargo run --bin generate_api_mapping -- github-openapi.json > src/api_mapping_generated.rs
	@echo "Generated mapping table in src/api_mapping_generated.rs. Integrate as needed."
