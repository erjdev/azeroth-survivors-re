import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("weapon_damage", Path(__file__).with_name("weapon-damage.py"))
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


class WeaponDamageTests(unittest.TestCase):
    def fixture(self):
        row = dict.fromkeys(audit.FIELDS, "0")
        row.update(entry="1", name="King's, Sword", **{"class": "2", "delay": "1900",
                   "dmg_min1": "44", "dmg_max1": "115", "dmg_min2": "16",
                   "dmg_max2": "30", "dmg_type2": "3"})
        return row

    def test_preserves_independent_damage_schools_and_escaped_names(self):
        row = self.fixture()
        header = "INSERT INTO `item_template` (" + ",".join(f"`{f}`" for f in audit.FIELDS) + ") VALUES"
        values = ["'King\\'s, Sword'" if f == "name" else row[f] for f in audit.FIELDS]
        raw = (header + "\n(" + ",".join(values) + ");\n").encode()
        rows = audit.parse(raw, verify=False)
        self.assertEqual(audit.selected(rows, [1]), [row])
        with self.assertRaises(ValueError):
            audit.parse(raw)
        with self.assertRaises(ValueError):
            audit.parse(raw + raw.splitlines()[-1] + b"\n", verify=False)

    def test_rejects_missing_nonweapon_and_invalid_numeric_rows(self):
        with self.assertRaises(ValueError):
            audit.selected({}, [1])
        for key, value in [("class", "4"), ("delay", "0"), ("dmg_min1", "NaN"),
                           ("dmg_max1", "-1"), ("dmg_type2", "7")]:
            row = self.fixture()
            row[key] = value
            with self.assertRaises(ValueError):
                audit.selected({1: row}, [1])


if __name__ == "__main__":
    unittest.main()
