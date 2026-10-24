//! Validated UI layout snapshots, embedded for players and editable during development.

pub const EMBEDDED: &str = include_str!("../ui.txt");

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    pub left_width: f32,
    pub right_width: f32,
    pub gap: f32,
    pub padding: f32,
    pub body_size: f32,
    pub title_size: f32,
    pub cell_height: f32,
}

impl Default for Layout {
    fn default() -> Self {
        Self::parse(EMBEDDED).expect("embedded UI layout must be valid")
    }
}

impl Layout {
    pub fn parse(text: &str) -> Result<Self, String> {
        let keys = [
            "left_width",
            "right_width",
            "gap",
            "padding",
            "body_size",
            "title_size",
            "cell_height",
        ];
        let bounds = [
            (15.0, 30.0),
            (15.0, 30.0),
            (0.0, 24.0),
            (2.0, 24.0),
            (10.0, 18.0),
            (14.0, 28.0),
            (24.0, 72.0),
        ];
        let mut values = [None; 7];
        for (line, raw) in text.lines().enumerate() {
            let raw = raw.split('#').next().unwrap_or("").trim();
            if raw.is_empty() {
                continue;
            }
            let (key, value) = raw
                .split_once('=')
                .ok_or_else(|| format!("Line {} needs key = value", line + 1))?;
            let key = key.trim();
            let i = keys
                .iter()
                .position(|name| *name == key)
                .ok_or_else(|| format!("Unknown layout key {key}"))?;
            if values[i].is_some() {
                return Err(format!("Duplicate layout key {key}"));
            }
            let value: f32 = value
                .trim()
                .parse()
                .map_err(|_| format!("Invalid number for {key}"))?;
            let (min, max) = bounds[i];
            if !value.is_finite() || !(min..=max).contains(&value) {
                return Err(format!("{key} must be between {min} and {max}"));
            }
            values[i] = Some(value);
        }
        let [Some(left_width), Some(right_width), Some(gap), Some(padding), Some(body_size), Some(title_size), Some(cell_height)] =
            values
        else {
            return Err("All seven layout keys are required".into());
        };
        Ok(Self {
            left_width,
            right_width,
            gap,
            padding,
            body_size,
            title_size,
            cell_height,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_rejects_incomplete_nonfinite_and_unbounded_edits() {
        assert!(Layout::parse(EMBEDDED).is_ok());
        for text in [
            EMBEDDED.replace("gap = 8", "gap = NaN"),
            EMBEDDED.replace("left_width = 23", "left_width = 90"),
            EMBEDDED.replace("body_size = 12", "body_size = 0"),
            EMBEDDED.replace("padding = 10", ""),
            format!("{EMBEDDED}\ngap=4"),
            format!("{EMBEDDED}\nunknown=4"),
        ] {
            assert!(Layout::parse(&text).is_err(), "{text}");
        }
        let changed = Layout::parse(&EMBEDDED.replace("body_size = 12", "body_size = 14")).unwrap();
        assert_eq!(changed.body_size, 14.0);
    }
}
