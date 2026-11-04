//! Epic identities from pinned vanilla templates; Survivors owns stats and acquisition.
use super::Item;

pub(super) const ARMOR: [[Item; 8]; 9] = [
    // Vanilla item set 218.
    [
        ("Helm of Wrath", 16963, 34215, 1, 4, 4, 6, 0),
        ("Pauldrons of Wrath", 16961, 34253, 3, 4, 4, 6, 0),
        ("Breastplate of Wrath", 16966, 33983, 5, 4, 4, 6, 0),
        ("Waistband of Wrath", 16960, 33990, 6, 4, 4, 6, 0),
        ("Legplates of Wrath", 16962, 33986, 7, 4, 4, 6, 0),
        ("Sabatons of Wrath", 16965, 33989, 8, 4, 4, 6, 0),
        ("Bracelets of Wrath", 16959, 33982, 9, 4, 4, 6, 0),
        ("Gauntlets of Wrath", 16964, 33984, 10, 4, 4, 6, 0),
    ],
    // Vanilla item set 217.
    [
        ("Judgement Crown", 16955, 34216, 1, 4, 4, 6, 0),
        ("Judgement Spaulders", 16953, 34258, 3, 4, 4, 6, 0),
        ("Judgement Breastplate", 16958, 33635, 5, 4, 4, 6, 0),
        ("Judgement Belt", 16952, 33633, 6, 4, 4, 6, 0),
        ("Judgement Legplates", 16954, 33637, 7, 4, 4, 6, 0),
        ("Judgement Sabatons", 16957, 33639, 8, 4, 4, 6, 0),
        ("Judgement Bindings", 16951, 33634, 9, 4, 4, 6, 0),
        ("Judgement Gauntlets", 16956, 33636, 10, 4, 4, 6, 0),
    ],
    // Vanilla item set 215.
    [
        ("Dragonstalker's Helm", 16939, 34367, 1, 4, 3, 5, 0),
        ("Dragonstalker's Spaulders", 16937, 34091, 3, 4, 3, 5, 0),
        ("Dragonstalker's Breastplate", 16942, 33667, 5, 4, 3, 5, 0),
        ("Dragonstalker's Belt", 16936, 33665, 6, 4, 3, 5, 0),
        ("Dragonstalker's Legguards", 16938, 33672, 7, 4, 3, 5, 0),
        ("Dragonstalker's Greaves", 16941, 34269, 8, 4, 3, 5, 0),
        ("Dragonstalker's Bracers", 16935, 33666, 9, 4, 3, 5, 0),
        ("Dragonstalker's Gauntlets", 16940, 33668, 10, 4, 3, 5, 0),
    ],
    // Vanilla item set 213.
    [
        ("Bloodfang Hood", 16908, 33743, 1, 4, 2, 8, 0),
        ("Bloodfang Spaulders", 16832, 33653, 3, 4, 2, 8, 0),
        ("Bloodfang Chestpiece", 16905, 33650, 5, 4, 2, 8, 0),
        ("Bloodfang Belt", 16910, 31110, 6, 4, 2, 8, 0),
        ("Bloodfang Pants", 16909, 31115, 7, 4, 2, 8, 0),
        ("Bloodfang Boots", 16906, 31111, 8, 4, 2, 8, 0),
        ("Bloodfang Bracers", 16911, 31127, 9, 4, 2, 8, 0),
        ("Bloodfang Gloves", 16907, 33651, 10, 4, 2, 8, 0),
    ],
    // Vanilla item set 211.
    [
        ("Halo of Transcendence", 16921, 34233, 1, 4, 1, 7, 0),
        ("Pauldrons of Transcendence", 16924, 34048, 3, 4, 1, 7, 0),
        ("Robes of Transcendence", 16923, 34047, 20, 4, 1, 7, 0),
        ("Belt of Transcendence", 16925, 34053, 6, 4, 1, 7, 0),
        ("Leggings of Transcendence", 16922, 34049, 7, 4, 1, 7, 0),
        ("Boots of Transcendence", 16919, 34055, 8, 4, 1, 7, 0),
        ("Bindings of Transcendence", 16926, 34052, 9, 4, 1, 7, 0),
        ("Handguards of Transcendence", 16920, 34051, 10, 4, 1, 7, 0),
    ],
    // Vanilla item set 216.
    [
        ("Helmet of Ten Storms", 16947, 34217, 1, 4, 3, 5, 0),
        ("Epaulets of Ten Storms", 16945, 34255, 3, 4, 3, 5, 0),
        ("Breastplate of Ten Storms", 16950, 34081, 5, 4, 3, 5, 0),
        ("Belt of Ten Storms", 16944, 34078, 6, 4, 3, 5, 0),
        ("Legplates of Ten Storms", 16946, 34084, 7, 4, 3, 5, 0),
        ("Greaves of Ten Storms", 16949, 34083, 8, 4, 3, 5, 0),
        ("Bracers of Ten Storms", 16943, 34079, 9, 4, 3, 5, 0),
        ("Gauntlets of Ten Storms", 16948, 34082, 10, 4, 3, 5, 0),
    ],
    // Vanilla item set 210.
    [
        ("Netherwind Crown", 16914, 34218, 1, 4, 1, 7, 0),
        ("Netherwind Mantle", 16917, 34254, 3, 4, 1, 7, 0),
        ("Netherwind Robes", 16916, 34038, 20, 4, 1, 7, 0),
        ("Netherwind Belt", 16818, 34046, 6, 4, 1, 7, 0),
        ("Netherwind Pants", 16915, 34039, 7, 4, 1, 7, 0),
        ("Netherwind Boots", 16912, 34044, 8, 4, 1, 7, 0),
        ("Netherwind Bindings", 16918, 34045, 9, 4, 1, 7, 0),
        ("Netherwind Gloves", 16913, 34041, 10, 4, 1, 7, 0),
    ],
    // Vanilla item set 212.
    [
        ("Nemesis Skullcap", 16929, 34369, 1, 4, 1, 7, 0),
        ("Nemesis Spaulders", 16932, 34022, 3, 4, 1, 7, 0),
        ("Nemesis Robes", 16931, 34014, 20, 4, 1, 7, 0),
        ("Nemesis Belt", 16933, 34011, 6, 4, 1, 7, 0),
        ("Nemesis Leggings", 16930, 29857, 7, 4, 1, 7, 0),
        ("Nemesis Boots", 16927, 34015, 8, 4, 1, 7, 0),
        ("Nemesis Bracers", 16934, 34012, 9, 4, 1, 7, 0),
        ("Nemesis Gloves", 16928, 34013, 10, 4, 1, 7, 0),
    ],
    // Vanilla item set 214.
    [
        ("Stormrage Cover", 16900, 33655, 1, 4, 2, 8, 0),
        ("Stormrage Pauldrons", 16902, 30546, 3, 4, 2, 8, 0),
        ("Stormrage Chestguard", 16897, 30536, 5, 4, 2, 8, 0),
        ("Stormrage Belt", 16903, 30541, 6, 4, 2, 8, 0),
        ("Stormrage Legguards", 16901, 30540, 7, 4, 2, 8, 0),
        ("Stormrage Boots", 16898, 30542, 8, 4, 2, 8, 0),
        ("Stormrage Bracers", 16904, 30548, 9, 4, 2, 8, 0),
        ("Stormrage Handguards", 16899, 34016, 10, 4, 2, 8, 0),
    ],
];

pub(super) const WEAPONS: [Item; 9] = [
    (
        "Ashkandi, Greatsword of the Brotherhood",
        19364,
        32000,
        17,
        2,
        8,
        1,
        1,
    ),
    ("The Untamed Blade", 19334, 31999, 17, 2, 8, 1, 1),
    (
        "Ashjre'thul, Crossbow of Smiting",
        19361,
        32763,
        26,
        2,
        18,
        2,
        0,
    ),
    ("Chromatically Tempered Sword", 19352, 31867, 13, 2, 7, 1, 3),
    ("Lok'amir il Romathis", 19360, 31863, 21, 2, 4, 1, 3),
    ("Herald of Woe", 19357, 31878, 17, 2, 5, 1, 1),
    ("Staff of the Shadow Flame", 19356, 31960, 17, 2, 10, 2, 2),
    ("Claw of Chromaggus", 19347, 31865, 13, 2, 15, 1, 3),
    ("Draconic Maul", 19358, 31877, 17, 2, 5, 1, 1),
];
