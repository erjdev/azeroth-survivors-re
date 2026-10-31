# Enemy pursuit of visible pets

`move_enemies` previously cleared navigation whenever the chosen Hunter pet was
visible. Visibility uses a ray one yard above the feet; movement uses the body's
capsule. A low fence can therefore pass the visibility check while stopping
movement. Clearing navigation every frame also clears the accumulated blocked
time, preventing the 0.35-second threshold for starting a detour search.

The correction retains navigation while pursuing the pet. Existing
waypoint logic invalidates routes when their destination moves more than six yards;
death, crowd control, fear, recycling and map changes retain their cleanup paths.
No new collision geometry, navmesh or IDA* algorithm is introduced.

The regression uses the actual `move_enemies` system with persistent local state,
a supported floor, a 0.8-yard-high fence and a living taunting pet. It independently
asserts that sight is clear and a direct body sweep is blocked. Success requires
walking around an end of the fence without climbing it, then damaging the pet.
This is headless movement evidence, not a rendered or attended Elwynn playtest.

The regression fails on the previous code after 600 movement frames (30 simulated
seconds), with sight/body-clearance preconditions passing. The same regression passes with the
correction, proving the enemy reaches and damages the pet via the detour. No UI source changed and no new screenshot is
claimed for this headless collision regression.

Validation: the eight-package formatting, Clippy and test gate passes with the
checkout data. The usual 31 optional-addon tests skip for the missing corpus.
