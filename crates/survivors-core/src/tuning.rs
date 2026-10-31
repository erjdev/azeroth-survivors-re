//! Small validated development tuning surface; packaged defaults remain embedded in the executable.

pub const EMBEDDED: &str = include_str!("../tuning.txt");

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tuning {
    pub damage: f32,
    pub movement: f32,
    pub experience: f32,
    pub pickup: f32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self::parse(EMBEDDED).expect("embedded Survivors tuning must be valid")
    }
}

impl Tuning {
    /// Accept the complete four-key file atomically; a half-written edit is not a new configuration.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut values = [None; 4];
        for (line, raw) in text.lines().enumerate() {
            let raw = raw.split('#').next().unwrap_or("").trim();
            if raw.is_empty() {
                continue;
            }
            let (key, value) = raw
                .split_once('=')
                .ok_or_else(|| format!("Line {} needs key = value", line + 1))?;
            let index = match key.trim() {
                "damage" => 0,
                "movement" => 1,
                "experience" => 2,
                "pickup" => 3,
                other => return Err(format!("Unknown tuning key {other}")),
            };
            if values[index].is_some() {
                return Err(format!("Duplicate tuning key {}", key.trim()));
            }
            let value: f32 = value
                .trim()
                .parse()
                .map_err(|_| format!("Invalid number for {}", key.trim()))?;
            if !value.is_finite() || !(0.1..=10.0).contains(&value) {
                return Err(format!("{} must be between 0.1 and 10", key.trim()));
            }
            values[index] = Some(value);
        }
        let [Some(damage), Some(movement), Some(experience), Some(pickup)] = values else {
            return Err("All four tuning keys are required".into());
        };
        Ok(Self {
            damage,
            movement,
            experience,
            pickup,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_edits_load_and_partial_or_invalid_edits_are_rejected() {
        let text = "damage=2\nmovement=1.2\nexperience=3\npickup=2 # test\n";
        let tuning = Tuning::parse(text).unwrap();
        assert_eq!(tuning.damage, 2.0);
        assert_eq!(tuning.pickup, 2.0);
        for bad in [
            "damage=2",
            "damage=NaN\nmovement=1\nexperience=1\npickup=1",
            "damage=0\nmovement=1\nexperience=1\npickup=1",
        ] {
            assert!(Tuning::parse(bad).is_err());
        }
        assert!(Tuning::parse(&(text.to_owned() + "damage=1\n")).is_err());
        assert!(Tuning::parse(&(text.to_owned() + "typo=1\n")).is_err());
        assert!(Tuning::parse(EMBEDDED).is_ok());
    }
}
