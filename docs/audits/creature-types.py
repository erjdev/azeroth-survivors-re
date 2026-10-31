"""Audit authored roster identities; never infer runtime types from names/models.

Run from the repository root:
python3 docs/audits/creature-types.py path/to/creature_template.sql
The input is the pinned MaNGOS Zero catalog cited in SURVIVORS-LOCATIONS.md.
"""

import csv
import hashlib
import io
import pathlib
import re
import sys


PINNED_SHA256 = "447f007c603f4e80fb6732ced568170414b25f6aa62e5eb13a16d9f05281a2cc"
TYPES = {
    1: "Beast", 2: "Dragonkin", 3: "Demon", 4: "Elemental", 5: "Giant",
    6: "Undead", 7: "Humanoid", 8: "Critter", 9: "Mechanical",
    10: "Unspecified", 11: "Totem",
}
# Explicit authored boss -> archetype entry, from SURVIVORS-LOCATIONS.md.
# In particular, a large Shredder remains Mechanical, not Giant or Humanoid.
AUTHORED = {
    "Warsong Scout Captain": 12862,
    "Silverwing Vanguard": 12897,
    "Warsong Shaman Elder": 11683,
    "Silverwing Sentinel Captain": 12896,
    "Warsong Shredder Foreman": 11684,
    "Warsong Grunt Champion": 11682,
    "Silverwing Blade Champion": 14715,
    "Silverwing Field Marshal": 14715,
    "Warsong Gulch Warmaster": 12864,
    "Defiler Farm Raider": 2619,
    "Arathor Stable Captain": 2584,
    "Defiler Mine Overseer": 2621,
    "Arathor Lumber Champion": 2585,
    "Defiler Blacksmith Champion": 15128,
    "Arathor Cavalry Captain": 2738,
    "Defiler Siege Commander": 15128,
    "Arathor Field Marshal": 15130,
    "Defiler Basin Warmaster": 15128,
}

# Named identities with intentionally retained legacy displays. These mappings
# authorize a category, not a claim that the display belongs to that template.
LEGACY = {
    "Araj the Summoner": 1852, "Blighted Zombie": 4475,
    "Boulderfist Lord": 2571, "Boulderfist Ogre": 2562,
    "Burning Blade Neophyte": 3196, "Crypt Fiend": 8556,
    "Dabyrie Militia": 2581, "Drywhisker Kobold": 2572,
    "Flesh Golem": 1805, "Highland Strider": 2559,
    "Lord Falconcrest": 2597, "Lord Maldazzar": 1848, "Or'Kalar": 2773,
    "Plaguebat": 8600, "Plains Creeper": 2563, "Rotting Cadaver": 4474,
    "Rumbling Exile": 2592, "Scarlet Knight": 1833,
    "Shadowsworn Cultist": 6004, "Skeletal Flayer": 1783,
    "Skeletal Warrior": 48, "Syndicate Highwayman": 2586,
    "The Hederine Slayer": 7463, "Witherbark Axe Thrower": 2554,
}


def audit(catalog, source):
    if hashlib.sha256(catalog).hexdigest() != PINNED_SHA256:
        raise ValueError("Catalog differs from the pinned audit input")
    templates = []
    for line in catalog.decode().splitlines():
        if not line.startswith("("):
            continue
        row = next(csv.reader([line[1:].rstrip(",;")[:-1]],
                              quotechar="'", escapechar="\\"))
        if len(row) != 74:
            raise ValueError("Unexpected creature_template schema")
        templates.append(row)
    constructors = re.findall(
        r'\b(?:en|foe|giant)\(\s*"([^"]+)",\s*(\d+)', source)
    roster = sorted(set(constructors))
    if not roster:
        raise ValueError("No roster constructors found; inspect source syntax")
    annotations = re.findall(
        r'\b(?:en|foe|giant)\(\s*"([^"]+)",\s*(\d+)[^)]*\)'
        r'\s*\.typed\((\d+), CreatureType::(\w+)\)', source)
    annotated = {(name, display) for name, display, _, _ in annotations}
    if annotated != set(roster) or len(annotations) != len(constructors):
        raise ValueError("Missing explicit creature metadata")
    for name, display, entry, kind in annotations:
        matches = [r for r in templates if r[0] == entry]
        if len(matches) != 1 or TYPES[int(matches[0][13])] != kind:
            raise ValueError(f"Incorrect template/type metadata: {name}")
        row = matches[0]
        expected = AUTHORED.get(name, LEGACY.get(name))
        if expected is not None:
            if int(entry) != expected:
                raise ValueError(f"Incorrect authored identity: {name}")
        elif row[1] != name or display not in row[5:9]:
            raise ValueError(f"Unverified roster identity: {name}")
    output = io.StringIO()
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(["name", "display", "status", "entry", "creature_type",
                     "candidate_entries", "candidate_displays"])
    for name, display in roster:
        candidates = [r for r in templates if r[1] == name]
        exact = [r for r in candidates if display in r[5:9]]
        status = "exact"
        if name in AUTHORED:
            exact = [r for r in templates if int(r[0]) == AUTHORED[name]
                     and display in r[5:9]]
            if len(exact) != 1:
                raise ValueError(f"Authored archetype mismatch: {name}")
            status = "authored archetype"
        elif name in LEGACY and not exact:
            exact = [r for r in templates if int(r[0]) == LEGACY[name]]
            status = "legacy appearance"
        if len(exact) == 1:
            row = exact[0]
            writer.writerow([name, display, status, row[0], TYPES[int(row[13])], "", ""])
        else:
            writer.writerow([name, display, "unresolved", "", "",
                             ";".join(r[0] for r in candidates),
                             ";".join("/".join(x for x in r[5:9] if x != "0")
                                      for r in candidates)])
    return output.getvalue()


if __name__ == "__main__":
    print(audit(pathlib.Path(sys.argv[1]).read_bytes(),
                pathlib.Path("crates/survivors-core/src/data.rs").read_text()), end="")
