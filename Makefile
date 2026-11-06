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

# M3 end-to-end: the real binary's dump/render output plus the CLI, storage and
# pinned demo-merge frame captures under docs/evidence/M3/.
e2e:
	$(CARGO) test --test cli -- --nocapture
	mkdir -p docs/evidence/M3
	$(CARGO) run --quiet -- --render-once --demo-merge --frame 4 --no-persist > docs/evidence/M3/frame_04.txt
	$(CARGO) run --quiet -- --render-once --demo-merge --frame 14 --no-persist > docs/evidence/M3/frame_14.txt
	$(CARGO) run --quiet -- --render-once --demo-merge --frame 24 --no-persist > docs/evidence/M3/frame_24.txt
