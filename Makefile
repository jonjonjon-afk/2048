CARGO ?= cargo

.PHONY: check run demo e2e

# M1: formatting, lint and the full test suite must be green.
check:
	$(CARGO) fmt --check
	$(CARGO) clippy --all-targets -- -D warnings
	$(CARGO) test

run:
	$(CARGO) run --release

# Seeded demo mode arrives with the TUI in M2/M3.
demo:
	$(CARGO) run --release -- --demo --seed 42 --no-persist

# M1 end-to-end: the real binary's dump against the SPEC fixture plus the
# CLI parsing and storage tests. Frames are added in M2/M3.
e2e:
	$(CARGO) test --test cli -- --nocapture
