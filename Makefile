default: frontend ## Debug build
	cargo build

clean: ## Clean all build artifacts and dependencies
	@cargo clean
	@yarn cache clean
	@-rm -rf browser/pkg
	@-rm ./src/cache_buster_data.json
	@-rm -rf ./static/cache/bundle
	@-rm -rf ./assets

check: ## Check for syntax errors on all workspaces
	cargo check --workspace --tests --all-features

coverage: migrate ## Generate HTML code coverage
	cargo tarpaulin -t 1200 --out Html

dev-env: ## Download development dependencies
	cargo fetch
	yarn install

doc: ## Prepare documentation
	cargo doc --no-deps --workspace --all-features

docker: ## Build docker images
	docker build -t mcaptcha/survey:master -t mcaptcha/survey:latest .

docker-publish: docker ## Build and publish docker images
	docker push mcaptcha/survey:master 
	docker push mcaptcha/survey:latest

frontend: ## Build frontend assets
	@yarn install
	@-rm -rf ./static/cache/bundle/
	@-mkdir -p ./static/cache/bundle/css/
	@yarn sass
	@yarn build
	@./scripts/bundle.sh

lint: ## Lint codebase
	cargo fmt -v --all -- --emit files
	cargo clippy --workspace --tests --all-features
	yarn lint

migrate: ## Run database migrations
	cargo run --bin tests-migrate

release: frontend ## Release build
	cargo build --release

run: default ## Run debug build
	cargo run

sqlx-offline-data: ## prepare sqlx offline data
	cargo sqlx prepare  \
		--database-url=${DATABASE_URL} -- \
		--all-features \
		--bin survey

test: frontend ## Run tests
	echo 'static/' && tree static || true
	echo 'tree/' && tree assets || true
	cargo test --all-features --no-fail-fast

xml-test-coverage: migrate  ## Generate cobertura.xml test coverage
	cargo tarpaulin -t 1200 --out Xml

help: ## Prints help for targets with comments
	@cat $(MAKEFILE_LIST) | grep -E '^[a-zA-Z_-]+:.*?## .*$$' | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
