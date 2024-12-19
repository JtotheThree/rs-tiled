use std::{
    io::Read,
    path::{Path, PathBuf},
};

use regex::Regex;
use serde::Deserialize;

use crate::{Error, ResourceReader};

/// A World is a list of maps files or regex patterns that define a layout of TMX maps.
/// You can use the loader to further load the maps defined by the world.
#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct World {
    /// The path first used in a [`ResourceReader`] to load this world.
    #[serde(skip_deserializing)]
    pub source: PathBuf,
    /// The [`WorldMap`]s defined by the world file.
    pub maps: Option<Vec<WorldMap>>,
    /// Optional regex pattern to load maps.
    pub patterns: Option<Vec<WorldPattern>>,
}

impl World {
    /// Utility function to test a single path against all defined patterns.
    /// Returns a parsed [`WorldMap`] on the first matched pattern or an error if no patterns match.
    pub fn match_path(&self, path: impl AsRef<Path>) -> Result<WorldMap, Error> {
        if let Some(patterns) = &self.patterns {
            for pattern in patterns {
                let captures = match pattern.regexp.captures(path.as_ref().to_str().unwrap()) {
                    Some(captures) => captures,
                    None => continue,
                };

                let x = match captures.get(1) {
                    Some(x) => x.as_str().parse::<i32>().unwrap(),
                    None => continue,
                };

                let y = match captures.get(2) {
                    Some(y) => y.as_str().parse::<i32>().unwrap(),
                    None => continue,
                };

                // Calculate x and y positions based on the multiplier and offset.
                let x = x
                    .checked_mul(pattern.multiplier_x)
                    .ok_or(Error::RangeError(
                        "Capture x * multiplierX causes overflow".to_string(),
                    ))?
                    .checked_add(pattern.offset_x)
                    .ok_or(Error::RangeError(
                        "Capture x * multiplierX + offsetX causes overflow".to_string(),
                    ))?;

                let y = y
                    .checked_mul(pattern.multiplier_y)
                    .ok_or(Error::RangeError(
                        "Capture y * multiplierY causes overflow".to_string(),
                    ))?
                    .checked_add(pattern.offset_y)
                    .ok_or(Error::RangeError(
                        "Capture y * multiplierY + offsetY causes overflow".to_string(),
                    ))?;

                // Returning the first matched pattern aligns with how Tiled handles patterns.
                return Ok(WorldMap {
                    filename: path.as_ref().to_str().unwrap().to_string(),
                    x,
                    y,
                    width: None,
                    height: None,
                });
            }
        }

        Err(Error::NoMatchFound {
            path: path.as_ref().to_owned(),
        })
    }

    /// Utility function to test a vec of filenames against all defined patterns.
    /// Returns a vec of results with the parsed [`WorldMap`]s if it matches the pattern.
    pub fn match_paths<P: AsRef<Path>>(&self, paths: &[P]) -> Vec<Result<WorldMap, Error>> {
        paths
            .into_iter()
            .map(|path| self.match_path(path))
            .collect()
    }
}

/// A WorldMap provides the information for a map in the world and its layout.
#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct WorldMap {
    /// The filename of the tmx map.
    #[serde(rename = "fileName")]
    pub filename: String,
    /// The x position of the map.
    pub x: i32,
    /// The y position of the map.
    pub y: i32,
    /// The optional width of the map.
    pub width: Option<i32>,
    /// The optional height of the map.
    pub height: Option<i32>,
}

/// A WorldPattern defines a regex pattern to automatically determine which maps to load and how to lay them out.
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorldPattern {
    /// The regex pattern to match against filenames.
    /// The first two capture groups should be the x integer and y integer positions.
    #[serde(with = "serde_regex")]
    pub regexp: Regex,
    /// The multiplier for the x position.
    pub multiplier_x: i32,
    /// The multiplier for the y position.
    pub multiplier_y: i32,
    /// The offset for the x position.
    pub offset_x: i32,
    /// The offset for the y position.
    pub offset_y: i32,
}

impl PartialEq for WorldPattern {
    fn eq(&self, other: &Self) -> bool {
        self.multiplier_x == other.multiplier_x
            && self.multiplier_y == other.multiplier_y
            && self.offset_x == other.offset_x
            && self.offset_y == other.offset_y
            && self.regexp.to_string() == other.regexp.to_string()
    }
}

pub(crate) fn parse_world(
    world_path: &Path,
    reader: &mut impl ResourceReader,
) -> Result<World, Error> {
    let mut path = reader
        .read_from(&world_path)
        .map_err(|err| Error::ResourceLoadingError {
            path: world_path.to_owned(),
            err: Box::new(err),
        })?;

    let mut world_string = String::new();
    path.read_to_string(&mut world_string)
        .map_err(|err| Error::ResourceLoadingError {
            path: world_path.to_owned(),
            err: Box::new(err),
        })?;

    let world: World =
        serde_json::from_str(&world_string).map_err(|err| Error::JsonDecodingError(err))?;

    Ok(world)
}
