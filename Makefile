# Makefile for zukan: convenience wrappers around cargo + asset management.
#
# Asset lifecycle:
#   make download   fetch the latest zukan-assets Release into assets/{data,icons}
#   make clean      wipe the cargo build cache AND the cached assets
#
# Build / dev:
#   make check      clippy + fmt check (fast, run before committing)
#   make test       unit + integration tests
#   make build      debug build
#   make release    optimized release build
#   make doc        build rustdoc
#   make cov        coverage report (needs `cargo install cargo-llvm-cov`)
#   make all        check + test + build

ASSETS_URL := https://github.com/lazywalker/zukan-assets/releases/latest/download/zukan-assets-bin.tar.gz
RELEASE    := target/release/zukan

# Default: print available targets (don't silently do nothing).
.DEFAULT_GOAL := help

.PHONY: help check test build release doc cov clean download all assets

help: ## Show this help
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

##@ Quality

check: ## clippy + fmt check (run before committing)
	cargo clippy --all-features --tests -- -D warnings
	cargo fmt --all -- --check

test: ## unit + integration tests
	cargo test --bins --tests

##@ Build

build: ## debug build
	cargo build

release: ## optimized release build
	cargo build --release
	@ls -lh $(RELEASE) | awk '{print "  -> " $$5 "  " $$9}'

doc: ## build rustdoc (warnings as errors)
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --package zukan

cov: ## coverage report (needs cargo-llvm-cov)
	cargo llvm-cov --bins --tests --summary-only

##@ Assets

# Guard: most targets need the assets present. Build.rs would download them
# via ureq on demand, but that path can't be exercised from sandboxed CI; the
# `download` target uses curl+tar (reliable everywhere curl exists).
assets: ## ensure assets/{data,icons} exist (download if missing)
	@if [ -d assets/data ] && [ -d assets/icons ]; then \
		echo "assets/ already populated"; \
	else \
		$(MAKE) download; \
	fi

download: ## fetch the latest zukan-assets Release into assets/
	@echo "==> downloading $(ASSETS_URL)"
	@mkdir -p assets
	@curl -fsSL "$(ASSETS_URL)" | tar xz -C assets
	@echo "==> assets ready: $$(find assets -name '*.png' | wc -l) icons, $$(ls assets/data/*.json | wc -l) data files"

##@ Maintenance

clean: ## wipe cargo build cache + cached assets
	cargo clean
	rm -rf assets/data assets/icons assets/.download-partials
	@echo "==> cleaned target/ and assets/{data,icons}"

# Composite convenience target.
all: check test build ## check + test + build
