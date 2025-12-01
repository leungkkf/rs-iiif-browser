use crate::{
    iiif::{IiifError, manifest::Language, one_or_many::OneTypeOrMany},
    presentation::model::{IsCanvas, IsImage, IsManifest, IsSequence},
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum PresentationType {
    Manifest,
    Collection,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Label {
    Map(HashMap<Language, Vec<String>>),
    Text(String),
}

impl Label {
    fn get(&self, lang: Language) -> Vec<&str> {
        match self {
            Self::Text(v) => vec![v],
            Self::Map(map) => {
                if let Some(v) = map.get(&lang) {
                    v.iter().map(|x| x.as_str()).collect()
                } else if let Some(v) = map.get(&Language::None) {
                    v.iter().map(|x| x.as_str()).collect()
                } else {
                    Vec::new()
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct LabelValue {
    label: Label,
    value: Label,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum DataType {
    Image,
    Dataset,
    Model,
    Sound,
    Text,
    Video,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Service {
    id: String,
    #[serde(rename = "type")]
    service_type: String,
    profile: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Thumbnail {
    id: String,
    #[serde(rename = "type")]
    type_: DataType,
    format: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    duration: Option<u32>,
    service: Option<Vec<Service>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct HomePage {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    label: Label,
    format: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Provider {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    label: Label,
    homepage: OneTypeOrMany<HomePage>,
    logo: OneTypeOrMany<Thumbnail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CanvasItem {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    label: Option<Label>,
    thumbnail: Option<OneTypeOrMany<Thumbnail>>,
    items: Vec<AnnotationPageItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AnnotationPageItem {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    items: Vec<AnnotationItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AnnotationItem {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    body: AnnotationItemBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AnnotationItemBody {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    width: u32,
    height: u32,
    service: Vec<Service>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Manifest {
    #[serde(rename = "@context")]
    context: OneTypeOrMany<String>,
    #[serde(rename = "type")]
    presentation_type: PresentationType,
    label: Label,
    summary: OneTypeOrMany<Label>,
    rights: String,
    required_statement: LabelValue,
    provider: Vec<Provider>,
    items: Vec<CanvasItem>,
}

impl IsManifest for Manifest {
    fn get_title(&self) -> Cow<'_, str> {
        Cow::from(self.label.get(Language::En).join("\n"))
    }

    fn get_attribution(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        Box::new(
            self.provider
                .iter()
                .flat_map(|x| x.label.get(Language::En))
                .map(|x| Cow::from(x))
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }

    fn get_description(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        Box::new(
            self.summary
                .iter()
                .flat_map(|x| x.get(Language::En))
                .map(|x| Cow::from(x))
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }

    fn get_license(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        Box::new(vec![Cow::from(&self.rights)].into_iter())
    }

    fn get_logo(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        Box::new(
            self.provider
                .iter()
                .flat_map(|x| &x.logo)
                .map(|x| Cow::from(&x.id))
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }

    fn get_sequences(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsSequence> + '_> {
        Box::new(vec![self as &dyn IsSequence].into_iter())
    }

    fn get_sequence(&self, _: usize) -> Result<&dyn IsSequence, IiifError> {
        Ok(self as &dyn IsSequence)
    }
}

impl IsSequence for Manifest {
    fn get_label(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        Box::new(std::iter::empty::<Cow<str>>())
    }

    fn get_canvases(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsCanvas> + '_> {
        Box::new(self.items.iter().map(|b| b as &dyn IsCanvas))
    }

    fn get_canvas(&self, index: usize) -> Result<&dyn IsCanvas, IiifError> {
        self.items
            .get(index)
            .map(|x| x as &dyn IsCanvas)
            .ok_or(IiifError::IiifMissingInfo(format!(
                "canvas not found at pos '{}'",
                index
            )))
    }
}

impl IsCanvas for CanvasItem {
    fn get_label(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        if let Some(label) = &self.label {
            return Box::new(
                label
                    .get(Language::En)
                    .iter()
                    .map(|y| Cow::from(*y))
                    .collect::<Vec<_>>()
                    .into_iter(),
            );
        } else {
            return Box::new(Vec::new().into_iter());
        }
    }

    fn get_thumbnail(&self) -> Cow<'_, str> {
        if let Some(thumbnail) = &self.thumbnail
            && let Some(thumbnail) = thumbnail.iter().next()
        {
            return Cow::from(&thumbnail.id);
        } else if let Some(annotation_page) = self.items.first()
            && let Some(image) = annotation_page.items.first()
        {
            let canvas_thumbnail = format!("{}/full/,64/0/default.jpg", image.get_service());

            Cow::from(canvas_thumbnail)
        } else {
            Cow::from("")
        }
    }

    fn get_image(&self, index: usize) -> Result<&dyn IsImage, IiifError> {
        self.items
            .get(index)
            .map(|x| x.items.first())
            .ok_or(IiifError::IiifMissingInfo(format!(
                "missing annotation page at pos '{}'",
                index
            )))?
            .map(|x| x as &dyn IsImage)
            .ok_or(IiifError::IiifMissingInfo(format!(
                "missing annotation item at pos '{}'",
                index
            )))
    }
}

impl IsImage for AnnotationItem {
    fn get_service(&self) -> Cow<'_, str> {
        if let Some(service) = self.body.service.get(0) {
            Cow::from(&service.id)
        } else {
            Cow::from("")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json() {
        let url = "https://iiif.rbge.org.uk/herb/iiif/E00008781/manifest";
        let json = ureq::get(url)
            .call()
            .unwrap()
            .body_mut()
            .read_to_string()
            .unwrap();

        let presentation_info: Manifest = serde_json::from_str(&json).unwrap();

        println!("{:?}", presentation_info);
    }
}
