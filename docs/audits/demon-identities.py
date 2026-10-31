"""Resolve summon creature templates against the hash-checked vanilla catalog.

python3 docs/audits/demon-identities.py target/survivors-validation
"""
import csv
import importlib.util
import pathlib
import sys

spec = importlib.util.spec_from_file_location(
    "equipment", pathlib.Path(__file__).with_name("enemy-equipment.py"))
equipment = importlib.util.module_from_spec(spec)
spec.loader.exec_module(equipment)
rows = equipment.read_table(pathlib.Path(sys.argv[1]), "creature")
writer = csv.writer(sys.stdout, lineterminator="\n")
writer.writerow(["summon_spell", "template", "name", "display", "creature_type"])
for spell, entry, name in [(688, 416, "Imp"), (697, 1860, "Voidwalker"),
                          (712, 1863, "Succubus"), (691, 417, "Felhunter")]:
    row = rows[entry]
    if row["Name"] != name or row["CreatureType"] != "3":
        raise ValueError(f"Unexpected demon identity: {entry}")
    displays = [int(row[f"ModelId{i}"]) for i in range(1, 5)]
    if not displays[0] or any(displays[1:]):
        raise ValueError(f"Expected one explicit display: {entry}")
    writer.writerow([spell, entry, name, displays[0], row["CreatureType"]])
