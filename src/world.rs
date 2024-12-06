use std::{
    fs,
    path::{Path, PathBuf},
};

use regex::Regex;
use serde::Deserialize;

use crate::Error;

/// A World is a collection of maps and their layout in the game world.
#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct World {
    /// The path first used in a ['ResourceReader'] to load this world.
    #[serde(skip_deserializing)]
    pub source: PathBuf,
    /// The maps present in this world.
    pub maps: Option<Vec<WorldMap>>,
    /// Optional regex pattern to load maps.
    patterns: Option<Vec<WorldPattern>>,
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
struct WorldPattern {
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

/// Parse a Tiled World file from a path.
/// If a the Patterns field is present, it will attempt to build the maps list based on the regex patterns.
///
/// ## Example
/// ```
/// # use tiled::Loader;
/// #
/// # fn main() {
/// #    let loader = Loader::new();
/// #    let world = loader.load_world("world.world").unwrap();
/// #    
/// #    for map in world.maps.unwrap() {
/// #        println!("Map: {:?}", map);
/// #    }
/// # }
/// ```
pub(crate) fn parse_world(path: &Path) -> Result<World, Error> {
    let world_file = match std::fs::read_to_string(path) {
        Ok(world_file) => world_file,
        Err(err) => {
            return Err(Error::ResourceLoadingError {
                path: path.to_owned(),
                err: Box::new(err),
            })
        }
    };

    let mut world: World = match serde_json::from_str(&world_file) {
        Ok(world) => world,
        Err(err) => {
            return Err(Error::JsonDecodingError(err));
        }
    };

    if world.patterns.is_some() {
        world.maps = match parse_world_pattern(path, &world.clone().patterns.unwrap()) {
            Ok(maps) => Some(maps),
            Err(err) => return Err(err),
        };
    }

    Ok(world)
}

/// If "patterns" key is present, it will attempt to build the maps list based on the regex patterns.
fn parse_world_pattern(path: &Path, patterns: &Vec<WorldPattern>) -> Result<Vec<WorldMap>, Error> {
    let mut maps = Vec::new();

    let parent_dir = path.parent().ok_or(Error::ResourceLoadingError {
        path: path.to_owned(),
        err: Box::new(std::io::Error::from(std::io::ErrorKind::NotFound)),
    })?;

    // There's no documentation on why "patterns" is a JSON array, so we'll just blast them into same maps list.
    for pattern in patterns {
        let files = fs::read_dir(parent_dir).map_err(|err| Error::ResourceLoadingError {
            path: parent_dir.to_owned(),
            err: Box::new(err),
        })?;

        let re = Regex::new(&pattern.regexp).unwrap();
        let files = files
            .filter_map(|entry| entry.ok())
            .filter(|entry| re.is_match(entry.path().file_name().unwrap().to_str().unwrap()))
            .map(|entry| {
                let filename = entry
                    .path()
                    .file_name()
                    .ok_or_else(|| Error::ResourceLoadingError {
                        path: path.to_owned(),
                        err: "Failed to get file name".into(),
                    })?
                    .to_str()
                    .ok_or_else(|| Error::ResourceLoadingError {
                        path: path.to_owned(),
                        err: "Failed to convert file name to string".into(),
                    })?
                    .to_owned();

                let captures = re.captures(&filename).unwrap();

                // let captures =
                //     re.captures(&filename)
                //         .ok_or_else(|| Error::ResourceLoadingError {
                //             path: path.to_owned(),
                //             err: format!("Failed checking regex match on file {}", filename).into(),
                //         })?;

                let x = captures
                    .get(1)
                    .ok_or_else(|| Error::ResourceLoadingError {
                        path: path.to_owned(),
                        err: format!("Failed to parse x pattern from file {}", filename).into(), 
                    })?
                    .as_str()
                    .parse::<i32>()
                    .map_err(|e| Error::ResourceLoadingError {
                        path: path.to_owned(),
                        err: Box::new(e),
                    })?;

                let x = match x
                    .checked_mul(pattern.multiplier_x as i32)
                    .and_then(|x| x.checked_add(pattern.offset_x))
                {
                    Some(x) => x,
                    None => {
                        return Err(Error::ResourceLoadingError {
                            path: path.to_owned(),
                            err: "Arithmetic Overflow on multiplierX and offsetX".into(),
                        })
                    }
                };
                let y = captures
                    .get(2)
                    .ok_or_else(|| Error::ResourceLoadingError {
                        path: path.to_owned(),
                        err: format!("Failed to parse y pattern from file {}", filename).into(),
                    })?
                    .as_str()
                    .parse::<i32>()
                    .map_err(|e| Error::ResourceLoadingError {
                        path: path.to_owned(),
                        err: Box::new(e),
                    })?;
                let y = match y
                    .checked_mul(pattern.multiplier_y as i32)
                    .and_then(|y| y.checked_add(pattern.offset_y))
                {
                    Some(y) => y,
                    None => {
                        return Err(Error::ResourceLoadingError {
                            path: path.to_owned(),
                            err: "Arithmetic Overflow on multiplierY and offsetY".into(),
                        })
                    }
                };
                Ok(WorldMap {
                    filename,
                    x,
                    y,
                    width: Some(pattern.multiplier_x),
                    height: Some(pattern.multiplier_y),
                })
            })
            .collect::<Vec<_>>();

        for file in files {
            maps.push(file?);
        }
    }

    Ok(maps)
}
