use crate::tiled_image::Size;
use bevy::prelude::debug;
use core::fmt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IiifError {
    #[error("ureq error")]
    Web(#[from] ureq::Error),

    #[error("serde_json deserialization error")]
    Deserialization(#[from] serde_json::Error),

    #[error("IIIF missing info")]
    IiifMissingInfo(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IiifImageInfo {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) sizes: Option<Vec<Size>>,
    pub(crate) tiles: Option<Vec<IiifTileInfo>>,
    pub(crate) profile: Vec<IiifProfileInfo>,
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
            qualities: Vec::new(),
            supports: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
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
    SizeByForcedWh, // old?
    SizeAboveFull,  // old?
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum IiifImageFormat {
    Jpg,
    Png,
}

impl fmt::Display for IiifImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IiifImageFormat::Jpg => write!(f, "jpg"),
            IiifImageFormat::Png => write!(f, "png"),
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
        let iiif_image_info: IiifImageInfo = serde_json::from_str(&info_json)?;
        debug!("iiif_image_info {:?}", iiif_image_info);

        if iiif_image_info.profile.len() == 0 {
            return Err(IiifError::IiifMissingInfo("Missing profile".into()));
        }

        Ok(iiif_image_info)
    }

    /// Get tile size.
    pub(crate) fn get_tile_size(&self) -> Size {
        let default_tile_size = Size::new(512, 512);
        if let Some(tiles) = &self.tiles {
            tiles.get(0).map_or(default_tile_size, |x| {
                Size::new(x.width, x.height.unwrap_or(x.width))
            })
        } else {
            default_tile_size
        }
    }

    /// Get profile details.
    pub(crate) fn get_profile_details(&self) -> IiifProfileDetails {
        self.profile
            .iter()
            .find_map(|x| {
                if let IiifProfileInfo::ProfileDetails(profile_details) = x {
                    Some((*profile_details).clone())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    /// Get image sizes.
    pub(crate) fn get_image_sizes(&self) -> Vec<Size> {
        let mut image_sizes = Vec::new();

        if let Some(sizes) = &self.sizes {
            image_sizes = sizes.clone();

            if image_sizes
                .iter()
                .find(|x| x.width == self.width && x.height == self.height)
                .is_none()
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
                "http://iiif.io/api/image/2/level1.json",
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
                    assert_eq!(url, "http://iiif.io/api/image/2/level1.json");
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
        };
        let profile_details = image_info.get_profile_details();

        assert_eq!(profile_details.formats, vec![IiifImageFormat::Png]);
    }
}
