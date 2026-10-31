"""Equipment lookup regressions: python3 -m unittest discover -s docs/audits -p 'test_enemy_equipment.py'."""
import importlib.util
import pathlib
import unittest

spec = importlib.util.spec_from_file_location(
    "equipment", pathlib.Path(__file__).with_name("enemy-equipment.py"))
equipment = importlib.util.module_from_spec(spec)
spec.loader.exec_module(equipment)


class EquipmentResolution(unittest.TestCase):
    def test_npc_items_take_precedence_over_raw_fallback(self):
        items = {1908: dict(displayid="123", **{"class": "2"}, subclass="10",
                            material="2", inventory_type="17", sheath_type="2")}
        result = equipment.resolve(1, 1, {1: {"equipentry1": "1908"}}, items,
                                   {1: {"equipmodel1": "999"}})
        self.assertEqual(result, (1908, "NPC item", [123, 2, 10, 2, 17, 2]))

    def test_empty_modern_slot_does_not_borrow_legacy_weapon(self):
        self.assertEqual(equipment.resolve(1, 1, {1: {"equipentry1": "0"}}, {},
                                           {1: {"equipmodel1": "999"}}),
                         (0, "empty slot", [0] * 6))

    def test_raw_bytes_keep_material_inventory_and_sheath_separate(self):
        raw = {1: {"equipmodel2": "123", "equipinfo2": str(0x11FF0A02),
                   "equipslot2": str(0x1202)}}
        self.assertEqual(equipment.resolve(1, 2, {}, {}, raw),
                         (0, "legacy raw", [123, 2, 10, 255, 17, 2]))

    def test_missing_npc_item_is_not_an_empty_slot_or_raw_fallback(self):
        with self.assertRaisesRegex(ValueError, "Missing NPC item 1908"):
            equipment.resolve(1, 1, {1: {"equipentry1": "1908"}}, {}, {1: {}})

    def test_missing_equipment_is_not_unarmed(self):
        with self.assertRaisesRegex(ValueError, "Missing equipment template 9"):
            equipment.resolve(9, 1, {}, {}, {})

    def test_zero_template_ignores_catalog_rows(self):
        self.assertEqual(equipment.resolve(0, 1, {0: {}}, {}, {0: {}}),
                         (0, "no equipment template", [0] * 6))
