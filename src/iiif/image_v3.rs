use crate::iiif::image::{IiifFeature, IiifImageFormat, IiifImageQuality};
use crate::rendering::model::{IsImage, IsProfileDetails};
use crate::{iiif::IiifError, rendering::tiled_image::Size};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
enum IiifImageInfoType {
    ImageService3,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IiifImageInfo {
    #[serde(rename(deserialize = "type"))]
    type_: IiifImageInfoType,
    width: u32,
    height: u32,
    sizes: Option<Vec<Size>>,
    tiles: Option<Vec<IiifTileInfo>>,
    profile: String,
    extra_formats: Option<Vec<IiifImageFormat>>,
    extra_qualities: Option<Vec<IiifImageQuality>>,
    extra_features: Option<Vec<IiifFeature>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IiifTileInfo {
    width: u32,
    height: Option<u32>,
    #[serde(rename(deserialize = "scaleFactors"))]
    scale_factors: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct IiifProfileDetails {
    formats: Vec<IiifImageFormat>,
    qualities: Vec<IiifImageQuality>,
    supports: Vec<IiifFeature>,
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

impl TryFrom<&str> for IiifProfileDetails {
    type Error = IiifError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let profile = match value {
            "level0" => Self {
                formats: vec![IiifImageFormat::Jpg],
                qualities: vec![IiifImageQuality::Default],
                supports: vec![IiifFeature::SizeByWhListed],
            },
            "level1" => Self {
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
            "level2" => Self {
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
                    "unexpected profile '{}'",
                    value
                )));
            }
        };

        Ok(profile)
    }
}

pub(crate) struct ImageInfo {
    iiif_image_info: IiifImageInfo,
    expanded_profiles: Vec<IiifProfileDetails>,
}

impl IsProfileDetails for IiifProfileDetails {
    fn get_supported_features(&self) -> Box<dyn ExactSizeIterator<Item = IiifFeature> + '_> {
        Box::new(self.supports.iter().map(|x| x.to_owned()))
    }

    fn get_formats(&self) -> Box<dyn ExactSizeIterator<Item = IiifImageFormat> + '_> {
        Box::new(self.formats.iter().map(|x| x.to_owned()))
    }
}

impl TryFrom<IiifImageInfo> for ImageInfo {
    type Error = IiifError;

    fn try_from(iiif_image_info: IiifImageInfo) -> Result<Self, Self::Error> {
        let expanded_profiles = vec![
            iiif_image_info.profile.as_str().try_into()?,
            IiifProfileDetails {
                formats: iiif_image_info.extra_formats.clone().unwrap_or_default(),
                qualities: iiif_image_info.extra_qualities.clone().unwrap_or_default(),
                supports: iiif_image_info.extra_features.clone().unwrap_or_default(),
            },
        ];

        Ok(Self {
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
    use crate::rendering::model::IsImage;

    #[test]
    fn test_from_json() {
        let json = r#"{
          "@context": [
            "http://example.org/extension/context1.json",
            "http://iiif.io/api/image/3/context.json"
          ],
          "id": "https://example.org/image-service/abcd1234/1E34750D-38DB-4825-A38A-B60A345E591C",
          "type": "ImageService3",
          "protocol": "http://iiif.io/api/image",
          "profile": "level1",
          "width": 6000,
          "height": 4000,
          "maxWidth": 3000,
          "maxHeight": 2000,
          "maxArea": 4000000,
          "sizes": [
            { "width": 150, "height": 100 },
            { "width": 600, "height": 400 },
            { "width": 3000, "height": 2000 }
          ],
          "tiles": [
            { "width": 512, "scaleFactors": [ 1, 2, 4 ] },
            { "width": 1024, "height": 2048, "scaleFactors": [ 8, 16 ] }
          ],
          "rights": "http://rightsstatements.org/vocab/InC-EDU/1.0/",
          "preferredFormats": [ "png", "gif"],
          "extraFormats": [ "png", "gif", "pdf" ],
          "extraQualities": [ "color", "gray" ],
          "extraFeatures": [ "canonicalLinkHeader", "rotationArbitrary", "profileLinkHeader" ],
          "service": [
            {
              "id": "https://example.org/service/example",
              "type": "Service",
              "profile": "https://example.org/docs/example-service.html"
            }
          ]
        }"#;

        let iiif_image_info: IiifImageInfo = serde_json::from_str(json).unwrap();

        assert_eq!(iiif_image_info.width, 6000);
        assert_eq!(iiif_image_info.height, 4000);
        assert_eq!(
            iiif_image_info.sizes,
            Some(vec![
                Size::new(150, 100),
                Size::new(600, 400),
                Size::new(3000, 2000),
            ])
        );

        let tiles = iiif_image_info.tiles.as_ref().unwrap();

        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0].width, 512);
        assert_eq!(tiles[0].height, None);
        assert_eq!(tiles[0].scale_factors, vec![1, 2, 4]);
        assert_eq!(tiles[1].width, 1024);
        assert_eq!(tiles[1].height, Some(2048));
        assert_eq!(tiles[1].scale_factors, vec![8, 16]);

        let image_info: ImageInfo = iiif_image_info.try_into().unwrap();

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
            vec![
                IiifFeature::SizeByWhListed,
                IiifFeature::BaseUriRedirect,
                IiifFeature::Cors,
                IiifFeature::JsonldMediaType,
                IiifFeature::RegionByPx,
                IiifFeature::SizeByH,
                IiifFeature::SizeByPct,
                IiifFeature::SizeByW,
            ]
        );

        assert_eq!(
            image_info.expanded_profiles[1].formats,
            vec![
                IiifImageFormat::Png,
                IiifImageFormat::Gif,
                IiifImageFormat::Pdf
            ]
        );
        assert_eq!(
            image_info.expanded_profiles[1].qualities,
            vec![IiifImageQuality::Color, IiifImageQuality::Gray]
        );
        assert_eq!(
            image_info.expanded_profiles[1].supports,
            vec![
                IiifFeature::CanonicalLinkHeader,
                IiifFeature::RotationArbitrary,
                IiifFeature::ProfileLinkHeader
            ]
        );
    }

    #[test]
    fn test_get_tile_size() {
        let image_info = ImageInfo {
            iiif_image_info: IiifImageInfo {
                height: 0,
                width: 0,
                profile: "level0".to_string(),
                tiles: None,
                sizes: None,
                type_: IiifImageInfoType::ImageService3,
                extra_features: None,
                extra_formats: None,
                extra_qualities: None,
            },
            expanded_profiles: Vec::new(),
        };
        let tile_size = image_info.get_tile_size();

        assert_eq!(tile_size, Size::new(512, 512));

        let image_info = ImageInfo {
            iiif_image_info: IiifImageInfo {
                height: 0,
                width: 0,
                profile: "level0".to_string(),
                tiles: Some(vec![IiifTileInfo {
                    width: 100,
                    height: Some(110),
                    scale_factors: Vec::new(),
                }]),
                sizes: None,
                type_: IiifImageInfoType::ImageService3,
                extra_features: None,
                extra_formats: None,
                extra_qualities: None,
            },
            expanded_profiles: Vec::new(),
        };
        let tile_size = image_info.get_tile_size();

        assert_eq!(tile_size, Size::new(100, 110));

        let image_info = ImageInfo {
            iiif_image_info: IiifImageInfo {
                height: 0,
                width: 0,
                profile: "level0".to_string(),
                tiles: Some(vec![IiifTileInfo {
                    width: 100,
                    height: None,
                    scale_factors: Vec::new(),
                }]),
                sizes: None,
                type_: IiifImageInfoType::ImageService3,
                extra_features: None,
                extra_formats: None,
                extra_qualities: None,
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
                profile: "level0".to_string(),
                tiles: None,
                sizes: None,
                type_: IiifImageInfoType::ImageService3,
                extra_features: None,
                extra_formats: None,
                extra_qualities: None,
            },
            expanded_profiles: Vec::new(),
        };
        let image_sizes = image_info.get_optional_sizes();

        assert_eq!(image_sizes, vec![Size::new(20, 10)]);

        let image_info = ImageInfo {
            iiif_image_info: IiifImageInfo {
                height: 10,
                width: 20,
                profile: "level0".to_string(),
                tiles: None,
                sizes: Some(vec![
                    Size::new(440, 361),
                    Size::new(220, 180),
                    Size::new(880, 723),
                ]),
                type_: IiifImageInfoType::ImageService3,
                extra_features: None,
                extra_formats: None,
                extra_qualities: None,
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
                profile: "level0".to_string(),
                tiles: None,
                sizes: None,
                type_: IiifImageInfoType::ImageService3,
                extra_features: None,
                extra_formats: None,
                extra_qualities: None,
            },
            expanded_profiles: vec![IiifProfileDetails {
                formats: vec![IiifImageFormat::Png],
                qualities: Vec::new(),
                supports: Vec::new(),
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
                profile: "level0".to_string(),
                tiles: Some(vec![IiifTileInfo {
                    width: 100,
                    height: None,
                    scale_factors: Vec::new(),
                }]),
                sizes: None,
                type_: IiifImageInfoType::ImageService3,
                extra_features: None,
                extra_formats: None,
                extra_qualities: None,
            },
            expanded_profiles: Vec::new(),
        };
        let scaling_sizes = image_info.get_tile_scaling_sizes();

        assert_eq!(scaling_sizes, vec![Size::new(10, 10)]);
    }
}
