// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer Abdullah and Concat contributors

//! The motion a text preset brings with it.
//!
//! A text preset has always been a *look*: face, colour, outline. This is the
//! other half - how the title arrives, what it does while it is there, how it
//! leaves - written as the same keys the inspector's diamonds write, so a
//! preset's animation is not a second mechanism: the keys land on the clip,
//! the curve editor shows them, and they can be dragged, re-eased or deleted
//! like any the user put there.
//!
//! In a preset file each key is one `[[animation]]` table:
//!
//! ```toml
//! [[animation]]
//! property = "offsetY"          # scale, offsetX, offsetY, rotation, opacity
//! atSeconds = 0.0               # from the clip's start ...
//! value = 0.08                  # offsets are *from where the preset rests*
//! ease = [0.215, 0.61, 0.355, 1.0]
//!
//! [[animation]]
//! property = "opacity"
//! fromEndSeconds = 0.25         # ... or counted back from its end ...
//! value = 1.0
//!
//! [[animation]]
//! property = "scale"
//! at = 0.5                      # ... or a fraction of the clip
//! value = 1.0
//! ```
//!
//! Seconds rather than fractions are what a motion designer reasons in - "the
//! bounce takes ten frames" is true on a two-second title and on a ten-second
//! one - and a key's own `at` is a fraction of the clip (`model::ClipKey`), so
//! [`resolve`] is where one becomes the other for the clip actually laid.

use serde::Deserialize;

use crate::model::{KeyEase, KeyProperty};

/// One key of a preset's motion, as written in its file.
///
/// Exactly one of `at`, `at_seconds` and `from_end_seconds` places it; with
/// more than one, seconds from the start win, then seconds from the end, then
/// the fraction. A key that names none is not a key and is dropped.
#[derive(Clone, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresetKey {
    /// Which property rides. Only the placement five are accepted - the
    /// clip's sound and mask are not a title's to be keyed by its look.
    pub property: KeyProperty,
    /// Where in the clip, a fraction `0..=1`.
    #[serde(default)]
    pub at: Option<f64>,
    /// Seconds after the clip's start.
    #[serde(default)]
    pub at_seconds: Option<f64>,
    /// Seconds before the clip's end.
    #[serde(default)]
    pub from_end_seconds: Option<f64>,
    /// The value there. For `scale`, `rotation` and `opacity` it is the
    /// property's own number; for `offsetX` and `offsetY` it is a distance
    /// *from where the preset rests*, so a slide that ends at zero ends on
    /// the title's place whatever place the preset gave it.
    pub value: f64,
    /// How the key is approached from the one before it.
    #[serde(default)]
    pub ease: KeyEase,
}

/// A key ready to be set on a clip: what `Command::SetClipKey` takes.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ResolvedKey {
    /// Which property.
    pub property: KeyProperty,
    /// A fraction of the clip, `0..=1`.
    pub at: f64,
    /// The property's own value.
    pub value: f64,
    /// How it is approached.
    pub ease: KeyEase,
}

/// How much of a clip its seconds-placed keys may take between them. Past
/// this they are squeezed to fit, so an entrance and an exit written for a
/// two-second title still happen, one after the other, on a half-second one.
const MOST_OF_A_CLIP: f64 = 0.9;

/// Whether a property is one a preset may key.
fn allowed(property: KeyProperty) -> bool {
    matches!(
        property,
        KeyProperty::Scale
            | KeyProperty::OffsetX
            | KeyProperty::OffsetY
            | KeyProperty::Rotation
            | KeyProperty::Opacity
    )
}

/// `keys` as they land on a clip `duration` seconds long, whose preset rests
/// at `rest_offset_y` (a frame-height fraction from the centre; zero when the
/// preset has no opinion).
///
/// Keys with no place, a place or value that is not a number, or a property a
/// preset may not key, are left out rather than failing the preset: the look
/// is still worth laying when the motion is half-written. The result is in
/// the order of the file, which is not necessarily the order of the clip -
/// `SetClipKey` files each key by its `at`.
pub fn resolve(keys: &[PresetKey], duration: f64, rest_offset_y: f64) -> Vec<ResolvedKey> {
    if !duration.is_finite() || duration <= 0.0 {
        return Vec::new();
    }

    // How long the two seconds-placed ends want, to see if they fit.
    let entrance = keys
        .iter()
        .filter_map(|key| key.at_seconds)
        .filter(|seconds| seconds.is_finite())
        .fold(0.0_f64, f64::max);
    let exit = keys
        .iter()
        .filter_map(|key| key.from_end_seconds)
        .filter(|seconds| seconds.is_finite())
        .fold(0.0_f64, f64::max);
    let wanted = entrance + exit;
    let squeeze = if wanted > duration * MOST_OF_A_CLIP {
        duration * MOST_OF_A_CLIP / wanted
    } else {
        1.0
    };

    keys.iter()
        .filter(|key| allowed(key.property) && key.value.is_finite())
        .filter_map(|key| {
            let at = if let Some(seconds) = key.at_seconds {
                seconds * squeeze / duration
            } else if let Some(seconds) = key.from_end_seconds {
                1.0 - seconds * squeeze / duration
            } else {
                key.at?
            };
            if !at.is_finite() {
                return None;
            }
            let value = match key.property {
                KeyProperty::OffsetY => key.value + rest_offset_y,
                _ => key.value,
            };
            Some(ResolvedKey {
                property: key.property,
                at: at.clamp(0.0, 1.0),
                value,
                ease: key.ease,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(property: KeyProperty, value: f64) -> PresetKey {
        PresetKey {
            property,
            at: None,
            at_seconds: None,
            from_end_seconds: None,
            value,
            ease: KeyEase::LINEAR,
        }
    }

    #[test]
    fn seconds_become_fractions_of_the_clip_laid() {
        let keys = [
            PresetKey {
                at_seconds: Some(0.0),
                ..key(KeyProperty::Opacity, 0.0)
            },
            PresetKey {
                at_seconds: Some(0.5),
                ..key(KeyProperty::Opacity, 1.0)
            },
            PresetKey {
                from_end_seconds: Some(1.0),
                ..key(KeyProperty::Opacity, 1.0)
            },
            PresetKey {
                at: Some(1.0),
                ..key(KeyProperty::Opacity, 0.0)
            },
        ];
        let on_four: Vec<f64> = resolve(&keys, 4.0, 0.0).iter().map(|k| k.at).collect();
        assert_eq!(on_four, vec![0.0, 0.125, 0.75, 1.0]);
        // The same ten frames take a bigger share of a shorter title.
        let on_two: Vec<f64> = resolve(&keys, 2.0, 0.0).iter().map(|k| k.at).collect();
        assert_eq!(on_two, vec![0.0, 0.25, 0.5, 1.0]);
    }

    #[test]
    fn an_offset_is_from_where_the_preset_rests() {
        let keys = [
            PresetKey {
                at: Some(0.0),
                ..key(KeyProperty::OffsetY, 0.1)
            },
            PresetKey {
                at: Some(0.5),
                ..key(KeyProperty::OffsetY, 0.0)
            },
            PresetKey {
                at: Some(0.0),
                ..key(KeyProperty::OffsetX, 0.2)
            },
            PresetKey {
                at: Some(0.0),
                ..key(KeyProperty::Scale, 0.5)
            },
        ];
        let out = resolve(&keys, 2.0, 0.3);
        assert!((out[0].value - 0.4).abs() < 1e-12, "slides in from below");
        assert!((out[1].value - 0.3).abs() < 1e-12, "and ends at rest");
        assert_eq!(out[2].value, 0.2, "x rests at the centre");
        assert_eq!(out[3].value, 0.5, "a scale is itself");
    }

    #[test]
    fn an_entrance_and_an_exit_squeeze_to_fit_a_short_clip() {
        let keys = [
            PresetKey {
                at_seconds: Some(1.0),
                ..key(KeyProperty::Opacity, 1.0)
            },
            PresetKey {
                from_end_seconds: Some(1.0),
                ..key(KeyProperty::Opacity, 1.0)
            },
        ];
        // Two seconds of motion in a one-second clip: the exit still comes
        // after the entrance.
        let out = resolve(&keys, 1.0, 0.0);
        assert!(out[0].at < out[1].at, "{:?}", out);
        assert!(out[0].at > 0.0 && out[1].at < 1.0);
    }

    #[test]
    fn what_a_title_may_not_wear_is_left_out_not_fatal() {
        let keys = [
            PresetKey {
                at: Some(0.0),
                ..key(KeyProperty::Volume, 1.0)
            },
            PresetKey {
                at: Some(0.0),
                ..key(KeyProperty::MaskX, 1.0)
            },
            key(KeyProperty::Scale, 1.0), // no place
            PresetKey {
                at: Some(f64::NAN),
                ..key(KeyProperty::Scale, 1.0)
            },
            PresetKey {
                at: Some(0.0),
                ..key(KeyProperty::Scale, f64::INFINITY)
            },
            PresetKey {
                at: Some(0.25),
                ..key(KeyProperty::Rotation, -6.0)
            },
        ];
        let out = resolve(&keys, 2.0, 0.0);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].property, KeyProperty::Rotation);
        assert!(resolve(&keys, 0.0, 0.0).is_empty(), "no clip, no keys");
    }

    #[test]
    fn a_key_reads_from_json_the_way_the_file_spells_it() {
        let key: PresetKey = serde_json::from_str(
            r#"{"property":"offsetY","atSeconds":0.1,"value":0.05,
                "ease":[0.215,0.61,0.355,1.0]}"#,
        )
        .expect("parses");
        assert_eq!(key.property, KeyProperty::OffsetY);
        assert_eq!(key.at_seconds, Some(0.1));
        assert_eq!(key.ease, KeyEase([0.215, 0.61, 0.355, 1.0]));
        // A misspelt field is an error, not a key that silently does nothing.
        assert!(serde_json::from_str::<PresetKey>(
            r#"{"property":"scale","atSecond":0.1,"value":1.0}"#
        )
        .is_err());
    }
}
