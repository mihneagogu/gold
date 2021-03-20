CARGO = cargo

run: 
	$(CARGO) run

run-release:
	$(CARGO) run --release

test:
	$(CARGO) test

test-release:
	$(CARGO) test --release



build:
	$(CARGO) build

build-release:
	$(CARGO) build --release

check:
	$(CARGO) check

