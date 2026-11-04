"""Read requested weapon damage rows from the pinned item-template catalog.

This reports source facts, not Survivors tuning or a runtime weapon formula.
"""
import argparse
import csv
from decimal import Decimal
import hashlib
from pathlib import Path
import re
import sys

SHA256 = "cbba79ec53c0bc451d75270d3ea3bc1264a6ae49ae60e7005cd6300a374a766b"
FIELDS = ["entry", "name", "class", "subclass", "InventoryType", "Quality", "delay"]
FIELDS += [f"dmg_{field}{slot}" for slot in range(1, 6) for field in ("min", "max", "type")]


def parse(raw, verify=True):
    if verify and hashlib.sha256(raw).hexdigest() != SHA256:
        raise ValueError("item catalog differs from pinned source")
    lines = raw.decode("utf-8").splitlines()
    header = next((line for line in lines if line.startswith("INSERT INTO `item_template`")), None)
    if header is None:
        raise ValueError("missing item_template header")
    columns = re.findall(r"`([^`]+)`", header)[1:]
    if len(set(columns)) != len(columns) or not set(FIELDS).issubset(columns):
        raise ValueError("missing or duplicate source columns")
    rows = {}
    for line in lines:
        if not line.startswith("("):
            continue
        values = next(csv.reader([line[1:].rstrip(",;")[:-1]],
                                 quotechar="'", escapechar="\\"))
        if len(values) != len(columns):
            raise ValueError("item row width differs from header")
        row = dict(zip(columns, values))
        entry = int(row["entry"])
        if entry in rows:
            raise ValueError("duplicate item entry")
        rows[entry] = row
    if not rows:
        raise ValueError("empty item catalog")
    return rows


def selected(rows, entries):
    result = []
    for entry in entries:
        if entry not in rows:
            raise ValueError(f"missing item {entry}")
        row = rows[entry]
        if row["class"] != "2" or int(row["delay"]) <= 0:
            raise ValueError(f"item {entry} is not a weapon with positive delay")
        for slot in range(1, 6):
            lo, hi = (Decimal(row[f"dmg_{key}{slot}"]) for key in ("min", "max"))
            if not lo.is_finite() or not hi.is_finite() or not 0 <= lo <= hi:
                raise ValueError(f"item {entry} has invalid damage range")
            if not 0 <= int(row[f"dmg_type{slot}"]) <= 6:
                raise ValueError(f"item {entry} has invalid school")
        result.append({field: row[field] for field in FIELDS})
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("catalog", type=Path)
    parser.add_argument("entries", nargs="+", type=int)
    args = parser.parse_args()
    # Validate every requested row before emitting any output.
    rows = selected(parse(args.catalog.read_bytes()), args.entries)
    writer = csv.DictWriter(sys.stdout, FIELDS, lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
