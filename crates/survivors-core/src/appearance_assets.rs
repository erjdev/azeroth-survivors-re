//! Explicit Survivors appearance repairs; original assets always win.
const VIRTUOUS_MALE: &str =
    "Item\\TextureComponents\\ArmLowerTexture\\Robe_DungeonPriest_A_02Gold_Sleeve_AL_M.blp";
const VIRTUOUS_FEMALE: &str =
    "Item\\TextureComponents\\ArmLowerTexture\\Robe_DungeonPriest_A_02Gold_Sleeve_AL_F.blp";

/// The vanilla patch chain omits the male lower sleeve but keeps its flared cuff
/// geometry. Reuse this robe's existing sleeve rather than painting the cuff skin.
/// Only the final male candidate falls back: a present unisex or male asset wins.
pub fn read<T, E>(
    enabled: bool,
    path: &str,
    mut original: impl FnMut(&str) -> Result<T, E>,
) -> Result<T, E> {
    match original(path) {
        Err(_) if enabled && path.replace('/', "\\").eq_ignore_ascii_case(VIRTUOUS_MALE) => {
            original(VIRTUOUS_FEMALE)
        }
        result => result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn present_original_wins_without_reading_fallback() {
        let mut seen = Vec::new();
        assert_eq!(
            read(true, VIRTUOUS_MALE, |path| {
                seen.push(path.to_owned());
                Ok::<_, ()>(7)
            }),
            Ok(7)
        );
        assert_eq!(seen, [VIRTUOUS_MALE]);
    }

    #[test]
    fn missing_male_uses_only_the_matching_robes_existing_sleeve() {
        let mut seen = Vec::new();
        assert_eq!(
            read(true, VIRTUOUS_MALE, |path| {
                seen.push(path.to_owned());
                if path == VIRTUOUS_FEMALE {
                    Ok(9)
                } else {
                    Err("missing")
                }
            }),
            Ok(9)
        );
        assert_eq!(seen, [VIRTUOUS_MALE, VIRTUOUS_FEMALE]);
        assert_eq!(
            read(true, VIRTUOUS_MALE, |_| Err::<(), _>("missing")),
            Err("missing")
        );
    }

    #[test]
    fn disabled_unisex_female_and_unrelated_paths_never_substitute() {
        for (enabled, path) in [
            (false, VIRTUOUS_MALE),
            (true, VIRTUOUS_FEMALE),
            (true, "other_M.blp"),
            (true, &VIRTUOUS_MALE.replace("_M.blp", "_U.blp")),
        ] {
            let mut calls = 0;
            assert!(read(enabled, path, |_| {
                calls += 1;
                Err::<(), _>(())
            })
            .is_err());
            assert_eq!(calls, 1);
        }
    }
}
