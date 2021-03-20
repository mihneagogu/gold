CARGO = cargo

run: 
	$(CARGO) run

run-release:
	$(CARGO) run -- release

build:
	$(CARGO) build

build-release:
	$(CARGO) build --release

check:
	$(CARGO) check

