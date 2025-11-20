use crate::{
    iiif::IiifError,
    rdf::{self, dataset_ext::DatasetExt},
    rendering::tiled_image::Size,
};
use bevy::prelude::debug;
use core::fmt;
use serde::{Deserialize, Serialize};
use sophia::{
    api::{
        dataset::CollectibleDataset,
        term::{SimpleTerm, Term},
    },
    inmem::dataset::FastDataset,
};

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

impl IiifTileInfo {
    fn try_from_dataset<T: CollectibleDataset>(
        tile_object: &SimpleTerm,
        dataset: &DatasetExt<T>,
    ) -> core::result::Result<Self, IiifError> {
        let width = dataset
            .get_objects_as::<_, _, u32>([tile_object], [rdf::exif::width])?
            .first()
            .cloned()
            .ok_or(IiifError::IiifMissingInfo(format!(
                "missing tile width in '{:?}'",
                tile_object
            )))?;

        let height = dataset
            .get_objects_as::<_, _, u32>([tile_object], [rdf::exif::height])?
            .first()
            .cloned();

        let scaling_factors: Vec<u32> =
            dataset.get_objects_as([tile_object], [rdf::iiif_image2::scaleFactor])?;

        Ok(IiifTileInfo {
            width,
            height,
            scale_factors: scaling_factors,
        })
    }
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
    fn new(
        formats: Vec<IiifImageFormat>,
        qualities: Vec<IiifImageQuality>,
        supports: Vec<IiifFeature>,
    ) -> Self {
        Self {
            formats,
            qualities,
            supports,
        }
    }

    pub fn try_from_dataset<T: CollectibleDataset>(
        profile_object: &SimpleTerm,
        dataset: &DatasetExt<T>,
    ) -> core::result::Result<Self, IiifError> {
        let qualities: Vec<IiifImageQuality> =
            dataset.get_objects_as([profile_object], [rdf::iiif_image2::quality])?;
        let formats: Vec<IiifImageFormat> =
            dataset.get_objects_as([profile_object], [rdf::iiif_image2::format])?;
        let supports: Vec<IiifFeature> =
            dataset.get_objects_as([profile_object], [rdf::iiif_image2::supports])?;

        Ok(Self::new(formats, qualities, supports))
    }

    pub(crate) fn from_url(url: &str) -> core::result::Result<IiifProfileDetails, IiifError> {
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
    ArbitraryRotation,
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

impl std::str::FromStr for IiifFeature {
    type Err = IiifError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if rdf::iiif_image2::baseUriRedirectFeature == s {
            Ok(IiifFeature::BaseUriRedirect)
        } else if rdf::iiif_image2::canonicalLinkHeaderFeature == s {
            Ok(IiifFeature::CanonicalLinkHeader)
        } else if rdf::iiif_image2::corsFeature == s {
            Ok(IiifFeature::Cors)
        } else if rdf::iiif_image2::jsonLdMediaTypeFeature == s {
            Ok(IiifFeature::JsonldMediaType)
        } else if rdf::iiif_image2::mirroringFeature == s {
            Ok(IiifFeature::Mirroring)
        } else if rdf::iiif_image2::profileLinkHeaderFeature == s {
            Ok(IiifFeature::ProfileLinkHeader)
        } else if rdf::iiif_image2::regionByPctFeature == s {
            Ok(IiifFeature::RegionByPct)
        } else if rdf::iiif_image2::regionByPxFeature == s {
            Ok(IiifFeature::RegionByPx)
        } else if rdf::iiif_image2::regionSquareFeature == s {
            Ok(IiifFeature::RegionSquare)
        } else if rdf::iiif_image2::arbitraryRotationFeature == s {
            Ok(IiifFeature::ArbitraryRotation)
        } else if rdf::iiif_image2::rotationBy90sFeature == s {
            Ok(IiifFeature::RotationBy90s)
        } else if rdf::iiif_image2::sizeByConfinedWHFeature == s {
            Ok(IiifFeature::SizeByConfinedWh)
        } else if rdf::iiif_image2::sizeByHFeature == s {
            Ok(IiifFeature::SizeByH)
        } else if rdf::iiif_image2::sizeByPctFeature == s {
            Ok(IiifFeature::SizeByPct)
        } else if rdf::iiif_image2::sizeByWFeature == s {
            Ok(IiifFeature::SizeByW)
        } else if rdf::iiif_image2::sizeByWHFeature == s {
            Ok(IiifFeature::SizeByWh)
        } else if rdf::iiif_image2::sizeUpscalingFeature == s {
            Ok(IiifFeature::SizeUpscaling)
        } else if rdf::iiif_image2::sizeByWHListedFeature == s {
            Ok(IiifFeature::SizeByWhListed)
        } else if rdf::iiif_image2::sizeByForcedWHFeature == s {
            Ok(IiifFeature::SizeByForcedWh)
        } else if rdf::iiif_image2::sizeAboveFullFeature == s {
            Ok(IiifFeature::SizeAboveFull)
        } else if rdf::iiif_image2::sizeByDistortedWHFeature == s {
            Ok(IiifFeature::SizeByDistortedWh)
        } else {
            Err(IiifError::IiifFormatError(format!(
                "failed to convert '{}' to IiifFeature",
                s
            )))
        }
    }
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
            IiifImageQuality::Native => write!(f, "native"),
        }
    }
}

impl std::str::FromStr for IiifImageQuality {
    type Err = IiifError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "native" => Ok(IiifImageQuality::Native),
            "default" => Ok(IiifImageQuality::Default),
            "bitonal" => Ok(IiifImageQuality::Bitonal),
            "color" => Ok(IiifImageQuality::Color),
            "gray" => Ok(IiifImageQuality::Gray),
            _ => Err(IiifError::IiifFormatError(format!(
                "failed to convert '{}' to IiifImageQuality",
                s
            ))),
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

impl std::str::FromStr for IiifImageFormat {
    type Err = IiifError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "jpg" => Ok(IiifImageFormat::Jpg),
            "png" => Ok(IiifImageFormat::Png),
            "tif" => Ok(IiifImageFormat::Tif),
            "gif" => Ok(IiifImageFormat::Gif),
            _ => Err(IiifError::IiifFormatError(format!(
                "failed to convert '{}' to IiifImageFormat",
                s
            ))),
        }
    }
}

impl IiifImageInfo {
    pub(crate) fn try_from_url(url: &str) -> core::result::Result<Self, IiifError> {
        let info_json = ureq::get(url).call()?.body_mut().read_to_string()?;
        debug!("info {:?}", info_json);

        let dataset = DatasetExt::<FastDataset>::try_from_json(&info_json)?;

        Self::try_from_database(&dataset)
    }

    fn try_from_database<T: CollectibleDataset>(
        dataset: &DatasetExt<T>,
    ) -> core::result::Result<Self, IiifError> {
        let id_subject = dataset.id();

        let profile_details = dataset
            .objects_iter([id_subject], [rdf::doap::implements])
            .map(|profile_object| {
                let profile_object = &profile_object?;

                match profile_object.kind() {
                    sophia::api::term::TermKind::Iri => Ok(IiifProfileDetails::from_url(
                        &profile_object
                            .iri()
                            .ok_or(IiifError::IiifFormatError(format!(
                                "failed to get profile url at '{}'",
                                id_subject
                            )))?
                            .as_ref(),
                    )?),
                    sophia::api::term::TermKind::BlankNode => {
                        IiifProfileDetails::try_from_dataset(profile_object, dataset)
                    }
                    _ => Err(IiifError::IiifFormatError(format!(
                        "unexpected term kind in image profile {:?}",
                        profile_object.kind()
                    ))),
                }
            })
            .collect::<Result<Vec<IiifProfileDetails>, IiifError>>()?;

        let image_width = dataset
            .get_objects_as::<_, _, u32>([id_subject], [rdf::exif::width])?
            .first()
            .copied()
            .ok_or(IiifError::IiifMissingInfo(format!(
                "missing width in '{}'",
                id_subject
            )))?;
        let image_height = dataset
            .get_objects_as::<_, _, u32>([id_subject], [rdf::exif::height])?
            .first()
            .copied()
            .ok_or(IiifError::IiifMissingInfo(format!(
                "missing height in '{}'",
                id_subject
            )))?;

        let mut tile_info = Vec::new();

        for tile_node in dataset.objects_iter([id_subject], [rdf::iiif_image2::hasTile]) {
            tile_info.push(IiifTileInfo::try_from_dataset(&tile_node?, dataset)?);
        }

        let image_sizes = dataset
            .objects_iter([id_subject], [rdf::iiif_image2::hasSize])
            .map(|x| {
                let x = &x?;

                let width = dataset
                    .get_objects_as::<_, _, u32>([x], [rdf::exif::width])?
                    .first()
                    .cloned()
                    .ok_or(IiifError::IiifMissingInfo(format!(
                        "missing height in image size in '{}'",
                        id_subject
                    )));
                let height = dataset
                    .get_objects_as::<_, _, u32>([x], [rdf::exif::height])?
                    .first()
                    .cloned()
                    .ok_or(IiifError::IiifMissingInfo(format!(
                        "missing height in image size in '{}'",
                        id_subject
                    )));

                Ok(Size::new(width?, height?))
            })
            .collect::<Result<Vec<Size>, IiifError>>()?;

        let image_sizes = if !image_sizes.is_empty() {
            Some(image_sizes)
        } else {
            None
        };

        let tile_info = if !tile_info.is_empty() {
            Some(tile_info)
        } else {
            None
        };

        Ok(IiifImageInfo {
            height: image_height,
            width: image_width,
            profile: Vec::new(),
            sizes: image_sizes,
            tiles: tile_info,
            expanded_profiles: profile_details,
        })
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
    use sophia::inmem::dataset::FastDataset;

    #[test]
    fn test_supports_features() {
        let json = r#"{"@context":"http://iiif.io/api/image/2/context.json","@id":"https://mps.lib.harvard.edu/assets/images/drs:20430287","protocol":"http://iiif.io/api/image","width":1074,"height":2550,"sizes":[{"width":67,"height":159},{"width":134,"height":319},{"width":269,"height":638},{"width":537,"height":1275},{"width":1074,"height":2550}],"tiles":[{"width":1024,"height":1024,"scaleFactors":[1,2,4,8,16]}],"profile":["http://iiif.io/api/image/2/level2.json",{"formats":["jpg","tif","gif","png"],"qualities":["bitonal","default","gray","color"],"supports":["regionByPx","sizeByW","sizeByWhListed","cors","regionSquare","sizeByDistortedWh","canonicalLinkHeader","sizeByConfinedWh","sizeByPct","jsonldMediaType","regionByPct","rotationArbitrary","sizeByH","baseUriRedirect","rotationBy90s","profileLinkHeader","sizeByForcedWh","sizeByWh","mirroring"]}]}"#;

        let dataset = DatasetExt::<FastDataset>::try_from_json(json).unwrap();

        assert!(IiifImageInfo::try_from_database(&dataset).is_ok());
    }

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
        let dataset = DatasetExt::<FastDataset>::try_from_json(json).unwrap();

        let image_info = IiifImageInfo::try_from_database(&dataset).unwrap();

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
