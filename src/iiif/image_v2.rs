use crate::iiif::image::{IiifFeature, IiifImageFormat, IiifImageQuality};
use crate::iiif::one_or_many::OneTypeOrMany;
use crate::rendering::model::{IsImage, IsProfileDetails};
use crate::{iiif::IiifError, rendering::tiled_image::Size};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IiifImageInfo {
    width: u32,
    height: u32,
    sizes: Option<Vec<Size>>,
    tiles: Option<Vec<IiifTileInfo>>,
    profile: OneTypeOrMany<IiifProfileInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IiifTileInfo {
    width: u32,
    height: Option<u32>,
    #[serde(rename(deserialize = "scaleFactors"))]
    scale_factors: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum IiifProfileInfo {
    Url(String),
    ProfileDetails(IiifProfileDetails),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct IiifProfileDetails {
    formats: Option<Vec<IiifImageFormat>>,
    qualities: Option<Vec<IiifImageQuality>>,
    supports: Option<Vec<IiifFeature>>,
}

impl Default for IiifProfileDetails {
    fn default() -> Self {
        Self {
            formats: Some(vec![IiifImageFormat::Jpg]),
            qualities: Some(vec![IiifImageQuality::Default]),
            supports: Some(vec![]),
        }
    }
}

pub(crate) struct ImageInfo {
    iiif_image_info: IiifImageInfo,
    expanded_profiles: Vec<IiifProfileDetails>,
}

impl IiifProfileDetails {
    fn from_url(url: &str) -> core::result::Result<IiifProfileDetails, IiifError> {
        let profile = match url {
            "http://iiif.io/api/image/2/level0.json"
            | "https://iiif.io/api/image/2/level0.json" => Self {
                formats: Some(vec![IiifImageFormat::Jpg]),
                qualities: Some(vec![IiifImageQuality::Default]),
                supports: Some(vec![IiifFeature::SizeByWhListed]),
            },
            "http://iiif.io/api/image/2/level1.json"
            | "https://iiif.io/api/image/2/level1.json" => Self {
                formats: Some(vec![IiifImageFormat::Jpg]),
                qualities: Some(vec![IiifImageQuality::Default]),
                supports: Some(vec![
                    IiifFeature::SizeByWhListed,
                    IiifFeature::BaseUriRedirect,
                    IiifFeature::Cors,
                    IiifFeature::JsonldMediaType,
                    IiifFeature::RegionByPx,
                    IiifFeature::SizeByH,
                    IiifFeature::SizeByPct,
                    IiifFeature::SizeByW,
                ]),
            },
            "http://iiif.io/api/image/2/level2.json"
            | "https://iiif.io/api/image/2/level2.json" => Self {
                formats: Some(vec![IiifImageFormat::Jpg, IiifImageFormat::Png]),
                qualities: Some(vec![IiifImageQuality::Default, IiifImageQuality::Bitonal]),
                supports: Some(vec![
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
                ]),
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

impl IsProfileDetails for IiifProfileDetails {
    fn get_supported_features(&self) -> Box<dyn ExactSizeIterator<Item = IiifFeature> + '_> {
        match &self.supports {
            None => Box::new(Vec::new().into_iter()),
            Some(v) => Box::new(v.iter().map(|x| x.to_owned())),
        }
    }

    fn get_formats(&self) -> Box<dyn ExactSizeIterator<Item = IiifImageFormat> + '_> {
        match &self.formats {
            None => Box::new(Vec::new().into_iter()),
            Some(v) => Box::new(v.iter().map(|x| x.to_owned())),
        }
    }
}

impl TryFrom<IiifImageInfo> for ImageInfo {
    type Error = IiifError;

    fn try_from(iiif_image_info: IiifImageInfo) -> Result<Self, Self::Error> {
        let iiif_image_info_profiles: Vec<_> = iiif_image_info.profile.iter().collect();

        if iiif_image_info_profiles.is_empty() {
            return Err(IiifError::IiifMissingInfo("Missing profile".into()));
        }

        let mut expanded_profiles = Vec::new();

        for p in iiif_image_info_profiles {
            let profile = match p {
                IiifProfileInfo::ProfileDetails(profile_details) => (*profile_details).clone(),
                IiifProfileInfo::Url(url) => IiifProfileDetails::from_url(url)?,
            };
            expanded_profiles.push(profile);
        }

        Ok(ImageInfo {
            iiif_image_info,
            expanded_profiles,
        })
    }
}

impl IsImage for ImageInfo {
    fn get_optional_sizes(&self) -> Vec<Size> {
        let mut image_sizes = Vec::new();

        if let Some(sizes) = &self.iiif_image_info.sizes {
            image_sizes = sizes.clone();

            if !image_sizes.iter().any(|x| {
                x.width == self.iiif_image_info.width && x.height == self.iiif_image_info.height
            }) {
                image_sizes.push(Size::new(
                    self.iiif_image_info.width,
                    self.iiif_image_info.height,
                ));
            }
        } else {
            image_sizes.push(Size::new(
                self.iiif_image_info.width,
                self.iiif_image_info.height,
            ));
        }
        image_sizes.sort_by(|a, b| (a.width * a.height).cmp(&(b.width * b.height)));

        image_sizes
    }

    fn get_profile_details(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsProfileDetails> + '_> {
        Box::new(
            self.expanded_profiles
                .iter()
                .map(|x| x as &dyn IsProfileDetails),
        )
    }

    fn get_tile_scaling_sizes(&self) -> Vec<Size> {
        let mut scaling_sizes = Vec::new();

        if let Some(tiles) = &self.iiif_image_info.tiles
            && let Some(tile) = tiles.first()
        {
            scaling_sizes = tile
                .scale_factors
                .iter()
                .map(|f| {
                    Size::new(
                        self.iiif_image_info.width / f,
                        self.iiif_image_info.height / f,
                    )
                })
                .collect();
        }

        let default_size = Size::new(self.iiif_image_info.width, self.iiif_image_info.height);

        if !scaling_sizes
            .iter()
            .any(|x| x.width == default_size.width && x.height == default_size.height)
        {
            scaling_sizes.push(default_size);
        }

        scaling_sizes.sort_by(|a, b| (a.width * a.height).cmp(&(b.width * b.height)));

        scaling_sizes
    }

    fn get_tile_size(&self) -> Size {
        let default_tile_size = Size::new(512, 512);

        if let Some(tiles) = &self.iiif_image_info.tiles {
            tiles.first().map_or(default_tile_size, |x| {
                Size::new(x.width, x.height.unwrap_or(x.width))
            })
        } else {
            default_tile_size
        }
    }

    fn get_width(&self) -> u32 {
        self.iiif_image_info.width
    }

    fn get_height(&self) -> u32 {
        self.iiif_image_info.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iiif::one_or_many::OneTypeOrMany;
    use crate::rendering::model::IsImage;

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

        let iiif_image_info: IiifImageInfo = serde_json::from_str(json).unwrap();

        assert_eq!(iiif_image_info.width, 7045);
        assert_eq!(iiif_image_info.height, 5785);
        assert_eq!(
            iiif_image_info.sizes,
            Some(vec![
                Size::new(440, 361),
                Size::new(220, 180),
                Size::new(880, 723),
            ])
        );

        let tiles = iiif_image_info.tiles.as_ref().unwrap();

        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].width, 256);
        assert_eq!(tiles[0].height, Some(256));
        assert_eq!(tiles[0].scale_factors, vec![1, 2, 4, 8, 16, 32]);

        let image_info_profiles: Vec<_> = iiif_image_info.profile.iter().collect();
        assert_eq!(image_info_profiles.len(), 2);

        for p in &iiif_image_info.profile {
            match p {
                IiifProfileInfo::Url(url) => {
                    assert_eq!(url, "http://iiif.io/api/image/2/level0.json");
                }
                IiifProfileInfo::ProfileDetails(detail) => {
                    assert_eq!(detail.formats, Some(vec![IiifImageFormat::Jpg]));
                    assert_eq!(
                        detail.qualities,
                        Some(vec![
                            IiifImageQuality::Native,
                            IiifImageQuality::Color,
                            IiifImageQuality::Gray
                        ])
                    );
                    assert_eq!(
                        detail.supports,
                        Some(vec![
                            IiifFeature::RegionByPct,
                            IiifFeature::RegionSquare,
                            IiifFeature::SizeByForcedWh,
                            IiifFeature::SizeByWh,
                            IiifFeature::SizeAboveFull,
                            IiifFeature::RotationBy90s,
                            IiifFeature::Mirroring
                        ])
                    );
                }
            }
        }

        let image_info: ImageInfo = iiif_image_info.try_into().unwrap();

        assert_eq!(
            image_info.expanded_profiles[0].formats,
            Some(vec![IiifImageFormat::Jpg])
        );
        assert_eq!(
            image_info.expanded_profiles[0].qualities,
            Some(vec![IiifImageQuality::Default,])
        );
        assert_eq!(
            image_info.expanded_profiles[0].supports,
            Some(vec![IiifFeature::SizeByWhListed])
        );

        assert_eq!(
            image_info.expanded_profiles[1].formats,
            Some(vec![IiifImageFormat::Jpg])
        );
        assert_eq!(
            image_info.expanded_profiles[1].qualities,
            Some(vec![
                IiifImageQuality::Native,
                IiifImageQuality::Color,
                IiifImageQuality::Gray
            ])
        );
        assert_eq!(
            image_info.expanded_profiles[1].supports,
            Some(vec![
                IiifFeature::RegionByPct,
                IiifFeature::RegionSquare,
                IiifFeature::SizeByForcedWh,
                IiifFeature::SizeByWh,
                IiifFeature::SizeAboveFull,
                IiifFeature::RotationBy90s,
                IiifFeature::Mirroring
            ])
        );
    }

    #[test]
    fn test_get_tile_size() {
        let image_info = ImageInfo {
            iiif_image_info: IiifImageInfo {
                height: 0,
                width: 0,
                profile: OneTypeOrMany::<IiifProfileInfo>::Many(vec![
                    IiifProfileInfo::ProfileDetails(IiifProfileDetails {
                        formats: Some(vec![IiifImageFormat::Jpg]),
                        qualities: Some(Vec::new()),
                        supports: Some(Vec::new()),
                    }),
                ]),
                tiles: None,
                sizes: None,
            },
            expanded_profiles: Vec::new(),
        };
        let tile_size = image_info.get_tile_size();

        assert_eq!(tile_size, Size::new(512, 512));

        let image_info = ImageInfo {
            iiif_image_info: IiifImageInfo {
                height: 0,
                width: 0,
                profile: OneTypeOrMany::<IiifProfileInfo>::Many(vec![
                    IiifProfileInfo::ProfileDetails(IiifProfileDetails {
                        formats: Some(vec![IiifImageFormat::Jpg]),
                        qualities: Some(Vec::new()),
                        supports: Some(Vec::new()),
                    }),
                ]),
                tiles: Some(vec![IiifTileInfo {
                    width: 100,
                    height: Some(110),
                    scale_factors: Vec::new(),
                }]),
                sizes: None,
            },
            expanded_profiles: Vec::new(),
        };
        let tile_size = image_info.get_tile_size();

        assert_eq!(tile_size, Size::new(100, 110));

        let image_info = ImageInfo {
            iiif_image_info: IiifImageInfo {
                height: 0,
                width: 0,
                profile: OneTypeOrMany::<IiifProfileInfo>::Many(vec![
                    IiifProfileInfo::ProfileDetails(IiifProfileDetails {
                        formats: Some(vec![IiifImageFormat::Jpg]),
                        qualities: Some(Vec::new()),
                        supports: Some(Vec::new()),
                    }),
                ]),
                tiles: Some(vec![IiifTileInfo {
                    width: 100,
                    height: None,
                    scale_factors: Vec::new(),
                }]),
                sizes: None,
            },
            expanded_profiles: Vec::new(),
        };
        let tile_size = image_info.get_tile_size();

        assert_eq!(tile_size, Size::new(100, 100));
    }

    #[test]
    fn test_get_optional_sizes() {
        let image_info = ImageInfo {
            iiif_image_info: IiifImageInfo {
                height: 10,
                width: 20,
                profile: OneTypeOrMany::<IiifProfileInfo>::Many(vec![
                    IiifProfileInfo::ProfileDetails(IiifProfileDetails {
                        formats: Some(vec![IiifImageFormat::Jpg]),
                        qualities: Some(Vec::new()),
                        supports: Some(Vec::new()),
                    }),
                ]),
                tiles: None,
                sizes: None,
            },
            expanded_profiles: Vec::new(),
        };
        let image_sizes = image_info.get_optional_sizes();

        assert_eq!(image_sizes, vec![Size::new(20, 10)]);

        let image_info = ImageInfo {
            iiif_image_info: IiifImageInfo {
                height: 10,
                width: 20,
                profile: OneTypeOrMany::<IiifProfileInfo>::Many(vec![
                    IiifProfileInfo::ProfileDetails(IiifProfileDetails {
                        formats: Some(vec![IiifImageFormat::Jpg]),
                        qualities: Some(Vec::new()),
                        supports: Some(Vec::new()),
                    }),
                ]),
                tiles: None,
                sizes: Some(vec![
                    Size::new(440, 361),
                    Size::new(220, 180),
                    Size::new(880, 723),
                ]),
            },
            expanded_profiles: Vec::new(),
        };
        let image_sizes = image_info.get_optional_sizes();

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
        let image_info = ImageInfo {
            iiif_image_info: IiifImageInfo {
                height: 10,
                width: 20,
                profile: OneTypeOrMany::<IiifProfileInfo>::Many(vec![
                    IiifProfileInfo::ProfileDetails(IiifProfileDetails {
                        formats: Some(vec![IiifImageFormat::Png]),
                        qualities: Some(Vec::new()),
                        supports: Some(Vec::new()),
                    }),
                ]),
                tiles: None,
                sizes: None,
            },
            expanded_profiles: vec![IiifProfileDetails {
                formats: Some(vec![IiifImageFormat::Png]),
                qualities: Some(Vec::new()),
                supports: Some(Vec::new()),
            }],
        };

        assert_eq!(
            image_info
                .get_profile_details()
                .next()
                .unwrap()
                .get_formats()
                .collect::<Vec<_>>(),
            vec![IiifImageFormat::Png]
        );
    }

    #[test]
    fn test_get_tile_scaling_sizes() {
        let image_info = ImageInfo {
            iiif_image_info: IiifImageInfo {
                height: 10,
                width: 10,
                profile: OneTypeOrMany::<IiifProfileInfo>::Many(vec![
                    IiifProfileInfo::ProfileDetails(IiifProfileDetails {
                        formats: Some(vec![IiifImageFormat::Jpg]),
                        qualities: Some(Vec::new()),
                        supports: Some(Vec::new()),
                    }),
                ]),
                tiles: Some(vec![IiifTileInfo {
                    width: 100,
                    height: None,
                    scale_factors: Vec::new(),
                }]),
                sizes: None,
            },
            expanded_profiles: Vec::new(),
        };
        let scaling_sizes = image_info.get_tile_scaling_sizes();

        assert_eq!(scaling_sizes, vec![Size::new(10, 10)]);
    }
}
