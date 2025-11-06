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
    // #[error("IIIF missing info")]
    // IiifMissingInfo(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IiifImageInfo {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) sizes: Vec<Size>,
    pub(crate) tiles: Option<Vec<IiifTileInfo>>,
    pub(crate) profile: Vec<IiifProfileInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IiifTileInfo {
    pub(crate) width: u32,
    pub(crate) height: u32,
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
    pub(crate) qualities: Vec<String>,
    pub(crate) supports: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) enum IiifImageFormat {
    #[serde(rename(deserialize = "jpg"))]
    Jpg,
    #[serde(rename(deserialize = "png"))]
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
    pub(crate) fn new(url: String) -> core::result::Result<Self, IiifError> {
        let info_json = ureq::get(url).call()?.body_mut().read_to_string()?;
        debug!("info {:?}", info_json);

        Self::from_json(&info_json)
    }

    fn from_json(info_json: &str) -> core::result::Result<Self, IiifError> {
        let mut iiif_image_info: IiifImageInfo = serde_json::from_str(&info_json)?;
        debug!("iiif_image_info {:?}", iiif_image_info);

        if iiif_image_info
            .sizes
            .iter()
            .find(|x| x.width == iiif_image_info.width && x.height == iiif_image_info.height)
            .is_none()
        {
            iiif_image_info
                .sizes
                .push(Size::new(iiif_image_info.width, iiif_image_info.height));
        }

        iiif_image_info
            .sizes
            .sort_by(|a, b| (a.width * a.height).cmp(&(b.width * b.height)));

        Ok(iiif_image_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_from_json() {
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
            vec![
                Size::new(220, 180),
                Size::new(440, 361),
                Size::new(880, 723),
                Size::new(7045, 5785),
            ]
        );

        let tiles = image_info.tiles.unwrap();

        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].width, 256);
        assert_eq!(tiles[0].height, 256);
        assert_eq!(tiles[0].scale_factors, vec![1, 2, 4, 8, 16, 32]);

        assert_eq!(image_info.profile.len(), 2);

        for p in &image_info.profile {
            match p {
                IiifProfileInfo::Url(url) => {
                    assert_eq!(url, "http://iiif.io/api/image/2/level1.json");
                }
                IiifProfileInfo::ProfileDetails(detail) => {
                    assert_eq!(detail.formats, vec![IiifImageFormat::Jpg]);
                    assert_eq!(detail.qualities, vec!["native", "color", "gray"]);
                    assert_eq!(
                        detail.supports,
                        vec![
                            "regionByPct",
                            "regionSquare",
                            "sizeByForcedWh",
                            "sizeByWh",
                            "sizeAboveFull",
                            "rotationBy90s",
                            "mirroring"
                        ]
                    );
                }
            }
        }
    }
}
