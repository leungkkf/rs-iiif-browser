use crate::{iiif::IiifError, rendering::tiled_image::Size};
use bevy::prelude::debug;
use core::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IiifImageInfo {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) sizes: Option<Vec<Size>>,
    pub(crate) tiles: Option<Vec<IiifTileInfo>>,
    pub(crate) profile: Vec<IiifProfileInfo>,
    #[serde(skip_deserializing)]
    pub(crate) expanded_profiles: Vec<IiifProfileDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IiifTileInfo {
    pub(crate) width: u32,
    pub(crate) height: Option<u32>,
    #[serde(rename(deserialize = "scaleFactors"))]
    pub(crate) scale_factors: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum IiifProfileInfo {
    Url(String),
    ProfileDetails(IiifProfileDetails),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct IiifProfileDetails {
    pub(crate) formats: Vec<IiifImageFormat>,
    pub(crate) qualities: Vec<IiifImageQuality>,
    pub(crate) supports: Vec<IiifFeature>,
}

impl Default for IiifProfileDetails {
    fn default() -> Self {
        Self {
            formats: vec![IiifImageFormat::Jpg],
            qualities: vec![IiifImageQuality::Default],
            supports: vec![],
        }
    }
}

impl IiifProfileDetails {
    fn from_url(url: &str) -> core::result::Result<IiifProfileDetails, IiifError> {
        let profile = match url {
            "http://iiif.io/api/image/2/level0.json" => Self {
                formats: vec![IiifImageFormat::Jpg],
                qualities: vec![IiifImageQuality::Default],
                supports: vec![IiifFeature::SizeByWhListed],
            },
            "http://iiif.io/api/image/2/level1.json" => Self {
                formats: vec![IiifImageFormat::Jpg],
                qualities: vec![IiifImageQuality::Default],
                supports: vec![
                    IiifFeature::SizeByWhListed,
                    IiifFeature::BaseUriRedirect,
                    IiifFeature::Cors,
                    IiifFeature::JsonldMediaType,
                    IiifFeature::RegionByPx,
                    IiifFeature::SizeByH,
                    IiifFeature::SizeByPct,
                    IiifFeature::SizeByW,
                ],
            },
            "http://iiif.io/api/image/2/level2.json" => Self {
                formats: vec![IiifImageFormat::Jpg, IiifImageFormat::Png],
                qualities: vec![IiifImageQuality::Default, IiifImageQuality::Bitonal],
                supports: vec![
                    IiifFeature::SizeByWhListed,
                    IiifFeature::BaseUriRedirect,
                    IiifFeature::Cors,
                    IiifFeature::JsonldMediaType,
                    IiifFeature::RegionByPx,
                    IiifFeature::SizeByH,
                    IiifFeature::SizeByPct,
                    IiifFeature::SizeByW,
                    IiifFeature::RegionByPct,
                    IiifFeature::RotationBy90s,
                    IiifFeature::SizeByConfinedWh,
                    IiifFeature::SizeByDistortedWh,
                    IiifFeature::SizeByForcedWh,
                    IiifFeature::SizeByWh,
                ],
            },
            _ => {
                return Err(IiifError::IiifFormatError(format!(
                    "unexpected profile url {}",
                    url
                )));
            }
        };

        Ok(profile)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Hash, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum IiifFeature {
    BaseUriRedirect,
    CanonicalLinkHeader,
    Cors,
    JsonldMediaType,
    Mirroring,
    ProfileLinkHeader,
    RegionByPct,
    RegionByPx,
    RegionSquare,
    RotationArbitrary,
    RotationBy90s,
    SizeByConfinedWh,
    SizeByH,
    SizeByPct,
    SizeByW,
    SizeByWh,
    SizeUpscaling,
    SizeByWhListed, // Deprecated.
    SizeByForcedWh, // Deprecated.
    SizeAboveFull,  // Deprecated.
    SizeByDistortedWh,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum IiifImageQuality {
    Color,
    Gray,
    Bitonal,
    Native,
    Default,
}

impl fmt::Display for IiifImageQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IiifImageQuality::Default => write!(f, "default"),
            IiifImageQuality::Bitonal => write!(f, "bitonal"),
            IiifImageQuality::Color => write!(f, "color"),
            IiifImageQuality::Gray => write!(f, "gray"),
            IiifImageQuality::Native => write!(f, "default"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum IiifImageFormat {
    Jpg,
    Png,
    Tif,
    Gif,
}

impl fmt::Display for IiifImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IiifImageFormat::Jpg => write!(f, "jpg"),
            IiifImageFormat::Png => write!(f, "png"),
            IiifImageFormat::Tif => write!(f, "tif"),
            IiifImageFormat::Gif => write!(f, "gif"),
        }
    }
}

impl IiifImageInfo {
    /// Build from a URL.
    pub(crate) fn from_url(url: &str) -> core::result::Result<Self, IiifError> {
        let info_json = ureq::get(url).call()?.body_mut().read_to_string()?;
        debug!("info {:?}", info_json);

        Self::from_json(&info_json)
    }

    /// Build from a Json string.
    fn from_json(info_json: &str) -> core::result::Result<Self, IiifError> {
        let mut iiif_image_info: IiifImageInfo = serde_json::from_str(info_json)?;
        debug!("iiif_image_info {:?}", iiif_image_info);

        if iiif_image_info.profile.is_empty() {
            return Err(IiifError::IiifMissingInfo("Missing profile".into()));
        }

        let mut expanded_profiles = Vec::new();

        for p in &iiif_image_info.profile {
            let profile = match p {
                IiifProfileInfo::ProfileDetails(profile_details) => (*profile_details).clone(),
                IiifProfileInfo::Url(url) => IiifProfileDetails::from_url(url)?,
            };
            expanded_profiles.push(profile);
        }

        iiif_image_info.expanded_profiles = expanded_profiles;

        Ok(iiif_image_info)
    }

    /// Get tile size.
    pub(crate) fn get_tile_size(&self) -> Size {
        let default_tile_size = Size::new(512, 512);

        if let Some(tiles) = &self.tiles {
            tiles.first().map_or(default_tile_size, |x| {
                Size::new(x.width, x.height.unwrap_or(x.width))
            })
        } else {
            default_tile_size
        }
    }

    /// Get the resolution for the image’s predefined tiles.
    pub(crate) fn get_tile_scaling_sizes(&self) -> Vec<Size> {
        let mut scaling_sizes = Vec::new();

        if let Some(tiles) = &self.tiles
            && let Some(tile) = tiles.first()
        {
            scaling_sizes = tile
                .scale_factors
                .iter()
                .map(|f| Size::new(self.width / f, self.height / f))
                .collect();
        }

        let default_size = Size::new(self.width, self.height);

        if !scaling_sizes
            .iter()
            .any(|x| x.width == default_size.width && x.height == default_size.height)
        {
            scaling_sizes.push(default_size);
        }

        scaling_sizes.sort_by(|a, b| (a.width * a.height).cmp(&(b.width * b.height)));

        scaling_sizes
    }

    /// Get profile details.
    pub(crate) fn get_profile_details(&self) -> &Vec<IiifProfileDetails> {
        &self.expanded_profiles
    }

    /// Get image sizes.
    pub(crate) fn get_image_sizes(&self) -> Vec<Size> {
        let mut image_sizes = Vec::new();

        if let Some(sizes) = &self.sizes {
            image_sizes = sizes.clone();

            if !image_sizes
                .iter()
                .any(|x| x.width == self.width && x.height == self.height)
            {
                image_sizes.push(Size::new(self.width, self.height));
            }
        } else {
            image_sizes.push(Size::new(self.width, self.height));
        }
        image_sizes.sort_by(|a, b| (a.width * a.height).cmp(&(b.width * b.height)));

        image_sizes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_json() {
        let json = r#"{
            "@context" : "http://iiif.io/api/image/2/context.json",
            "@id" : "https://nationalmuseumse.iiifhosting.com/iiif/6b67e82d21f66308380c15509e97bafa5e696618cff1137988ff80c1aa05e4ee",
            "protocol" : "http://iiif.io/api/image",
            "width" : 7045,
            "height" : 5785,
            "sizes" : [
                { "width" : 440, "height" : 361 },
                { "width" : 220, "height" : 180 },
                { "width" : 880, "height" : 723 }
            ],
            "tiles" : [
                { "width" : 256, "height" : 256, "scaleFactors" : [ 1, 2, 4, 8, 16, 32 ] }
            ],
            "profile" : [
                "http://iiif.io/api/image/2/level0.json",
                {
                  "formats" : [ "jpg" ],
                  "qualities" : [ "native","color","gray" ],
                  "supports" : ["regionByPct","regionSquare","sizeByForcedWh","sizeByWh","sizeAboveFull","rotationBy90s","mirroring"]
                }
            ]
        }"#;

        let image_info = IiifImageInfo::from_json(&json).unwrap();

        assert_eq!(image_info.width, 7045);
        assert_eq!(image_info.height, 5785);
        assert_eq!(
            image_info.sizes,
            Some(vec![
                Size::new(440, 361),
                Size::new(220, 180),
                Size::new(880, 723),
            ])
        );

        let tiles = image_info.tiles.unwrap();

        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].width, 256);
        assert_eq!(tiles[0].height, Some(256));
        assert_eq!(tiles[0].scale_factors, vec![1, 2, 4, 8, 16, 32]);

        assert_eq!(image_info.profile.len(), 2);

        for p in &image_info.profile {
            match p {
                IiifProfileInfo::Url(url) => {
                    assert_eq!(url, "http://iiif.io/api/image/2/level0.json");
                }
                IiifProfileInfo::ProfileDetails(detail) => {
                    assert_eq!(detail.formats, vec![IiifImageFormat::Jpg]);
                    assert_eq!(
                        detail.qualities,
                        vec![
                            IiifImageQuality::Native,
                            IiifImageQuality::Color,
                            IiifImageQuality::Gray
                        ]
                    );
                    assert_eq!(
                        detail.supports,
                        vec![
                            IiifFeature::RegionByPct,
                            IiifFeature::RegionSquare,
                            IiifFeature::SizeByForcedWh,
                            IiifFeature::SizeByWh,
                            IiifFeature::SizeAboveFull,
                            IiifFeature::RotationBy90s,
                            IiifFeature::Mirroring
                        ]
                    );
                }
            }
        }

        assert_eq!(
            image_info.expanded_profiles[0].formats,
            vec![IiifImageFormat::Jpg]
        );
        assert_eq!(
            image_info.expanded_profiles[0].qualities,
            vec![IiifImageQuality::Default,]
        );
        assert_eq!(
            image_info.expanded_profiles[0].supports,
            vec![IiifFeature::SizeByWhListed]
        );

        assert_eq!(
            image_info.expanded_profiles[1].formats,
            vec![IiifImageFormat::Jpg]
        );
        assert_eq!(
            image_info.expanded_profiles[1].qualities,
            vec![
                IiifImageQuality::Native,
                IiifImageQuality::Color,
                IiifImageQuality::Gray
            ]
        );
        assert_eq!(
            image_info.expanded_profiles[1].supports,
            vec![
                IiifFeature::RegionByPct,
                IiifFeature::RegionSquare,
                IiifFeature::SizeByForcedWh,
                IiifFeature::SizeByWh,
                IiifFeature::SizeAboveFull,
                IiifFeature::RotationBy90s,
                IiifFeature::Mirroring
            ]
        );
    }

    #[test]
    fn test_get_tile_size() {
        let image_info = IiifImageInfo {
            height: 0,
            width: 0,
            profile: vec![IiifProfileInfo::ProfileDetails(IiifProfileDetails {
                formats: vec![IiifImageFormat::Jpg],
                qualities: Vec::new(),
                supports: Vec::new(),
            })],
            tiles: None,
            sizes: None,
            expanded_profiles: Vec::new(),
        };
        let tile_size = image_info.get_tile_size();

        assert_eq!(tile_size, Size::new(512, 512));

        let image_info = IiifImageInfo {
            height: 0,
            width: 0,
            profile: vec![IiifProfileInfo::ProfileDetails(IiifProfileDetails {
                formats: vec![IiifImageFormat::Jpg],
                qualities: Vec::new(),
                supports: Vec::new(),
            })],
            tiles: Some(vec![IiifTileInfo {
                width: 100,
                height: Some(110),
                scale_factors: Vec::new(),
            }]),
            sizes: None,
            expanded_profiles: Vec::new(),
        };
        let tile_size = image_info.get_tile_size();

        assert_eq!(tile_size, Size::new(100, 110));

        let image_info = IiifImageInfo {
            height: 0,
            width: 0,
            profile: vec![IiifProfileInfo::ProfileDetails(IiifProfileDetails {
                formats: vec![IiifImageFormat::Jpg],
                qualities: Vec::new(),
                supports: Vec::new(),
            })],
            tiles: Some(vec![IiifTileInfo {
                width: 100,
                height: None,
                scale_factors: Vec::new(),
            }]),
            sizes: None,
            expanded_profiles: Vec::new(),
        };
        let tile_size = image_info.get_tile_size();

        assert_eq!(tile_size, Size::new(100, 100));
    }

    #[test]
    fn test_get_image_sizes() {
        let image_info = IiifImageInfo {
            height: 10,
            width: 20,
            profile: vec![IiifProfileInfo::ProfileDetails(IiifProfileDetails {
                formats: vec![IiifImageFormat::Jpg],
                qualities: Vec::new(),
                supports: Vec::new(),
            })],
            tiles: None,
            sizes: None,
            expanded_profiles: Vec::new(),
        };
        let image_sizes = image_info.get_image_sizes();

        assert_eq!(image_sizes, vec![Size::new(20, 10)]);

        let image_info = IiifImageInfo {
            height: 10,
            width: 20,
            profile: vec![IiifProfileInfo::ProfileDetails(IiifProfileDetails {
                formats: vec![IiifImageFormat::Jpg],
                qualities: Vec::new(),
                supports: Vec::new(),
            })],
            tiles: None,
            sizes: Some(vec![
                Size::new(440, 361),
                Size::new(220, 180),
                Size::new(880, 723),
            ]),
            expanded_profiles: Vec::new(),
        };
        let image_sizes = image_info.get_image_sizes();

        assert_eq!(
            image_sizes,
            vec![
                Size::new(20, 10),
                Size::new(220, 180),
                Size::new(440, 361),
                Size::new(880, 723),
            ]
        );
    }

    #[test]
    fn test_get_profile_details() {
        let image_info = IiifImageInfo {
            height: 10,
            width: 20,
            profile: vec![IiifProfileInfo::ProfileDetails(IiifProfileDetails {
                formats: vec![IiifImageFormat::Png],
                qualities: Vec::new(),
                supports: Vec::new(),
            })],
            tiles: None,
            sizes: None,
            expanded_profiles: vec![IiifProfileDetails {
                formats: vec![IiifImageFormat::Png],
                qualities: Vec::new(),
                supports: Vec::new(),
            }],
        };
        let profile_details = image_info.get_profile_details();

        assert_eq!(profile_details[0].formats, vec![IiifImageFormat::Png]);
    }

    #[test]
    fn test_get_tile_scaling_sizes() {
        let image_info = IiifImageInfo {
            height: 10,
            width: 10,
            profile: vec![IiifProfileInfo::ProfileDetails(IiifProfileDetails {
                formats: vec![IiifImageFormat::Jpg],
                qualities: Vec::new(),
                supports: Vec::new(),
            })],
            tiles: Some(vec![IiifTileInfo {
                width: 100,
                height: None,
                scale_factors: Vec::new(),
            }]),
            sizes: None,
            expanded_profiles: Vec::new(),
        };
        let scaling_sizes = image_info.get_tile_scaling_sizes();

        assert_eq!(scaling_sizes, vec![Size::new(10, 10)]);
    }
}
