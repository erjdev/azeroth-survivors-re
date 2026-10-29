"""Join the authored roster to pinned equipment evidence, without guessing combat rules.

python3 docs/audits/enemy-equipment.py target/survivors-validation > report.csv
"""

import csv
import hashlib
import importlib.util
import pathlib
import re
import sys


HASHES = {
    "creature": "447f007c603f4e80fb6732ced568170414b25f6aa62e5eb13a16d9f05281a2cc",
    "creature-equip": "741f07e7740e6c86e6cba5022e370ee4f62b9d492db92128ed03bd0a4341fcb1",
    "item": "cbba79ec53c0bc451d75270d3ea3bc1264a6ae49ae60e7005cd6300a374a766b",
}


def read_table(directory, name):
    raw = (directory / f"vanilla-{name}-template.sql").read_bytes()
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
    creatures, equipment, items = [read_table(directory, name) for name in HASHES]
    roster = sorted(set(re.findall(
        r'\b(?:en|foe|giant)\(\s*"([^"]+)",\s*(\d+)[^)]*\)'
        r'\s*\.typed\((\d+), CreatureType::(\w+)\)', source)))
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(["enemy", "display", "template", "creature_type", "equipment",
                     "slot", "equipment_entry", "item_catalog_status", "item_class",
                     "item_subclass", "inventory_type", "mechanic_immune_mask"])
    for name, display, entry, kind in roster:
        creature = creatures[int(entry)]
        equipment_id = int(creature["EquipmentTemplateId"])
        equipped = equipment.get(equipment_id)
        for slot in range(1, 4):
            item_id = int(equipped[f"equipentry{slot}"]) if equipped else 0
            item = items.get(item_id)
            status = ("no equipment template" if equipment_id == 0 else
                      "missing equipment template" if equipped is None else
                      "empty slot" if item_id == 0 else
                      "item catalog match" if item else "missing item entry")
            writer.writerow([name, display, entry, kind, equipment_id, slot, item_id,
                             status, item["class"] if item else "",
                             item["subclass"] if item else "",
                             item["InventoryType"] if item else "",
                             creature["MechanicImmuneMask"]])


if __name__ == "__main__":
    audit(pathlib.Path(sys.argv[1]), sys.stdout)
