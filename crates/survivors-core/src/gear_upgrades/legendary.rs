//! Legendary-band armor identities; vanilla powers and set bonuses are not imported.
use super::Item;

pub(super) const ARMOR: [[Item; 8]; 9] = [
    // Vanilla item set 523.
    [
        ("Dreadnaught Helmet", 22418, 35447, 1, 4, 4, 1, 0),
        ("Dreadnaught Pauldrons", 22419, 35177, 3, 4, 4, 1, 0),
        ("Dreadnaught Breastplate", 22416, 35049, 5, 4, 4, 1, 0),
        ("Dreadnaught Waistguard", 22422, 35058, 6, 4, 4, 1, 0),
        ("Dreadnaught Legplates", 22417, 35051, 7, 4, 4, 1, 0),
        ("Dreadnaught Sabatons", 22420, 35067, 8, 4, 4, 1, 0),
        ("Dreadnaught Bracers", 22423, 35044, 9, 4, 4, 1, 0),
        ("Dreadnaught Gauntlets", 22421, 35050, 10, 4, 4, 1, 0),
    ],
    // Vanilla item set 528.
    [
        ("Redemption Headpiece", 22428, 35612, 1, 4, 4, 1, 0),
        ("Redemption Spaulders", 22429, 35617, 3, 4, 4, 1, 0),
        ("Redemption Tunic", 22425, 35618, 5, 4, 4, 1, 0),
        ("Redemption Girdle", 22431, 35614, 6, 4, 4, 1, 0),
        ("Redemption Legguards", 22427, 35616, 7, 4, 4, 1, 0),
        ("Redemption Boots", 22430, 35613, 8, 4, 4, 1, 0),
        ("Redemption Wristguards", 22424, 35619, 9, 4, 4, 1, 0),
        ("Redemption Handguards", 22426, 35615, 10, 4, 4, 1, 0),
    ],
    // Vanilla item set 530.
    [
        ("Cryptstalker Headpiece", 22438, 35601, 1, 4, 3, 5, 0),
        ("Cryptstalker Spaulders", 22439, 35611, 3, 4, 3, 5, 0),
        ("Cryptstalker Tunic", 22436, 35415, 5, 4, 3, 5, 0),
        ("Cryptstalker Girdle", 22442, 35410, 6, 4, 3, 5, 0),
        ("Cryptstalker Legguards", 22437, 35413, 7, 4, 3, 5, 0),
        ("Cryptstalker Boots", 22440, 35409, 8, 4, 3, 5, 0),
        ("Cryptstalker Wristguards", 22443, 35416, 9, 4, 3, 5, 0),
        ("Cryptstalker Handguards", 22441, 35411, 10, 4, 3, 5, 0),
    ],
    // Vanilla item set 524.
    [
        ("Bonescythe Helmet", 22478, 35132, 1, 4, 2, 8, 0),
        ("Bonescythe Pauldrons", 22479, 35064, 3, 4, 2, 8, 0),
        ("Bonescythe Breastplate", 22476, 35054, 5, 4, 2, 8, 0),
        ("Bonescythe Waistguard", 22482, 36349, 6, 4, 2, 8, 0),
        ("Bonescythe Legplates", 22477, 35065, 7, 4, 2, 8, 0),
        ("Bonescythe Sabatons", 22480, 36351, 8, 4, 2, 8, 0),
        ("Bonescythe Bracers", 22483, 35053, 9, 4, 2, 8, 0),
        ("Bonescythe Gauntlets", 22481, 35055, 10, 4, 2, 8, 0),
    ],
    // Vanilla item set 525.
    [
        ("Circlet of Faith", 22514, 35155, 1, 4, 1, 7, 0),
        ("Shoulderpads of Faith", 22515, 35149, 3, 4, 1, 7, 0),
        ("Robe of Faith", 22512, 36354, 20, 4, 1, 7, 0),
        ("Belt of Faith", 22518, 35143, 6, 4, 1, 7, 0),
        ("Leggings of Faith", 22513, 35154, 7, 4, 1, 7, 0),
        ("Sandals of Faith", 22516, 35148, 8, 4, 1, 7, 0),
        ("Bindings of Faith", 22519, 35144, 9, 4, 1, 7, 0),
        ("Gloves of Faith", 22517, 35145, 10, 4, 1, 7, 0),
    ],
    // Vanilla item set 527.
    [
        ("Earthshatter Headpiece", 22466, 35749, 1, 4, 3, 5, 0),
        ("Earthshatter Spaulders", 22467, 35751, 3, 4, 3, 5, 0),
        ("Earthshatter Tunic", 22464, 35752, 5, 4, 3, 5, 0),
        ("Earthshatter Girdle", 22470, 35747, 6, 4, 3, 5, 0),
        ("Earthshatter Legguards", 22465, 35754, 7, 4, 3, 5, 0),
        ("Earthshatter Boots", 22468, 35746, 8, 4, 3, 5, 0),
        ("Earthshatter Wristguards", 22471, 35753, 9, 4, 3, 5, 0),
        ("Earthshatter Handguards", 22469, 35748, 10, 4, 3, 5, 0),
    ],
    // Vanilla item set 526.
    [
        ("Frostfire Circlet", 22498, 36440, 1, 4, 1, 7, 0),
        ("Frostfire Shoulderpads", 22499, 35326, 3, 4, 1, 7, 0),
        ("Frostfire Robe", 22496, 35523, 20, 4, 1, 7, 0),
        ("Frostfire Belt", 22502, 35519, 6, 4, 1, 7, 0),
        ("Frostfire Leggings", 22497, 35522, 7, 4, 1, 7, 0),
        ("Frostfire Sandals", 22500, 35525, 8, 4, 1, 7, 0),
        ("Frostfire Bindings", 22503, 35677, 9, 4, 1, 7, 0),
        ("Frostfire Gloves", 22501, 35521, 10, 4, 1, 7, 0),
    ],
    // Vanilla item set 529.
    [
        ("Plagueheart Circlet", 22506, 35182, 1, 4, 1, 7, 0),
        ("Plagueheart Shoulderpads", 22507, 35187, 3, 4, 1, 7, 0),
        ("Plagueheart Robe", 22504, 35185, 20, 4, 1, 7, 0),
        ("Plagueheart Belt", 22510, 35179, 6, 4, 1, 7, 0),
        ("Plagueheart Leggings", 22505, 35184, 7, 4, 1, 7, 0),
        ("Plagueheart Sandals", 22508, 35186, 8, 4, 1, 7, 0),
        ("Plagueheart Bindings", 22511, 35180, 9, 4, 1, 7, 0),
        ("Plagueheart Gloves", 22509, 35183, 10, 4, 1, 7, 0),
    ],
    // Vanilla item set 521.
    [
        ("Dreamwalker Headpiece", 22490, 35162, 1, 4, 2, 8, 0),
        ("Dreamwalker Spaulders", 22491, 35160, 3, 4, 2, 8, 0),
        ("Dreamwalker Tunic", 22488, 35159, 5, 4, 2, 8, 0),
        ("Dreamwalker Girdle", 22494, 35164, 6, 4, 2, 8, 0),
        ("Dreamwalker Legguards", 22489, 35161, 7, 4, 2, 8, 0),
        ("Dreamwalker Boots", 22492, 35173, 8, 4, 2, 8, 0),
        ("Dreamwalker Wristguards", 22495, 35158, 9, 4, 2, 8, 0),
        ("Dreamwalker Handguards", 22493, 35167, 10, 4, 2, 8, 0),
    ],
];
