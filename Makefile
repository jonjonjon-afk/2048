CARGO ?= cargo

.PHONY: check run demo e2e

# M1: formatting, lint and the full test suite must be green.
check:
	$(CARGO) fmt --check
	$(CARGO) clippy --all-targets -- -D warnings
	$(CARGO) test

run:
	$(CARGO) run --release

# Seeded demo mode with the plasma and merge effects, in-memory score.
demo:
	$(CARGO) run --release -- --demo --seed 42 --no-persist

# M4 end-to-end: every integration suite (cli, effects, restart) plus three
# deterministic captures, each diffed against its committed fixture.
e2e:
	$(CARGO) test --tests
	mkdir -p docs/evidence/M4
	$(CARGO) run --quiet -- --dump --seed 42 > docs/evidence/M4/dump_seed42.txt
	$(CARGO) run --quiet -- --render-once --seed 42 --no-persist > docs/evidence/M4/frame_100x36_seed42.txt
	$(CARGO) run --quiet -- --render-once --demo-merge --frame 14 --no-persist > docs/evidence/M4/frame_100x36_demo_merge_14.txt
	diff -u tests/fixtures/dump_seed42.txt docs/evidence/M4/dump_seed42.txt
	diff -u tests/fixtures/frame_100x36_seed42.txt docs/evidence/M4/frame_100x36_seed42.txt
	diff -u tests/fixtures/effect_100x36_frame_14.txt docs/evidence/M4/frame_100x36_demo_merge_14.txt
