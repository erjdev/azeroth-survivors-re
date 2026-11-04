# Companion fetching and obstacles

Fetch companions previously slid directly toward a pickup and retried the same
blocked direction. They now use the existing incremental local A* navigation for
both fetch and return trips, with a 0.35-yard body sweep and supported-floor
checks on route edges. Spawn waits for a supported, clear hero position. This
introduces no navmesh, physics replacement or new reward cost.

The route target is the pickup's supporting floor, not its half-yard hover
height. Collection uses three-dimensional distance and clear sight from the pet;
nearby coins behind a solid wall or on a distant storey cannot be banked through
geometry. Collection still reserves each pickup before deferred removal, keeps
the collecting companion's gold bonus, and leaves healing pickups and chests
alone. Blocked movement reports idle instead of playing a walking animation.

The actual `herd` system has a persistent-local-state regression with a supported
floor, a low finite fence and a hovering coin. It asserts that sight is clear
but direct body travel is blocked, then requires a detour around a fence end,
exactly one 11-gold settlement and a successful return to the hero. A second
regression compares a reachable coin with wall-occluded and vertically separated
coins. Existing two-companion ownership and death-gate regressions remain in scope.

Reproduce against checkout data:

```sh
WOW_DATA="$PWD/wow-data/Data" cargo test -p benilla-app --lib --features survivors-visibility survivors::companions::tests
```

These are headless runtime checks. They do not establish attended Elwynn routing,
all multi-storey interiors, crowded route performance or Windows visuals. No UI
changed and no new rendered screenshot is claimed. Bounded searches can fail on
unreachable destinations; broad navigation acceptance remains open.

Validation: all seven companion tests passed. The eight-package scoped gate
passed formatting, clippy and tests, with 31 optional addon-corpus skips.
`cargo survivors-build` passed; the final incremental rebuild took 15.95 seconds.
