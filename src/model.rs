use crate::{
    api::Api,
    helpers::send_req,
    request::{CraiyonRequest, CraiyonResponse},
    utils::{MODEL_VER, URL_IMAGE},
};

use clap::ValueEnum;
use image::DynamicImage;
use std::{error::Error, fmt::Display};

#[derive(Default, Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Model<'a> {
    model: ModelType,
    version: Api,
    api_token: Option<&'a str>,
    // TODO: Add client
}

#[allow(dead_code)]
impl<'a> Model<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn version(mut self, ver: Api) -> Self {
        self.version = ver;
        self
    }

    pub fn api_token(mut self, api_token: Option<&'a str>) -> Self {
        self.api_token = api_token;
        self
    }

    pub fn model_type(mut self, mod_type: ModelType) -> Self {
        self.model = mod_type;
        self
    }

    pub fn from(model: ModelType, version: Api) -> Self {
        Self {
            model,
            version,
            api_token: None,
        }
    }

    #[allow(dead_code)]
    pub async fn generate_from_prompt(
        &self,
        prompt: &str,
        num_images: usize,
    ) -> Result<Vec<DynamicImage>, Box<dyn Error>> {
        self.generate(prompt, "", num_images).await
    }

    #[allow(dead_code)]
    pub async fn generate(
        &self,
        prompt: &str,
        negative_prompt: &str,
        num_images: usize,
    ) -> Result<Vec<DynamicImage>, Box<dyn Error>> {
        // FIXME add paralelisation for more than 9 images (max 9 images per request)
        if num_images > 9 {
            return Err("Number of images must be between 1 and 9".into()); // TODO: Add
                                                                           // paralelisation for more than 9 images (max 9 images per request)
        }

        let model = self.model.as_str();

        let data = match self.version {
            Api::V1 => {
                // Dall-e Mini V1 is not supported anymore.
                // The breaking change is due to the fact that the API has changed on about 10th of July 2023.
                // PR's are welcome

                panic!("V1 is not supported anymore. Please use V3 instead.");

                /*
                    CraiyonRequest::V1 {
                        prompt: Some(prompt),
                    }
                */
            }

            Api::V3 => CraiyonRequest::V3 {
                prompt: Some(prompt),
                negative_prompt: Some(negative_prompt),
                model: Some(model),
                version: Some(MODEL_VER),
                token: self.api_token,
            },
        };

        let response = send_req(self.version.as_str(), &data).await?;

        let res: CraiyonResponse = response.json().await?;

        let image_urls: Vec<String> = res // FIXME here are some heap allocations.
            .images
            .iter()
            .take(num_images)
            .map(|image| format!("{}/{}", URL_IMAGE, image))
            .collect();

        let mut image_buf: Vec<DynamicImage> = Vec::with_capacity(image_urls.len());

        for image_url in image_urls {
            let pixels = reqwest::blocking::get(image_url)?.bytes()?.to_vec();

            let image = image::load_from_memory(&pixels)?;

            image_buf.push(image);
        }
        Ok(image_buf)
    }

    async fn generate_api_chunks<T>(&self, data: CraiyonRequest<'_>, image_buf: &mut Vec<DynamicImage>) -> Result<(), Box<dyn Error>>
        where
            T: AsRef<str>
    {
        let response = send_req(self.version.as_str(), &data).await?;

        let res: CraiyonResponse = response.json().await?;

        let image_urls: Vec<String> = res // FIXME here are some heap allocations.
            .images
            .iter()
            .map(|image| format!("{}/{}", URL_IMAGE, image))
            .collect();

        for image_url in image_urls {
            let pixels = reqwest::blocking::get(image_url)?.bytes()?.to_vec();

            let image = image::load_from_memory(&pixels)?;

            image_buf.push(image);
        }
        Ok(())
    }

    async fn generate_exact<T>(&self, data: CraiyonRequest<'_>, image_buf: &mut Vec<DynamicImage>, num_images: usize) -> Result<(), Box<dyn Error>>
        where
            T: AsRef<str>
    {
        let response = send_req(self.version.as_str(), &data).await?;

        let res: CraiyonResponse = response.json().await?;

        let image_urls: Vec<String> = res // FIXME here are some heap allocations.
            .images
            .iter()
            .take(num_images)
            .map(|image| format!("{}/{}", URL_IMAGE, image))
            .collect();

        for image_url in image_urls {
            let pixels = reqwest::blocking::get(image_url)?.bytes()?.to_vec();

            let image = image::load_from_memory(&pixels)?;

            image_buf.push(image);
        }
        Ok(())
    }
}

// TOOD: Add macro to overload Model::generate() and Model::generate_from_prompt().

/// Variants of craiyon::Model
#[allow(dead_code)]
#[derive(Debug, Default, Clone, Eq, PartialEq, PartialOrd, Ord, ValueEnum)]
pub enum ModelType {
    Art,
    Drawing,
    Photo,
    #[default]
    General,
}

impl ModelType {
    fn as_str(&self) -> &str {
        match self {
            ModelType::Art => "art",
            ModelType::Drawing => "drawing",
            ModelType::Photo => "photo",
            ModelType::General => "none",
        }
    }
}

impl Display for ModelType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        match self {
            ModelType::Art => f.write_str("art"),
            ModelType::Drawing => f.write_str("drawing"),
            ModelType::Photo => f.write_str("photo"),
            ModelType::General => f.write_str("none"),
        }
    }
}
