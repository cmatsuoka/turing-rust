use std::collections::HashMap;
use std::error;
use std::fs::File;
use std::io::BufReader;

use bevy_reflect::Reflect;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct Display {
    display_orientation: String,
    display_rgb_led: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct Background {
    path: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct StaticImages {
    background: Background,
}

#[derive(Debug, Clone, Deserialize, Reflect)]
#[serde(rename_all = "UPPERCASE")]
struct Text {
    show: bool,
    show_unit: bool,
    x: u32,
    y: u32,
    font: String,
    font_size: u32,
    font_color: String,
    background_color: Option<String>,
    background_image: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Reflect)]
#[serde(rename_all = "UPPERCASE")]
struct Graph {
    show: bool,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    min_value: String,
    max_value: u32,
    bar_color: String,
    bar_outline: bool,
    background_color: Option<String>,
    background_image: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Reflect)]
#[serde(rename_all = "UPPERCASE")]
pub struct Percentage {
    interval: Option<u32>,
    text: Option<Text>,
    graph: Option<Graph>,
}

#[derive(Debug, Clone, Deserialize, Reflect)]
#[serde(rename_all = "UPPERCASE")]
pub struct Frequency {
    interval: Option<u32>,
    text: Option<Text>,
    graph: Option<Graph>,
}

#[derive(Debug, Clone, Deserialize, Reflect)]
#[serde(rename_all = "UPPERCASE")]
pub struct Temperature {
    interval: Option<u32>,
    text: Option<Text>,
    graph: Option<Graph>,
}

#[derive(Debug, Clone, Deserialize, Reflect)]
#[serde(rename_all = "UPPERCASE")]
pub struct DeviceMeter {
    interval: Option<u32>,
    text: Option<Text>,
    graph: Option<Graph>,
}

#[derive(Debug, Clone, Default, Deserialize, Reflect)]
#[serde(rename_all = "UPPERCASE")]
pub struct DeviceStats {
    pub interval: Option<u32>,
    pub percentage: Option<DeviceMeter>,
    pub frequency: Option<DeviceMeter>,
    pub temperature: Option<DeviceMeter>,
}

#[derive(Debug, Clone, Deserialize, Reflect)]
#[serde(rename_all = "UPPERCASE")]
pub struct Stats {
    pub interval: Option<u32>,
    pub cpu: Option<DeviceStats>,
    pub gpu: Option<DeviceStats>,
    //pub memory: Option<MemoryStat>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    display: Display,
    static_images: StaticImages,
    #[serde(alias = "STATS")]
    pub stats: Stats,
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
