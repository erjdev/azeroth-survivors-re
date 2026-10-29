"""Join the authored roster to pinned equipment evidence, without guessing combat rules.

python3 docs/audits/enemy-equipment.py target/survivors-validation > report.csv
"""

import csv
import hashlib
import importlib.util
import io
import pathlib
import re
import sys


HASHES = {
    "creature": "447f007c603f4e80fb6732ced568170414b25f6aa62e5eb13a16d9f05281a2cc",
    "creature-equip": "741f07e7740e6c86e6cba5022e370ee4f62b9d492db92128ed03bd0a4341fcb1",
    "creature_item_template": "c060e86253865f999844a047e099d4af10f8ab0b6e375a3e4d60b565e0619ff1",
    "creature_equip_template_raw": "00dd0553f5b2e45553d7813268e0eeca484c0b93bf555556965c224f5a8646fa",
}


def read_table(directory, name):
    filename = f"{name}.sql" if "_" in name else f"vanilla-{name}-template.sql"
    raw = (directory / filename).read_bytes()
    if hashlib.sha256(raw).hexdigest() != HASHES[name]:
        raise ValueError(f"{name}: input differs from pinned catalog")
    lines = raw.decode().splitlines()
    header = next(line for line in lines if line.startswith("INSERT INTO"))
    columns = re.findall(r"`([^`]+)`", header)[1:]
    rows = {}
    for line in lines:
        if not line.startswith("("):
            continue
        values = next(csv.reader([line[1:].rstrip(",;")[:-1]],
                                 quotechar="'", escapechar="\\"))
        if len(values) != len(columns) or int(values[0]) in rows:
            raise ValueError(f"{name}: unexpected schema or duplicate entry")
        rows[int(values[0])] = dict(zip(columns, values))
    if not rows:
        raise ValueError(f"{name}: empty catalog")
    return rows


def audit(directory, output):
    # Reuse the identity audit so roster syntax drift cannot silently omit enemies.
    spec = importlib.util.spec_from_file_location(
        "creature_types", pathlib.Path(__file__).with_name("creature-types.py"))
    identities = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(identities)
    source = pathlib.Path("crates/survivors-core/src/data.rs").read_text()
    identities.audit((directory / "vanilla-creature-template.sql").read_bytes(), source)
    creatures, equipment, items, legacy = [read_table(directory, name) for name in HASHES]
    roster = sorted(set(re.findall(
        r'\b(?:en|foe|giant)\(\s*"([^"]+)",\s*(\d+)[^)]*\)'
        r'\s*\.typed\((\d+), CreatureType::(\w+)\)', source)))
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(["enemy", "display", "template", "creature_type", "equipment",
                     "slot", "equipment_entry", "source", "item_display", "item_class",
                     "item_subclass", "material", "inventory_type", "sheath",
                     "mechanic_immune_mask"])
    for name, display, entry, kind in roster:
        creature = creatures[int(entry)]
        equipment_id = int(creature["EquipmentTemplateId"])
        for slot in range(1, 4):
            item_id, status, values = resolve(equipment_id, slot, equipment, items, legacy)
            writer.writerow([name, display, entry, kind, equipment_id, slot, item_id,
                             status, *values, creature["MechanicImmuneMask"]])


def resolve(equipment_id, slot, equipment, items, legacy):
    """Follow Creature::LoadEquipment/SetVirtualItem; never use player item rows."""
    if equipment_id == 0:
        return 0, "no equipment template", [0] * 6
    if equipment_id in equipment:
        item_id = int(equipment[equipment_id][f"equipentry{slot}"])
        if not item_id:
            return 0, "empty slot", [0] * 6
        if item_id not in items:
            raise ValueError(f"Missing NPC item {item_id} in equipment {equipment_id}")
        item = items[item_id]
        return item_id, "NPC item", [int(item[key]) for key in (
            "displayid", "class", "subclass", "material", "inventory_type", "sheath_type")]
    if equipment_id in legacy:
        raw = legacy[equipment_id]
        display = int(raw[f"equipmodel{slot}"])
        info = int(raw[f"equipinfo{slot}"])
        sheath = int(raw[f"equipslot{slot}"]) & 255
        return 0, "legacy raw", [display, *[(info >> shift) & 255
                                           for shift in (0, 8, 16, 24)], sheath]
    raise ValueError(f"Missing equipment template {equipment_id}")


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[2] == "--visuals":
        report = io.StringIO()
        audit(pathlib.Path(sys.argv[1]), report)
        rows = csv.DictReader(io.StringIO(report.getvalue()))
        visuals = sorted({(int(row["item_display"]), int(row["inventory_type"]))
                          for row in rows if int(row["item_display"]) != 0})
        for display, inventory in visuals:
            print(display, inventory)
    else:
        audit(pathlib.Path(sys.argv[1]), sys.stdout)
