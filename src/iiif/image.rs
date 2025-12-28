use crate::iiif::{IiifError, image_v2, image_v3};
use crate::rendering::model::IsImage;
use bevy::prelude::debug;
use core::fmt;
use serde::{Deserialize, Serialize};

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
            IiifImageQuality::Native => write!(f, "native"),
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
    Txt,
    Jp2,
    Pdf,
    Webp,
}

impl fmt::Display for IiifImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IiifImageFormat::Jpg => write!(f, "jpg"),
            IiifImageFormat::Png => write!(f, "png"),
            IiifImageFormat::Tif => write!(f, "tif"),
            IiifImageFormat::Gif => write!(f, "gif"),
            IiifImageFormat::Txt => write!(f, "txt"),
            IiifImageFormat::Jp2 => write!(f, "jpg2"),
            IiifImageFormat::Pdf => write!(f, "pdf"),
            IiifImageFormat::Webp => write!(f, "webp"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum IiifImageInfo {
    Version3(image_v3::IiifImageInfo),
    Version2(image_v2::IiifImageInfo),
}

impl IiifImageInfo {
    /// Build from a Json string.
    pub(crate) fn try_from_json(
        info_json: &str,
    ) -> core::result::Result<Box<dyn IsImage>, IiifError> {
        let iiif_image_info: IiifImageInfo = serde_json::from_str(info_json)?;
        debug!("iiif_image_info {:?}", iiif_image_info);

        let output = match iiif_image_info {
            IiifImageInfo::Version2(v) => {
                let image_info: image_v2::ImageInfo = v.try_into()?;

                Box::new(image_info) as Box<dyn IsImage>
            }
            IiifImageInfo::Version3(v) => {
                let image_info: image_v3::ImageInfo = v.try_into()?;

                Box::new(image_info) as Box<dyn IsImage>
            }
        };

        Ok(output)
    }
}
