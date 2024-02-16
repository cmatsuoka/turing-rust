use serde::Deserialize;
use std::collections::HashMap;
use std::error;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Clone, Deserialize)]
struct Display {
    #[serde(alias = "DISPLAY_ORIENTATION")]
    display_orientation: String,
    #[serde(alias = "DISPLAY_RGB_LED")]
    display_rgb_led: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Background {
    #[serde(alias = "PATH")]
    path: String,
    #[serde(alias = "X")]
    x: u32,
    #[serde(alias = "Y")]
    y: u32,
    #[serde(alias = "WIDTH")]
    width: u32,
    #[serde(alias = "HEIGHT")]
    height: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct StaticImages {
    #[serde(alias = "BACKGROUND")]
    background: Background,
}

#[derive(Debug, Clone, Deserialize)]
struct Text {
    #[serde(alias = "SHOW")]
    show: bool,
    #[serde(alias = "SHOW_UNIT")]
    show_unit: bool,
    #[serde(alias = "X")]
    x: u32,
    #[serde(alias = "Y")]
    y: u32,
    #[serde(alias = "FONT")]
    font: String,
    #[serde(alias = "FONT_SIZE")]
    font_size: u32,
    #[serde(alias = "FONT_COLOR")]
    font_color: String,
    #[serde(alias = "BACKGROUND_COLOR")]
    background_color: Option<String>,
    #[serde(alias = "BACKGROUND_IMAGE")]
    background_image: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Stat {
    #[serde(alias = "INTERVAL")]
    interval: u32,
    #[serde(alias = "TEXT")]
    text: Text,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    display: Display,
    static_images: StaticImages,
    #[serde(alias = "STATS")]
    pub stats: HashMap<String, HashMap<String, Stat>>,
}

pub fn load(name: &str) -> Result<Theme, Box<dyn error::Error>> {
    let filepath = format!("res/themes/{}/theme.yaml", name);
    let theme: Theme = load_yaml(&filepath)?;

    // TODO: check theme compatibility

    Ok(theme)
}

fn load_yaml<T>(filename: &str) -> Result<T, Box<dyn error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let result: T = serde_yaml::from_reader(reader)?;
    Ok(result)
}
