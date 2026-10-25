//! Rare-band armor identities; pinned source and authored tier choice are documented in
//! docs/SURVIVORS-GEAR-VISUALS.md. No vanilla powers or loot rules are imported.
use super::Item;

pub(super) const ARMOR: [[Item; 8]; 9] = [
    // Vanilla item set 209.
    [
        ("Helm of Might", 16866, 31260, 1, 4, 4, 6, 0),
        ("Pauldrons of Might", 16868, 31024, 3, 4, 4, 6, 0),
        ("Breastplate of Might", 16865, 31021, 5, 4, 4, 6, 0),
        ("Belt of Might", 16864, 31019, 6, 4, 4, 6, 0),
        ("Legplates of Might", 16867, 31023, 7, 4, 4, 6, 0),
        ("Sabatons of Might", 16862, 31025, 8, 4, 4, 6, 0),
        ("Bracers of Might", 16861, 31020, 9, 4, 4, 6, 0),
        ("Gauntlets of Might", 16863, 31022, 10, 4, 4, 6, 0),
    ],
    // Vanilla item set 208.
    [
        ("Lawbringer Helm", 16854, 31506, 1, 4, 4, 6, 0),
        ("Lawbringer Spaulders", 16856, 31510, 3, 4, 4, 6, 0),
        ("Lawbringer Chestguard", 16853, 31505, 5, 4, 4, 6, 0),
        ("Lawbringer Belt", 16858, 31353, 6, 4, 4, 6, 0),
        ("Lawbringer Legplates", 16855, 31352, 7, 4, 4, 6, 0),
        ("Lawbringer Boots", 16859, 31354, 8, 4, 4, 6, 0),
        ("Lawbringer Bracers", 16857, 31509, 9, 4, 4, 6, 0),
        ("Lawbringer Gauntlets", 16860, 31507, 10, 4, 4, 6, 0),
    ],
    // Vanilla item set 206.
    [
        ("Giantstalker's Helmet", 16846, 32028, 1, 4, 3, 5, 0),
        ("Giantstalker's Epaulets", 16848, 32030, 3, 4, 3, 5, 0),
        ("Giantstalker's Breastplate", 16845, 32022, 5, 4, 3, 5, 0),
        ("Giantstalker's Belt", 16851, 32019, 6, 4, 3, 5, 0),
        ("Giantstalker's Leggings", 16847, 32029, 7, 4, 3, 5, 0),
        ("Giantstalker's Boots", 16849, 32040, 8, 4, 3, 5, 0),
        ("Giantstalker's Bracers", 16850, 32021, 9, 4, 3, 5, 0),
        ("Giantstalker's Gloves", 16852, 32024, 10, 4, 3, 5, 0),
    ],
    // Vanilla item set 204.
    [
        ("Nightslayer Cover", 16821, 31514, 1, 4, 2, 8, 0),
        ("Nightslayer Shoulder Pads", 16823, 31504, 3, 4, 2, 8, 0),
        ("Nightslayer Chestpiece", 16820, 31105, 5, 4, 2, 8, 0),
        ("Nightslayer Belt", 16827, 31339, 6, 4, 2, 8, 0),
        ("Nightslayer Pants", 16822, 31340, 7, 4, 2, 8, 0),
        ("Nightslayer Boots", 16824, 31109, 8, 4, 2, 8, 0),
        ("Nightslayer Bracelets", 16825, 31106, 9, 4, 2, 8, 0),
        ("Nightslayer Gloves", 16826, 31503, 10, 4, 2, 8, 0),
    ],
    // Vanilla item set 202.
    [
        ("Circlet of Prophecy", 16813, 31371, 1, 4, 1, 7, 0),
        ("Mantle of Prophecy", 16816, 30623, 3, 4, 1, 7, 0),
        ("Robes of Prophecy", 16815, 31490, 20, 4, 1, 7, 0),
        ("Girdle of Prophecy", 16817, 30621, 6, 4, 1, 7, 0),
        ("Pants of Prophecy", 16814, 28198, 7, 4, 1, 7, 0),
        ("Boots of Prophecy", 16811, 31718, 8, 4, 1, 7, 0),
        ("Vambraces of Prophecy", 16819, 30617, 9, 4, 1, 7, 0),
        ("Gloves of Prophecy", 16812, 30620, 10, 4, 1, 7, 0),
    ],
    // Vanilla item set 207.
    [
        ("Earthfury Helmet", 16842, 31835, 1, 4, 3, 5, 0),
        ("Earthfury Epaulets", 16844, 31833, 3, 4, 3, 5, 0),
        ("Earthfury Vestments", 16841, 31832, 20, 4, 3, 5, 0),
        ("Earthfury Belt", 16838, 31829, 6, 4, 3, 5, 0),
        ("Earthfury Legguards", 16843, 31836, 7, 4, 3, 5, 0),
        ("Earthfury Boots", 16837, 31830, 8, 4, 3, 5, 0),
        ("Earthfury Bracers", 16840, 31831, 9, 4, 3, 5, 0),
        ("Earthfury Gauntlets", 16839, 31834, 10, 4, 3, 5, 0),
    ],
    // Vanilla item set 201.
    [
        ("Arcanist Crown", 16795, 31517, 1, 4, 1, 7, 0),
        ("Arcanist Mantle", 16797, 30586, 3, 4, 1, 7, 0),
        ("Arcanist Robes", 16798, 30581, 20, 4, 1, 7, 0),
        ("Arcanist Belt", 16802, 30583, 6, 4, 1, 7, 0),
        ("Arcanist Leggings", 16796, 30582, 7, 4, 1, 7, 0),
        ("Arcanist Boots", 16800, 30587, 8, 4, 1, 7, 0),
        ("Arcanist Bindings", 16799, 30584, 9, 4, 1, 7, 0),
        ("Arcanist Gloves", 16801, 30585, 10, 4, 1, 7, 0),
    ],
    // Vanilla item set 203.
    [
        ("Felheart Horns", 16808, 31987, 1, 4, 1, 7, 0),
        ("Felheart Shoulder Pads", 16807, 31974, 3, 4, 1, 7, 0),
        ("Felheart Robes", 16809, 31973, 20, 4, 1, 7, 0),
        ("Felheart Belt", 16806, 31969, 6, 4, 1, 7, 0),
        ("Felheart Pants", 16810, 31972, 7, 4, 1, 7, 0),
        ("Felheart Slippers", 16803, 31975, 8, 4, 1, 7, 0),
        ("Felheart Bracers", 16804, 31970, 9, 4, 1, 7, 0),
        ("Felheart Gloves", 16805, 31971, 10, 4, 1, 7, 0),
    ],
    // Vanilla item set 205.
    [
        ("Cenarion Helm", 16834, 32790, 1, 4, 2, 8, 0),
        ("Cenarion Spaulders", 16836, 32016, 3, 4, 2, 8, 0),
        ("Cenarion Vestments", 16833, 31797, 20, 4, 2, 8, 0),
        ("Cenarion Belt", 16828, 31722, 6, 4, 2, 8, 0),
        ("Cenarion Leggings", 16835, 31729, 7, 4, 2, 8, 0),
        ("Cenarion Boots", 16829, 31724, 8, 4, 2, 8, 0),
        ("Cenarion Bracers", 16830, 31725, 9, 4, 2, 8, 0),
        ("Cenarion Gloves", 16831, 31726, 10, 4, 2, 8, 0),
    ],
];
