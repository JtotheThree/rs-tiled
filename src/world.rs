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
    /// The [`WorldMap`]s defined in the world file.
    pub maps: Option<Vec<WorldMap>>,
    /// Optional regex pattern to load maps.
    pub patterns: Option<Vec<WorldPattern>>,
    /// The type of world, which is arbitrary and set by the user.
    #[serde(rename = "type")]
    pub world_type: Option<String>,
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
    pub width: Option<u32>,
    /// The optional height of the map.
    pub height: Option<u32>,
}

/// A WorldPattern defines a regex pattern to automatically determine which maps to load and how to lay them out.
#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct WorldPattern {
    /// The regex pattern to match against filenames. The first two capture groups should be the x integer and y integer positions.
    pub regexp: String,
    /// The multiplier for the x position.
    #[serde(rename = "multiplierX")]
    pub multiplier_x: u32,
    /// The multiplier for the y position.
    #[serde(rename = "multiplierY")]
    pub multiplier_y: u32,
    /// The offset for the x position.
    #[serde(rename = "offsetX")]
    pub offset_x: i32,
    /// The offset for the y position.
    #[serde(rename = "offsetY")]
    pub offset_y: i32,
}

impl WorldPattern {
    /// Utility function to test a single path against the defined regexp field and returns a parsed WorldMap if it matches.
    /// Returns none if the filename does not match the pattern.
    pub fn capture_path(&self, path: &Path) -> Result<WorldMap, Error> {
        let re = Regex::new(&self.regexp).unwrap();
        let captures = re
            .captures(path.to_str().unwrap())
            .ok_or(Error::CapturesNotFound)?;

        let x = captures
            .get(1)
            .ok_or(Error::CapturesNotFound)?
            .as_str()
            .parse::<i32>()
            .unwrap();
        let y = captures
            .get(2)
            .ok_or(Error::CapturesNotFound)?
            .as_str()
            .parse::<i32>()
            .unwrap();

        // Calculate x and y positions based on the multiplier and offset.
        let x = x
            .checked_mul(self.multiplier_x as i32)
            .ok_or(Error::InvalidPropertyValue {
                description: "multiplierX causes overflow".to_string(),
            })?
            .checked_add(self.offset_x)
            .ok_or(Error::InvalidPropertyValue {
                description: "offsetX causes overflow".to_string(),
            })?;

        let y = y
            .checked_mul(self.multiplier_y as i32)
            .ok_or(Error::InvalidPropertyValue {
                description: "multiplierY causes overflow".to_string(),
            })?
            .checked_add(self.offset_y)
            .ok_or(Error::InvalidPropertyValue {
                description: "offsetY causes overflow".to_string(),
            })?;

        Ok(WorldMap {
            filename: path.to_str().unwrap().to_owned(),
            x,
            y,
            width: None,
            height: None,
        })
    }

    /// Utility function to test a list of paths against the defined regexp field.
    /// Returns a parsed list of WorldMaps from any matched filenames.
    pub fn capture_paths(&self, paths: Vec<PathBuf>) -> Result<Vec<WorldMap>, Error> {
        paths
            .iter()
            .map(|path| self.capture_path(path.as_path()))
            .collect::<Result<Vec<_>, _>>()
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

    let world: World = match serde_json::from_str(&world_string) {
        Ok(world) => world,
        Err(err) => {
            return Err(Error::JsonDecodingError(err));
        }
    };

    Ok(world)
}
