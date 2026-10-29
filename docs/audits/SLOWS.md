# Independent movement slows

Ordinary slows previously combined the lowest speed multiplier with the longest
remaining duration, even when those values came from different spells. A hit-pipeline
regression reproduced a 50% slow lasting 10 seconds instead of its own 2.5 seconds
when paired with a 20% slow lasting 10 seconds.

Each target now retains one application per authored spell, secondary talent source,
and Aftermath proc. Movement chooses the strongest currently active source. On its
expiry a weaker source can remain for its own duration. Equal strengths last until
the final matching source expires. Refresh replaces only its source's strength and
expiry; older timestamped hits cannot overwrite newer applications.

Expiry follows active run time, so pausing does not consume duration. Existing boss
caps remain. Curse and channel effects remain independently owned and contribute to
the strongest effective slow. Death clears ordinary sources. Expired entries retain
the latest application timestamp until target despawn, preventing stale hits from
reviving replaced effects; storage is bounded by distinct authored sources.

Core tests cover overlap order, refresh, invalid inputs, equal strengths, timestamp
ordering and independent direct/secondary/proc sources. Runtime regressions use the
actual hit settlement and enemy movement systems. This is a solo-caster contract;
future multiplayer needs caster identity. Attended Windows combat remains unverified.
