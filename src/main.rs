use crate::topo_settings::{TopoSettings};
use fltk::app::Sender;
use image_crate::imageops::FilterType;
use map_range::MapRange;
use fltk::image::{Image, RgbImage, SharedImage};
use fltk::{prelude::*, *};
use image_crate::{ImageBuffer, Luma, Pixel, DynamicImage, EncodableLayout, Rgb, Rgba};
use noise::utils::{ImageRenderer, NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use noise::{Billow, Curve, Fbm, MultiFractal, Perlin, Seedable, Simplex};
use rand::{thread_rng, Rng};

use std::sync::Arc;
use fltk::window::{GlutWindow};
use three_d::{AmbientLight, Camera, ClearState, ColorMaterial, Context, CpuMaterial, CpuMesh, CpuTexture, Cull, DirectionalLight, FromCpuMaterial, Gm, LightingModel, Mat4, Mesh, PhysicalMaterial, RenderTarget, Srgba, Terrain, Vec3, Viewport};
use std::default::Default;
use std::fs::File;
use std::{fs, io};
use std::fmt::format;
use std::io::BufReader;
use std::mem::{replace, swap};
use std::path::PathBuf;

use colorgrad::Color;
use fltk::dialog::{FileDialog, FileDialogOptions, FileDialogType};
use fltk::enums::{ColorDepth, Shortcut};
use ron::de::from_reader;
use serde::{Deserialize, Serialize};


use crate::erosion::world::{Vec2, World};
use topo_settings::NoiseTypesUi;
use crate::utils::{get_height, get_raw_u16, get_raw_u8};
use crate::weather::{Climate, GenData, GridComponent, HumidDry, koppen_afam, koppen_as, koppen_aw, koppen_bsh, koppen_bsk, koppen_bwh, koppen_bwk, koppen_cfa, koppen_cfb, koppen_cfc, koppen_cwa, koppen_cwb, koppen_cwc, koppen_dfa, koppen_dfb, koppen_dfc, koppen_dsc, koppen_et};
use crate::weather_settings::WeatherSettings;
use crate::WeatherVisualization::Init;

mod erosion;
mod topo_settings;
mod ui;
mod utils;
mod weather;
mod weather_settings;

const DIMENSIONS: usize = 512;
const PREV_DIMENSIONS: usize = 512;

#[derive(Clone, Serialize, Deserialize, Debug)]
struct FileData {
    topography: TopoSettings,
    weather: WeatherSettings,
    raw_map_512: Vec<u16>,
    color_map_512: Vec<u8>,
    eroded_raw_512: Vec<u16>,
    color_eroded_512: Vec<u8>,
    raw_full: Vec<u16>,
    eroded_full_color: Vec<u8>,
    eroded_full: Vec<u16>,
    weather_data: Vec<GenData>,
    discharge: Vec<u8>
}

#[derive(Copy, Clone)]
enum WeatherVisualization {
    Wind,
    Temperature,
    Pressure,
    Humidity,
    Init
}
#[derive(Copy, Clone)]
enum Message {
    PerlinChoice,
    BillowChoice,
    SimplexChoice,
    SeedInput,
    SeedRandom,
    OctaveInput,
    LacInput,
    FreqInput,
    MtnSlider,
    SeaSlider,
    CycleInput,
    ErodeButton,
    FullPreview,
    TurnViewRight,
    TurnViewLeft,
    GenWeather,
    WeatherSeedInput,
    WeatherClimInput,
    WeatherLatInput,
    WeatherGridSize,
    WeatherRandomSeed,
    ViewTemperature,
    ViewHumidity,
    ViewPressure,
    ViewWind,
    DaySlider,
    Layer,
    MinHeightInput,
    MaxHeightInput,
    OpenFile,
    SaveFile,
    SaveFileAs,
    NewScenario,
}

struct ViewState {
    mode: WeatherVisualization,
    hour: u32,
    layer: u8
}

fn menu_do(w: &mut impl MenuExt, sender: &Sender<Message>) {
    w.add_emit(
        "&File/New Scenario...\t",
        Shortcut::Ctrl | 'n',
        menu::MenuFlag::Normal,
        *sender,
        Message::NewScenario
    );
    w.add_emit(
        "&File/Open Scenario...\t",
        Shortcut::Ctrl | 'o',
        menu::MenuFlag::Normal,
        *sender,
        Message::OpenFile
    );
    w.add_emit(
        "&File/Save Scenario\t",
        Shortcut::Ctrl | 's',
        menu::MenuFlag::Normal,
        *sender,
        Message::SaveFile
    );
    w.add_emit(
        "&File/Save Scenario as...\t",
        Shortcut::Ctrl | 'a',
        menu::MenuFlag::Normal,
        *sender,
        Message::SaveFileAs
    );
}

fn new_do(program_data: &mut FileData) {
    let mut clean = FileData {
        topography: TopoSettings {
            seed: None,
            noise_type: None,
            noise_octaves: None,
            noise_frequency: None,
            noise_lacunarity: None,
            mountain_pct: 0.0,
            sea_pct: 0.0,
            min_height: 0,
            max_height: 0,
            erosion_cycles: 0,
        },
        weather: WeatherSettings {
            seed: None,
            koppen: None,
            latitude: 0,
            grid_size: 0,
        },
        raw_map_512: vec![],
        color_map_512: vec![],
        eroded_raw_512: vec![],
        color_eroded_512: vec![],
        raw_full: vec![],
        eroded_full_color: vec![],
        eroded_full: vec![],
        weather_data: vec![],
        discharge: vec![],
    };
    clean.raw_map_512.fill(0);
    clean.color_map_512.fill(0);
    clean.eroded_raw_512.fill(0);
    clean.color_eroded_512.fill(0);
    clean.raw_full.fill(0);
    clean.eroded_full_color.fill(0);
    clean.eroded_full.fill(0);
    clean.discharge.fill(0);
        
    let _ = replace::<FileData>(program_data, clean);
}

fn set_data(loaded_data: &mut FileData, data: &mut FileData) {
    let _ = swap::<FileData>(data, &mut loaded_data.clone());
}

// TODO poner nombre del escenario en window
fn save_file_do(program_data: &mut FileData, is_workplace: &mut bool, path: &mut String, filename: &mut String) {
    if *is_workplace {
        let s = ron::ser::to_string(&program_data).expect("Error serializing file data.");
        fs::write(path, s).expect("Unable to write file.");
    } else {
        let mut nfc = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseSaveFile);
        nfc.set_filter("RON files\t*.ron");
        nfc.set_option(dialog::FileDialogOptions::SaveAsConfirm);
        nfc.show();
        let dir = nfc.filename();
        let final_name = dir.to_str().unwrap().to_string() + ".ron";
        if !dir.clone().to_str().unwrap().is_empty() {
            let s = ron::ser::to_string(&program_data).expect("Error serializing file data.");
            fs::write(final_name.as_str(), s).expect("Unable to write file.");
            let _ = replace::<bool>(is_workplace, true);
            let p = final_name.clone();
            let _ = replace::<String>(path, p);
        }
    }
}
fn open_file_do(program_data: &mut FileData) -> (FileData, PathBuf) {
    let mut data: FileData = FileData {
        topography: TopoSettings {
            seed: None,
            noise_type: None,
            noise_octaves: None,
            noise_frequency: None,
            noise_lacunarity: None,
            mountain_pct: 0.0,
            sea_pct: 0.0,
            min_height: 0,
            max_height: 0,
            erosion_cycles: 0,
        },
        weather: WeatherSettings {
            seed: None,
            koppen: None,
            latitude: 0,
            grid_size: 0,
        },
        raw_map_512: vec![],
        color_map_512: vec![],
        eroded_raw_512: vec![],
        color_eroded_512: vec![],
        raw_full: vec![],
        eroded_full_color: vec![],
        eroded_full: vec![],
        weather_data: vec![],
        discharge: vec![],
    };
    let mut nfc = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseFile);
    nfc.set_filter("RON files\t*.ron");
    nfc.show();
    let dir = nfc.filename();
    if !dir.clone().into_os_string().is_empty() {
        let f = File::open(dir.clone()).expect("Error opening file.");
        let mut data_n: FileData = match from_reader(f) {
            Ok(x) => x,
            Err(e) => {
                println!("Failed to load file: {}", e);
                std::process::exit(1);
            }
        };
        data = data_n;
    }
    (data, dir)
}

fn set_hour(w: &mut impl ValuatorExt, state: &mut ViewState) {
    state.hour = w.value() as u32;
}

/// Outputs previous state for comparison
fn set_view_state(options: &mut ViewState, state: WeatherVisualization) -> WeatherVisualization {
    let previous_state = options.mode;
    options.mode = state;
    previous_state
}
fn update_grid_at_time(hour: u32, grid_vector: &mut Vec<GenData>, cube_vector: &mut [Gm<Mesh, ColorMaterial>], state: &ViewState) {
    use ordered_float::OrderedFloat;
    for component in grid_vector.as_slice() {
        let cube = &mut cube_vector[component.index.0 as usize + 16 *(component.index.1 as usize + 6 * component.index.2 as usize)];
        let range = match hour {
            0 => 0..24,
            _ => (((24 * hour) - 24) as usize)..((24 * hour) as usize)
        };
        match state.mode {
            Init => {},
            WeatherVisualization::Wind => {},
            WeatherVisualization::Temperature => {
                if !component.pressure.is_empty() {
                    let median = (component.temperature[range.clone()].iter().sum::<OrderedFloat<f64>>().0 as isize) / range.clone().len() as isize;
                    let color = match median {
                        -60..=-10 => Color::from_rgba8(30, 92, 179, 10),
                        -11..=-1 => Color::from_rgba8(4, 161, 230, 10),
                        0..=5 => Color::from_rgba8(102, 204, 206, 10),
                        6..=10 => Color::from_rgba8(192, 229, 136, 10),
                        11..=15 => Color::from_rgba8(204, 230, 75, 10),
                        16..=20 => Color::from_rgba8(243, 240, 29, 10),
                        21..=25 => Color::from_rgba8(248, 157, 14, 10),
                        26..=30 => Color::from_rgba8(219, 30, 38, 10),
                        31..=90 => Color::from_rgba8(164, 38, 44, 10),
                        _ => Color::from_rgba8(255, 255, 255, 10)
                    };
                    let color_rgba = color.to_linear_rgba_u8();
                    let mut opacity = 10;

                    match state.layer {
                        0 => {opacity = 10},
                        1 => {if component.index.1 != 0 {opacity = 0}},
                        2 => {if component.index.1 != 1 {opacity = 0}},
                        3 => {if component.index.1 != 2 {opacity = 0}},
                        4 => {if component.index.1 != 3 {opacity = 0}},
                        5 => {if component.index.1 != 4 {opacity = 0}},
                        6 => {if component.index.1 != 5 {opacity = 0}},
                        _ => { opacity = 10 }
                    }
                    if color.r == 1.0 && color.g == 1.0 && color.b == 1.0 {
                        opacity = 0;
                    }
                    cube.material.color = Srgba::new(color_rgba.0, color_rgba.1, color_rgba.2, opacity);
                }
            },
            WeatherVisualization::Pressure => {
                if !component.pressure.is_empty() {
                    let median = (component.pressure[range.clone()].iter().sum::<OrderedFloat<f64>>().0 as usize) / range.clone().len();
                    let color = match median {
                        50..=950 => Color::from_rgba8(40, 40, 255, 10) ,
                        951..=990 => Color::from_rgba8(102, 102, 255, 10),
                        991..=1000 => Color::from_rgba8(161, 161, 255, 10),
                        1001..=1015 => Color::from_rgba8(203, 203, 255, 10),
                        1016..=1030 => Color::from_rgba8(255, 138, 138, 10),
                        1031..=1060 => Color::from_rgba8(255, 103, 103, 10),
                        1061..=2000 => Color::from_rgba8(255, 41, 41, 10),
                        _ => Color::from_rgba8(255, 255, 255, 10)
                    };
                    let color_rgba = color.to_linear_rgba_u8();
                    let mut opacity = 10;
                    match state.layer {
                        0 => {opacity = 10},
                        1 => {if component.index.1 != 0 {opacity = 0}},
                        2 => {if component.index.1 != 1 {opacity = 0}},
                        3 => {if component.index.1 != 2 {opacity = 0}},
                        4 => {if component.index.1 != 3 {opacity = 0}},
                        5 => {if component.index.1 != 4 {opacity = 0}},
                        6 => {if component.index.1 != 5 {opacity = 0}},
                        _ => { opacity = 10 }
                    }
                    cube.material.color = Srgba::new(color_rgba.0, color_rgba.1, color_rgba.2, opacity);
                }
            },
            WeatherVisualization::Humidity => {
                let median = (component.humidity[range.clone()].iter().sum::<OrderedFloat<f64>>().0 as usize) / range.clone().len();
                let color = match median {
                    0..=10 => Color::from_rgba8(255, 255, 217, 10),
                    11..=20 => Color::from_rgba8(237, 248, 177, 10),
                    21..=30 => Color::from_rgba8(199, 233, 180, 10),
                    31..=40 => Color::from_rgba8(127, 205, 187, 10),
                    41..=50 => Color::from_rgba8(65, 182, 196, 10),
                    51..=60 => Color::from_rgba8(29, 145, 192, 10),
                    61..=70 => Color::from_rgba8(34, 94, 168, 10),
                    71..=80 => Color::from_rgba8(37, 52, 148, 10),
                    81..=90 => Color::from_rgba8(8, 29, 88, 10),
                    91..=100 => Color::from_rgba8(3, 20, 70, 10),
                    _ => Color::from_rgba8(255, 255, 255, 10)
                };
                let color_rgba = color.to_linear_rgba_u8();
                let mut opacity = 10;

                match state.layer {
                    0 => {opacity = 10},
                    1 => {if component.index.1 != 0 {opacity = 0}},
                    2 => {if component.index.1 != 1 {opacity = 0}},
                    3 => {if component.index.1 != 2 {opacity = 0}},
                    4 => {if component.index.1 != 3 {opacity = 0}},
                    5 => {if component.index.1 != 4 {opacity = 0}},
                    6 => {if component.index.1 != 5 {opacity = 0}},
                    _ => { opacity = 10 }
                }
                if color.r == 1.0 && color.g == 1.0 && color.b == 1.0 {
                    opacity = 0;
                }
                cube.material.color = Srgba::new(color_rgba.0, color_rgba.1, color_rgba.2, opacity);
            }
        }
    }
}

fn hydro_preview_do(data: &mut FileData) {
    match data.topography.noise_type.unwrap() {
        NoiseTypesUi::Perlin => update_perlin_noise(data, PREV_DIMENSIONS),
        NoiseTypesUi::Simplex => update_simplex_noise(PREV_DIMENSIONS, data),
        NoiseTypesUi::BillowPerlin => update_billow_noise(PREV_DIMENSIONS, data),
    }
}

fn erode_terrain_preview(file: &mut FileData) {
    let b: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::from_raw(512, 512, file.clone().raw_map_512).unwrap();
    file.raw_map_512 = b.clone().into_raw();
    let (width, height) = b.dimensions();
    let heightmap = b.into_raw();
    let mut erosion_world = World::new(
        heightmap,
        width as usize,
        height as usize,
        file.topography.seed.unwrap() as i16,
    );

    eprintln!("Eroding preview.");
    eprintln!(
        "Total cycle count: {}",
        file.topography.erosion_cycles as i32
    );

    for cycle in 0..(file.topography.erosion_cycles as i32) {
        erosion_world.erode(width as usize, 1.0);
        if cycle == 0 {
            eprintln!("0")
        } else {
            eprint!("..{}", cycle)
        }
        // update_text_buffer(w, cycle);
    }
    let eroded_preview: Vec<u16> = erosion_world
        .map
        .heightmap
        .iter()
        .map(|x| (x.height * 255.0) as u16)
        .collect();
    let buffer: ImageBuffer<Luma<u16>, Vec<u16>> =
        ImageBuffer::from_raw(width, height, eroded_preview).unwrap();
    file.eroded_raw_512 = buffer.clone().into_raw();
    apply_color_eroded(file, 512);
}

fn erode_heightmap_full(file: &mut FileData) {

    let mut discharge_map = vec![0; 8192 * 8192];

    let img: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::from_raw(512, 512, file.clone().raw_map_512).unwrap();

    let heightmap = img.into_raw();
    let mut erosion_world_1 = World::new(
        heightmap,
        512,
        512,
        file.topography.seed.unwrap() as i16,
    );
    
    //////////////////////
    //    SIZE: 512     //
    //////////////////////

    eprint!("Eroding full preview.");
    eprintln!("Size: 512");
    eprintln!(
        "Total cycle count: {}",
        file.topography.erosion_cycles as i32
    );

    for cycle in 0..(file.topography.erosion_cycles as i32) {
        erosion_world_1.erode(512, 1.0);
        if cycle == 0 {
            eprintln!("0")
        } else {
            eprint!("..{}", cycle)
        }
    }
    
    //////////////////////
    //   SIZE: 1024     //
    //////////////////////
    
    let eroded_preview_to_be_1024: Vec<u16> = erosion_world_1
        .map
        .heightmap
        .iter()
        .map(|x| (x.height * 255.0) as u16)
        .collect();
        
    let buffer: ImageBuffer<Luma<u16>, Vec<u16>> =
        ImageBuffer::from_raw(512, 512, eroded_preview_to_be_1024).unwrap();
    
    let resampled_vec_1024 = image_crate::imageops::resize(&buffer, 1024, 1024, FilterType::Lanczos3).into_raw();
    
    let mut erosion_world_2 = World::new(
        resampled_vec_1024,
        1024,
        1024,
        file.topography.seed.unwrap() as i16,
    );
    
    eprintln!("Size: 1024");
    eprintln!(
        "Total cycle count: {}",
        file.topography.erosion_cycles as i32
    );
    
    for cycle in 0..(file.topography.erosion_cycles as i32) {
        erosion_world_2.erode(1024, 1.0);
        if cycle == 0 {
            eprintln!("0")
        } else {
            eprint!("..{}", cycle)
        }
    }
    
    //////////////////////
    //   SIZE: 2048     //
    //////////////////////
    
    let eroded_preview_to_be_2048: Vec<u16> = erosion_world_2
        .map
        .heightmap
        .iter()
        .map(|x| (x.height * 255.0) as u16)
        .collect();
    
    let buffer: ImageBuffer<Luma<u16>, Vec<u16>> =
        ImageBuffer::from_raw(1024, 1024, eroded_preview_to_be_2048).unwrap();
    
    let resampled_vec_2048 = image_crate::imageops::resize(&buffer, 2048, 2048, FilterType::Lanczos3).into_raw();
    
    let mut erosion_world_3 = World::new(
        resampled_vec_2048,
        2048,
        2048,
        file.topography.seed.unwrap() as i16,
    );
    
    eprintln!("Size: 2048");
    eprintln!(
        "Total cycle count: {}",
        file.topography.erosion_cycles as i32
    );
    
    for cycle in 0..(file.topography.erosion_cycles as i32) {
        erosion_world_3.erode(2048, 1.0);
        if cycle == 0 {
            eprintln!("0")
        } else {
            eprint!("..{}", cycle)
        }
        // update_text_buffer(w, cycle);
    }
    
    //////////////////////
    //   SIZE: 4096     //
    //////////////////////
    
    let eroded_preview_to_be_4096: Vec<u16> = erosion_world_3
        .map
        .heightmap
        .iter()
        .map(|x| (x.height * 255.0) as u16)
        .collect();
    
    let buffer: ImageBuffer<Luma<u16>, Vec<u16>> =
        ImageBuffer::from_raw(2048, 2048, eroded_preview_to_be_4096).unwrap();
    
    let resampled_vec_4096 = image_crate::imageops::resize(&buffer, 4096, 4096, FilterType::Lanczos3).into_raw();
    
    let mut erosion_world_4 = World::new(
        resampled_vec_4096,
        4096,
        4096,
        file.topography.seed.unwrap() as i16,
    );
    
    eprintln!("Size: 4096");
    eprintln!(
        "Total cycle count: {}",
        file.topography.erosion_cycles as i32
    );
    
    for cycle in 0..(file.topography.erosion_cycles as i32) {
        erosion_world_4.erode(4096, 1.0);
        if cycle == 0 {
            eprintln!("0")
        } else {
            eprint!("..{}", cycle)
        }
        // update_text_buffer(w, cycle);
    }
    
    //////////////////////
    //   SIZE: 8192     //
    //////////////////////
    
    let eroded_preview_to_be_8192: Vec<u16> = erosion_world_4
        .map
        .heightmap
        .iter()
        .map(|x| (x.height * 255.0) as u16)
        .collect();
    
    let buffer: ImageBuffer<Luma<u16>, Vec<u16>> =
        ImageBuffer::from_raw(4096, 4096, eroded_preview_to_be_8192).unwrap();
    
    let resampled_vec_8192 = image_crate::imageops::resize(&buffer, 8192, 8192, FilterType::Lanczos3).into_raw();
    
    let mut erosion_world_5 = World::new(
        resampled_vec_8192,
        8192,
        8192,
        file.topography.seed.unwrap() as i16,
    );
    
    eprintln!("Size: 8192");
    eprintln!(
        "Total cycle count: {}",
        file.topography.erosion_cycles as i32
    );
    
    for cycle in 0..(file.topography.erosion_cycles as i32) {
        erosion_world_5.erode(8192, 1.0);
        if cycle == 0 {
            eprintln!("0")
        } else {
            eprint!("..{}", cycle)
        }
        // update_text_buffer(w, cycle);
    }
    
    /////////////////
    
    for i in 0..discharge_map.len() {
        let pos = Vec2::new(i as f64 % 8192.0, (i / 8192) as f64);
        discharge_map[i] = ((erosion_world_5.map.discharge(pos) + 1.0) * 0.5 * 255.0) as u8;
    }
    let eroded_preview: Vec<u16> = erosion_world_5
        .map
        .heightmap
        .iter()
        .map(|x| (x.height * 255.0) as u16)
        .collect();
    let buffer: ImageBuffer<Luma<u16>, Vec<u16>> =
        ImageBuffer::from_raw(8192, 8192, eroded_preview.clone()).unwrap();

    file.eroded_full = eroded_preview.clone();

    let mut discharge_buffer: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::from_raw(8192, 8192, discharge_map.clone()).unwrap();

    imageproc::contrast::stretch_contrast_mut(&mut discharge_buffer, 130, 200);

    file.discharge = discharge_buffer.clone().into_raw();

    // here was apply_color_eroded(file, 8192);
}

fn update_hydro_prev(w: &mut impl WidgetExt, left: bool, file: &mut FileData) {
    apply_color_eroded(file, 8192);
    w.set_image_scaled(None::<SharedImage>);

    match left {
        true => {
            let img = fltk::image::RgbImage::new(file.eroded_full_color.as_slice(), 8192, 8192, ColorDepth::Rgba8).unwrap();
            w.set_image_scaled(Some(img));
            w.redraw();
        }
        false => {
            let img = fltk::image::RgbImage::new(file.discharge.as_slice(), 8192, 8192, ColorDepth::L8).unwrap();
            w.set_image_scaled(Some(img));
            w.redraw();
        }
    };
}

fn apply_variations_perlin(source: Fbm<Perlin>, mtn: f64, sea: f64) -> Curve<f64, Fbm<Perlin>, 2> {
    let sealevel = sea / 100.0;
    let mountainlevel = mtn / 100.0;

    let curved: Curve<f64, Fbm<Perlin>, 2> = Curve::new(source)
        .add_control_point(1.0, mountainlevel)
        .add_control_point(0.999, mountainlevel)
        .add_control_point(-0.999, 0.0 - sealevel)
        .add_control_point(-1.0, 0.0 - sealevel);

    curved
}

fn apply_variations_simplex(
    source: Fbm<Simplex>,
    mtn: f64,
    sea: f64,
) -> Curve<f64, Fbm<Simplex>, 2> {
    let sealevel = sea / 100.0;
    let mountainlevel = mtn / 100.0;

    let curved: Curve<f64, Fbm<Simplex>, 2> = Curve::new(source)
        .add_control_point(1.0, mountainlevel)
        .add_control_point(0.999, mountainlevel)
        .add_control_point(-0.999, 0.0 - sealevel)
        .add_control_point(-1.0, 0.0 - sealevel);

    curved
}

fn apply_variations_billow(
    source: Billow<Perlin>,
    mtn: f64,
    sea: f64,
) -> Curve<f64, Billow<Perlin>, 2> {
    let sealevel = sea / 100.0;
    let mountainlevel = mtn / 100.0;

    let curved: Curve<f64, Billow<Perlin>, 2> = Curve::new(source)
        .add_control_point(1.0, mountainlevel)
        .add_control_point(0.999, mountainlevel)
        .add_control_point(-0.999, 0.0 - sealevel)
        .add_control_point(-1.0, 0.0 - sealevel);

    curved
}

fn apply_color_eroded(file: &mut FileData, size: u16) {
    
    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let data = match size {
        512 => file.eroded_raw_512.clone(),
        8192 => file.eroded_full.clone(),
        _ => vec![]
    };
    let eroded_image: ImageBuffer<Luma<u16>, Vec<u16>> = image_crate::ImageBuffer::from_raw(size as u32, size as u32, data).unwrap();
    let mut map = NoiseMap::new(size as usize, size as usize);

    for x in 0..size {
        for y in 0..size {
            let pixel = eroded_image
                .get_pixel(x as u32, y as u32)
                .channels()
                .first()
                .unwrap();
            let p_i = *pixel as f32;
            let output = p_i.map_range(0.0..32767.0, -1.0..1.0);
            map.set_value(x as usize, y as usize, output as f64);
        }
    }

    let mut r = ImageRenderer::new()
        .set_gradient(gradient)
        .set_light_elevation(Default::default())
        .set_light_color(Default::default())
        .set_light_azimuth(Default::default())
        .set_light_brightness(Default::default())
        .set_light_contrast(Default::default())
        .set_light_intensity(Default::default());
    r.disable_light();
    let b = r.render(&map);
    let i = get_raw_u8(&b);

    match size {
        512 => file.color_eroded_512 = i.to_vec(),
        8192 => file.eroded_full_color = i.to_vec(),
        _ => {}
    };

}

fn apply_color_hydro_full(file: &mut FileData) {
    
    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let b: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::from_raw(8192, 8192, file.clone().eroded_full).unwrap();

    let mut map = NoiseMap::new(8192, 8192);

    for x in 0..8192 {
        for y in 0..8192 {
            let pixel = b
                .get_pixel(x as u32, y as u32)
                .channels()
                .first()
                .unwrap();
            let p_i = *pixel as f32;
            let output = p_i.map_range(0.0..32767.0, -1.0..1.0);
            map.set_value(x, y, output as f64);
        }
    }
    let r = ImageRenderer::new().set_gradient(gradient).render(&map);

    let i = get_raw_u8(&r);

    file.eroded_full_color = i.to_vec();
}

fn update_perlin_noise(data: &mut FileData, dimensions: usize) {
    
    let mut perlin: Fbm<Perlin> = Default::default();
    perlin = perlin
        .set_seed(data.topography.seed.unwrap())
        .set_octaves(data.topography.noise_octaves.unwrap() as usize)
        .set_frequency(data.topography.noise_frequency.unwrap())
        .set_lacunarity(data.topography.noise_lacunarity.unwrap());

    let curved = apply_variations_perlin(perlin, data.topography.mountain_pct, data.topography.sea_pct);

    // .add_control_point(sealevel, 0.0)

    let map = PlaneMapBuilder::<Curve<f64, Fbm<Perlin>, 2>, 2>::new(curved)
        .set_size(dimensions, dimensions)
        .set_is_seamless(false)
        .set_x_bounds(-1.0, 1.0)
        .set_y_bounds(-1.0, 1.0)
        .build();

    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let renderer = ImageRenderer::new().set_gradient(gradient).render(&map);

    // for x in 0..DIMENSIONS {
    //     for y in 0..DIMENSIONS {
    //         println!("{}", map.get_value(x, y));
    //     }
    // }

    let a = get_raw_u8(&renderer);
    let b = get_raw_u16(&map);

    data.raw_map_512 = b.to_vec();
    data.color_map_512 = a.to_vec();

}

fn update_simplex_noise(dimensions: usize, data: &mut FileData) {
    
    let mut simplex: Fbm<Simplex> = Default::default();
    simplex = simplex
        .set_seed(data.topography.seed.unwrap())
        .set_octaves(data.topography.noise_octaves.unwrap() as usize)
        .set_frequency(data.topography.noise_frequency.unwrap())
        .set_lacunarity(data.topography.noise_lacunarity.unwrap());

    let curved = apply_variations_simplex(simplex, data.topography.mountain_pct, data.topography.sea_pct);

    // .add_control_point(sealevel, 0.0)
    let map = PlaneMapBuilder::<Curve<f64, Fbm<Simplex>, 2>, 2>::new(curved)
        .set_size(dimensions, dimensions)
        .set_is_seamless(false)
        .set_x_bounds(-1.0, 1.0)
        .set_y_bounds(-1.0, 1.0)
        .build();

    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let renderer = ImageRenderer::new().set_gradient(gradient).render(&map);

    data.raw_map_512 = get_raw_u16(&map).to_vec();
    data.color_map_512 = get_raw_u8(&renderer).to_vec();

}

/// 0 => preview_box_topo (512 not eroded)
/// 1 => preview_erosion_topo (512 eroded)
/// 2 => hydro_preview (8192 eroded)
/// 3 => hydro_mask_preview (8192 water mask)
fn update_noise_img(w: &mut impl WidgetExt, data: &FileData, img_type: u8) {
    let map = match img_type {
        0 => { image::RgbImage::new(data.color_map_512.as_slice(), 512, 512, ColorDepth::Rgba8).unwrap() },
        1 => image::RgbImage::new(data.color_eroded_512.as_slice(), 512, 512, ColorDepth::Rgba8).unwrap(),
        _ => RgbImage::new(&[], 512, 512, ColorDepth::Rgba8).unwrap()
    };
    w.set_image_scaled(None::<SharedImage>);
    let img = SharedImage::from_image(map).unwrap();
    w.set_image_scaled(Some(img));
    w.redraw();
}

fn update_billow_noise(dimensions: usize, data: &mut FileData) {
    
    let mut perlin: Billow<Perlin> = Default::default();
    perlin = perlin
        .set_seed(data.topography.seed.unwrap())
        .set_octaves(data.topography.noise_octaves.unwrap() as usize)
        .set_frequency(data.topography.noise_frequency.unwrap())
        .set_lacunarity(data.topography.noise_lacunarity.unwrap());

    let curved = apply_variations_billow(perlin, data.topography.mountain_pct, data.topography.sea_pct);

    // .add_control_point(sealevel, 0.0)
    let map = PlaneMapBuilder::<Curve<f64, Billow<Perlin>, 2>, 2>::new(curved)
        .set_size(dimensions, dimensions)
        .set_is_seamless(false)
        .set_x_bounds(-1.0, 1.0)
        .set_y_bounds(-1.0, 1.0)
        .build();

    let gradient = noise::utils::ColorGradient::new().build_terrain_gradient();
    let renderer = ImageRenderer::new().set_gradient(gradient).render(&map);

    let a = get_raw_u8(&renderer);
    let b = get_raw_u16(&map);

    data.raw_map_512 = b.to_vec();
    data.color_map_512 = a.to_vec();

}

fn weather_seed_do(w: &mut impl InputExt, data: &mut FileData) {
    if w.changed() {
        data.weather.set_seed(w.value().parse().unwrap());
    }
}

fn weather_lat_do(w: &mut impl InputExt, data: &mut FileData) {
    if w.changed() {
        data.weather.set_latitude(w.value().parse().unwrap());
    }
}

fn weather_grid_size_do(w: &mut impl InputExt, data: &mut FileData) {
    if w.changed() {
        data.weather.set_grid_size(w.value().parse().unwrap());
    }
}

fn weather_climate_do(w: &mut impl MenuExt, data: &mut FileData, climates: &[Climate; 18]) {

        let choice_name = w.choice().unwrap();
        let mut climate_choice: Climate = Climate {
            name: "Blank".to_string(),
            general_type: 'x',
            second_type: 'x',
            third_type: 'x',
            spring: (HumidDry::Humid, Default::default()),
            winter: (HumidDry::Humid, Default::default()),
            fall: (HumidDry::Humid, Default::default()),
            summer: (HumidDry::Humid, Default::default()),
            diurnal_range: Default::default(),
        };
        for climate in climates {
            if choice_name.as_str() == climate.name.as_str() {
                climate_choice = climate.clone();
                break
            } else {
                continue
            }
        }
    data.weather.set_climate(climate_choice);

}

fn seed_input_do(w: &mut impl InputExt, data: &mut FileData) {
    if w.changed() {
        data.topography.set_seed(Some(w.value().parse().unwrap()));

        match data.topography.noise_type {
            Some(NoiseTypesUi::Simplex) => {
                update_simplex_noise(DIMENSIONS, data);
            }
            Some(NoiseTypesUi::Perlin) => {
                update_perlin_noise(data, DIMENSIONS);
            }
            Some(NoiseTypesUi::BillowPerlin) => {
                update_billow_noise(DIMENSIONS, data);
            }
            _ => {}
        };
    }
}

fn weather_seed_random_do(
    _w: &mut impl ButtonExt,
    seed_box: &mut impl InputExt,
    data: &mut FileData,
) {
    let mut rng = thread_rng();
    let seed: u32 = rng.gen_range(u32::MIN..u32::MAX);
    seed_box.set_value(&format!("{}", seed));
    data.weather.seed = Some(seed);
}

fn seed_random_do(
    _w: &mut impl ButtonExt,
    data: &mut FileData,
    seed_box: &mut impl InputExt,
) {
    let mut rng = thread_rng();
    let seed: u32 = rng.gen_range(u32::MIN..u32::MAX);
    seed_box.set_value(&format!("{}", seed));
    data.topography.set_seed(Some(seed));

    match data.topography.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(DIMENSIONS, data);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(data, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(DIMENSIONS, data);
        }
        _ => {}
    };
}

fn aux_choice_do(data: &mut FileData) {
    match data.topography.noise_type.unwrap() {
        NoiseTypesUi::Perlin => {
            data.topography.set_type(Some(NoiseTypesUi::Perlin));
            update_perlin_noise(data, DIMENSIONS);
        }
        NoiseTypesUi::Simplex => {
            data.topography.set_type(Some(NoiseTypesUi::Simplex));
            update_simplex_noise(DIMENSIONS, data);
        }
        NoiseTypesUi::BillowPerlin => {
            data.topography.set_type(Some(NoiseTypesUi::BillowPerlin));
            update_billow_noise(DIMENSIONS, data);
        }
    }
}

fn noise_choice_do(w: &mut impl MenuExt, sender: &Sender<Message>) {
    w.add_emit(
        "Simplex",
        enums::Shortcut::None,
        menu::MenuFlag::Normal,
        *sender,
        Message::SimplexChoice,
    );
    w.add_emit(
        "Perlin",
        enums::Shortcut::None,
        menu::MenuFlag::Normal,
        *sender,
        Message::PerlinChoice,
    );
    w.add_emit(
        "Billowed Perlin",
        enums::Shortcut::None,
        menu::MenuFlag::Normal,
        *sender,
        Message::BillowChoice,
    );
}

fn change_noise_type(noise_types_ui: NoiseTypesUi, data: &mut FileData) {
    data.topography.set_type(Some(noise_types_ui));
}

fn octaves_input_do(w: &mut impl InputExt, data: &mut FileData) {
    data.topography.set_octaves(Some(w.value().parse::<u32>().unwrap()));

    match data.topography.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(DIMENSIONS, data);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(data, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(DIMENSIONS, data);
        }
        _ => {}
    };
}

fn frequency_input_do(w: &mut impl InputExt, data: &mut FileData) {
    println!("Has input changed? {}", w.changed());
    data.topography.set_frequency(Some(w.value().parse::<f64>().unwrap()));

    match data.topography.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(DIMENSIONS, data);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(data, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(DIMENSIONS, data);
        }
        _ => {}
    };
}

fn lacunarity_input_do(w: &mut impl InputExt, data: &mut FileData) {
    data.topography.set_lacunarity(Some(w.value().parse().unwrap()));

    match data.topography.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(DIMENSIONS, data);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(data, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(DIMENSIONS, data);
        }
        _ => {}
    };
}

fn mtn_slider_do(w: &mut impl ValuatorExt, data: &mut FileData) {
    data.topography.set_mtn_pct(w.value());
    match data.topography.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(DIMENSIONS, data);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(data, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(DIMENSIONS, data);
        }
        _ => {}
    };
}
fn sea_slider_do(w: &mut impl ValuatorExt, data: &mut FileData) {
    data.topography.set_sea_pct(w.value());
    match data.topography.noise_type {
        Some(NoiseTypesUi::Simplex) => {
            update_simplex_noise(DIMENSIONS, data);
        }
        Some(NoiseTypesUi::Perlin) => {
            update_perlin_noise(data, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            update_billow_noise(DIMENSIONS, data);
        }
        _ => {}
    };
}

fn cycle_input_do(w: &mut impl ValuatorExt, data: &mut FileData) {
    data.topography.set_cycles(w.value() as u64);
}


fn main() {

    let (mut is_file_workspace, mut workspace_path, mut file_name) = (false, "".to_string(), "".to_string());

    let mut view_state = ViewState {
        mode: Init,
        hour: 0,
        layer: 0
    };

    let climates: [Climate; 18] = [koppen_cfa(), koppen_cfb(), koppen_cfc(), koppen_dfb(), koppen_dfc(), koppen_dfa(), koppen_cwc(), koppen_cwb(), koppen_cwa(), koppen_et(), koppen_afam(), koppen_as(), koppen_aw(), koppen_dsc(), koppen_bsh(), koppen_bsk(), koppen_bwh(), koppen_bwk()];

    let mut grid: Vec<GenData> = vec![];

    let mut topo_settings: TopoSettings = TopoSettings {
        seed: Some(42949),
        noise_type: Some(NoiseTypesUi::BillowPerlin),
        noise_octaves: Some(20),
        noise_frequency: Some(3.0),
        noise_lacunarity: Some(4.0),
        mountain_pct: 25.0,
        sea_pct: 5.0,
        min_height: -50,
        max_height: 1000,
        erosion_cycles: 0,
    };

    let mut weather_settings: WeatherSettings = WeatherSettings {
        seed: None,
        koppen: None,
        latitude: 0,
        grid_size: 16,
    };

    let mut file = FileData {
        topography: topo_settings.clone(),
        weather: weather_settings.clone(),
        raw_map_512: vec![],
        color_map_512: vec![],
        eroded_raw_512: vec![],
        color_eroded_512: vec![],
        raw_full: vec![],
        eroded_full_color: vec![],
        eroded_full: vec![],
        weather_data: vec![],
        discharge: vec![]
    };

    // Initialize grid
    for x in 0..file.weather.clone().grid_size {
        for y in 0..6 {
            for z in 0..file.weather.clone().grid_size {
                let _h = 0;
                let component = GenData {
                    index: (x as i32, y, z as i32),
                    temperature: vec![],
                    altitude: 0.0,
                    pressure: vec![],
                    humidity: vec![],
                    wind: vec![],
                    td: vec![],
                };
                grid.push(component);
            }
        }
    }

    let (s, r) = app::channel::<Message>();

    let app = app::App::default();
    let mut ui = ui::UserInterface::make_window();
    let mut win = ui.main_window.clone();
    let (x, y, w, h) = (ui.weather_preview.x(), ui.weather_preview.y(), ui.weather_preview.w(), ui.weather_preview.h());
    // create gl window
    let mut gl_widget = GlutWindow::new(x, y, w, h, None);
    // let mut gl_win = GlutWindow::new(x, y, w, h, None);
    gl_widget.set_mode(enums::Mode::Opengl3);
    ui.weather_preview.add(&gl_widget);
    win.end();
    gl_widget.end();
    win.show();
    gl_widget.show();

    ui.turn_right_vis.set_image(Some(SharedImage::load("icons/turn_right.png").unwrap()));
    ui.turn_left_vis.set_image(Some(SharedImage::load("icons/turn_left.png").unwrap()));
    ui.wind_mode.set_image(Some(SharedImage::load("icons/wind.png").unwrap()));
    ui.temperature_mode.set_image(Some(SharedImage::load("icons/temperature.png").unwrap()));
    ui.humidity_mode.set_image(Some(SharedImage::load("icons/humidity.png").unwrap()));
    ui.pressure_mode.set_image(Some(SharedImage::load("icons/pressure.png").unwrap()));
    ui.legend_box.set_image(Some(SharedImage::load("icons/init_legend.png").unwrap()));

    /////////////

    let viewport = Viewport {
        x: (0-x) + 30, // don't know why tf it must be like this in order for the viewport to be aligned with the widget
        y: 0-y,
        width: w as u32,
        height: h as u32,
    };


    let gl = unsafe {
        three_d::context::Context::from_loader_function(|s| gl_widget.get_proc_address(s) as *const _)
    };

    // and this is three_d context
    let context = Context::from_gl_context(Arc::new(gl)).unwrap();

    context.set_cull(Cull::Back);

    let mut camera = Camera::new_orthographic(viewport,
                                              Vec3::new(0.0, 256.0, 0.0),
                                              Vec3::new(255.0, 128.0, 255.0),
                                              Vec3::new(0.0, 1.0, 0.0),
                                              768.0,
                                              0.1,
                                              10000.0
    );

    let m: CpuTexture = CpuTexture::default();

    let cpu_mat = CpuMaterial {
        name: "".to_string(),
        albedo: Default::default(),
        albedo_texture: Some(m),
        metallic: 0.0,
        roughness: 0.0,
        occlusion_metallic_roughness_texture: None,
        metallic_roughness_texture: None,
        occlusion_strength: 0.0,
        occlusion_texture: None,
        normal_scale: 0.0,
        normal_texture: None,
        emissive: Default::default(),
        emissive_texture: Default::default(),
        alpha_cutout: None,
        lighting_model: LightingModel::Phong,
        index_of_refraction: 0.0,
        transmission: 0.0,
        transmission_texture: None,
    };

    // three_d_asset::io::load(&["example_images/eroded_cache.png"]).unwrap().deserialize("").unwrap();

    let terrain_material = PhysicalMaterial::new_opaque(&context, &cpu_mat);

    let heightmap_opt_dyn = DynamicImage::new_luma16(512, 512);

    let heightmap_opt = heightmap_opt_dyn.to_luma16();

    let ambient = AmbientLight::new(&context, 0.4, Srgba::WHITE);
    let directional = DirectionalLight::new(&context, 0.4, Srgba::WHITE, &Vec3::new(0.0, -1.0, 100.0));
    let mut terrain = Terrain::new(
        &context,
        terrain_material.clone(),
        Arc::new(
            move |x, y| {
                *heightmap_opt.get_pixel(x as u32, y as u32).channels().first().unwrap() as f32 * 0.01
            }
        ),
        512.0,
        1.0,
        three_d::prelude::Vec2::new(255.0, 255.0)
    );

    let mut mesh_v: Vec<Gm<Mesh, ColorMaterial>> = vec![];

    //// RANDOMIZER ////

    let mut rng = thread_rng();

    ////           ////


    for x in 0..file.weather.clone().grid_size {
        for y in 0..6 {
            for z in 0..file.weather.clone().grid_size {
                let color: (u8, u8, u8) = (rng.gen_range(0..256) as u8, rng.gen_range(0..256) as u8, rng.gen_range(0..256) as u8);
                let mut cube = Gm::new(
                    Mesh::new(&context, &CpuMesh::cube()),
                    ColorMaterial::from_cpu_material(&context,
                        &CpuMaterial {
                            albedo: Srgba {
                                r: color.0,
                                g: color.1,
                                b: color.2,
                                a: 10,
                            },
                            ..Default::default()
                        },
                    )
                );
                cube.set_transformation(Mat4::from_translation(Vec3::new(32.0 * x as f32, 32.0 * (y + 6) as f32, 32.0 * z as f32)) * Mat4::from_scale(32.0));
                mesh_v.push(cube);
            }
        }
    }

    let mut frame = 0;

    /////////////

    for climate in &climates {
        ui.weather_type.add_choice(climate.name.as_str());
    }

    ui.wind_mode.emit(s, Message::ViewWind);

    ui.humidity_mode.emit(s, Message::ViewHumidity);

    ui.temperature_mode.emit(s, Message::ViewTemperature);

    ui.pressure_mode.emit(s, Message::ViewPressure);

    ui.day_vis_slider.emit(s, Message::DaySlider);

    ui.seed_random_button.emit(s, Message::SeedRandom);

    ui.seed_input.emit(s, Message::SeedInput);

    noise_choice_do(&mut ui.noise_choice, &s);

    menu_do(&mut ui.menu_bar, &s);

    ui.noise_octaves_input.emit(s, Message::OctaveInput);

    ui.noise_freq_input.emit(s, Message::FreqInput);

    ui.noise_lacunarity_input.emit(s, Message::LacInput);

    ui.high_elev_slider.emit(s, Message::MtnSlider);

    ui.sea_elev_slider.emit(s, Message::SeaSlider);

    ui.erosion_cycles_input.emit(s, Message::CycleInput);

    ui.erode_terrain_button.emit(s, Message::ErodeButton);

    ui.generate_hydro_prev.emit(s, Message::FullPreview);

    ui.turn_right_vis.emit(s, Message::TurnViewRight);

    ui.turn_left_vis.emit(s, Message::TurnViewLeft);

    ui.weather_seed_input.emit(s, Message::WeatherSeedInput);

    ui.weather_noise_random_seed.emit(s, Message::WeatherRandomSeed);

    ui.weather_type.emit(s, Message::WeatherClimInput);

    ui.latitude_input.emit(s, Message::WeatherLatInput);

    ui.grid_size_input.emit(s, Message::WeatherGridSize);

    ui.generate_weather_button.emit(s, Message::GenWeather);

    ui.min_height_input.emit(s, Message::MinHeightInput);

    ui.max_height_input.emit(s, Message::MaxHeightInput);

    ui.layer_slider.emit(s, Message::Layer);

    let target = *camera.target();

    let camera_y = camera.position().y;

    while app.wait() {
        if let Some(msg) = r.recv() {
            match msg {
                Message::MinHeightInput => { topo_settings.min_height = ui.min_height_input.value() as i32 }
                Message::MaxHeightInput => { topo_settings.max_height = ui.max_height_input.value() as i32 }
                Message::SimplexChoice => {
                    change_noise_type(NoiseTypesUi::Simplex, &mut file);
                    aux_choice_do(&mut file);
                    update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::PerlinChoice => {
                    change_noise_type(NoiseTypesUi::Perlin, &mut file);
                    aux_choice_do(&mut file);
                    update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::BillowChoice => {
                    change_noise_type(NoiseTypesUi::BillowPerlin, &mut file);
                    aux_choice_do(&mut file);
                    update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::SeedRandom => {
                    seed_random_do(
                        &mut ui.seed_random_button,
                        &mut file,
                        &mut ui.seed_input,
                    );
                    update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::SeedInput => {
                    seed_input_do(&mut ui.seed_input, &mut file);
                    update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::OctaveInput => {
                    octaves_input_do(&mut ui.noise_octaves_input, &mut file);
                    update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::FreqInput => {
                    frequency_input_do(&mut ui.noise_freq_input, &mut file);
                    update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::LacInput => {
                    lacunarity_input_do(&mut ui.noise_lacunarity_input, &mut file);
                    update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::MtnSlider => {
                    mtn_slider_do(&mut ui.high_elev_slider, &mut file);
                    update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::SeaSlider => {
                    sea_slider_do(&mut ui.sea_elev_slider, &mut file);
                    update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::CycleInput => {
                    cycle_input_do(&mut ui.erosion_cycles_input, &mut file)
                }
                Message::ErodeButton => {
                    erode_terrain_preview(&mut file);
                    update_noise_img(&mut ui.preview_erosion_topo, &mut file, 1);
                }
                Message::FullPreview => {
                    hydro_preview_do(&mut file);
                    erode_heightmap_full(&mut file);
                    update_hydro_prev(&mut ui.hydro_preview, true, &mut file);
                    update_hydro_prev(&mut ui.hydro_mask_preview, false, &mut file);
                }
                Message::TurnViewRight => {
                    camera.rotate_around_with_fixed_up(&target, 300.0, 0.0);
                    let camera_act_pos = Vec3::new(camera.position().x, camera_y, camera.position().z);
                    camera.set_view(camera_act_pos, target, Vec3::new(0.0, 1.0, 0.0));
                    // println!("pos: {:?}, target: {:?}, up: {:?}", camera.position(), camera.target(), camera.up());
                }
                Message::TurnViewLeft => {
                    camera.rotate_around_with_fixed_up(&target, -300.0, 0.0);
                    let camera_act_pos = Vec3::new(camera.position().x, camera_y, camera.position().z);
                    camera.set_view(camera_act_pos, target, Vec3::new(0.0, 1.0, 0.0));
                }
                Message::Layer => {
                    match ui.layer_slider.value() as u8 {
                        0 => view_state.layer = 0,
                        1 => view_state.layer = 1,
                        2 => view_state.layer = 2,
                        3 => view_state.layer = 3,
                        4 => view_state.layer = 4,
                        5 => view_state.layer = 5,
                        6 => view_state.layer = 6,
                        _ => view_state.layer = 0
                    }
                    update_grid_at_time(view_state.hour, &mut file.weather_data, &mut mesh_v, &view_state);
                }
                Message::WeatherSeedInput => {
                    weather_seed_do(&mut ui.weather_seed_input, &mut file);
                }
                Message::WeatherLatInput => {
                    weather_lat_do(&mut ui.latitude_input, &mut file);
                }
                Message::WeatherGridSize => {
                    weather_grid_size_do(&mut ui.grid_size_input, &mut file);
                }
                Message::WeatherClimInput => {
                    weather_climate_do(&mut ui.weather_type, &mut file, &climates);
                }
                Message::WeatherRandomSeed => {
                    weather_seed_random_do(&mut ui.weather_noise_random_seed, &mut ui.weather_seed_input, &mut file);
                }
                Message::GenWeather => {
                    let noise: Fbm<Perlin> = Fbm::new(file.weather.seed.unwrap());
                    let map: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::from_raw(512, 512, file.eroded_raw_512.clone()).unwrap();
                    let map_b = map.clone();
                    let terrain_map = Terrain::new(
                        &context,
                        terrain_material.clone(),
                        Arc::new(
                            move |x, y| {
                                map_b.clone().get_pixel(x as u32, y as u32).channels().first().unwrap().clone() as f32 * 0.01
                            }
                        ),
                        512.0,
                        1.0,
                        three_d::prelude::Vec2::new(255.0, 255.0)
                    );
                    terrain = terrain_map;
                    let component_size = 512.0 / file.weather.clone().grid_size as f64;
                    let min_total = map.iter().as_slice().iter().min().unwrap();
                    let max_total = map.iter().as_slice().iter().max().unwrap();

                    for component in grid.as_mut_slice() {
                                let mut h: i32;
                                match component.index.1 {
                                    0 => {
                                        let dynamic = DynamicImage::from(map.clone());
                                        let area = dynamic.clone().crop_imm(component_size as u32 * component.index.0 as u32, component_size as u32 * component.index.2 as u32, component_size as u32, component_size as u32);
                                        h = get_height(&area, file.topography.clone().max_height as f64, *min_total, *max_total);
                                    },
                                    _ => { h = 4000 * component.index.1; },
                                };
                                component.altitude = h as f64;
                                let gen = GenData::gen_year_data(file.weather.clone().latitude as i32, component.altitude, component.index, noise.clone(), file.weather.clone().koppen.unwrap());

                                component.humidity = gen.humidity;
                                component.pressure = gen.pressure;
                                component.td = gen.td;
                                component.temperature = gen.temperature;
                                component.wind = gen.wind;
                    }
                    file.weather_data = grid.clone();
                    println!("Finished.");
                },
                Message::ViewHumidity => {
                    set_view_state(&mut view_state, WeatherVisualization::Humidity);
                    update_grid_at_time(view_state.hour, &mut file.weather_data, &mut mesh_v, &view_state);
                    ui.legend_box.set_image(Some(SharedImage::load("icons/humidity_legend.png").unwrap()));
                    ui.legend_box.redraw_label();
                },
                Message::ViewPressure => {
                    set_view_state(&mut view_state, WeatherVisualization::Pressure);
                    update_grid_at_time(view_state.hour, &mut file.weather_data, &mut mesh_v, &view_state);
                    ui.legend_box.set_image(Some(SharedImage::load("icons/pressure_legend.png").unwrap()));
                    ui.legend_box.redraw_label();
                },
                Message::ViewTemperature => {
                    set_view_state(&mut view_state, WeatherVisualization::Temperature);
                    update_grid_at_time(view_state.hour, &mut file.weather_data, &mut mesh_v, &view_state);
                    ui.legend_box.set_image(Some(SharedImage::load("icons/temp_legend.png").unwrap()));
                    ui.legend_box.redraw_label();
                },
                Message::ViewWind => {
                    set_view_state(&mut view_state, WeatherVisualization::Wind);
                    update_grid_at_time(view_state.hour, &mut file.weather_data, &mut mesh_v, &view_state);
                },
                Message::DaySlider => {
                    set_hour(&mut ui.day_vis_slider, &mut view_state);
                    update_grid_at_time(view_state.hour, &mut file.weather_data, &mut mesh_v, &view_state);
                }
                Message::SaveFile => {
                    save_file_do(&mut file, &mut is_file_workspace, &mut workspace_path, &mut file_name);
                }
                Message::SaveFileAs => {
                    save_file_do(&mut file, &mut false, &mut workspace_path, &mut file_name);
                    let _ = replace::<bool>(&mut is_file_workspace, true);
                }
                Message::NewScenario => {
                    new_do(&mut file);
                    println!("{workspace_path}");
                    let _ = replace::<bool>(&mut is_file_workspace, false);
                }
                Message::OpenFile => {
                    let mut p = open_file_do(&mut file);
                    ui.main_window.set_label(format!("OpenBattlesim - {:?}", p.1).as_str());
                    let s = p.1.to_str().unwrap().to_string();
                    workspace_path = s;
                    let _ = replace::<bool>(&mut is_file_workspace, true);

                    // test
                    file.topography = p.0.topography;
                    file.weather = p.0.weather;
                    file.weather_data = p.0.weather_data;
                    file.discharge = p.0.discharge;
                    file.eroded_full = p.0.eroded_full;
                    file.eroded_full_color = p.0.eroded_full_color;
                    file.raw_full = p.0.raw_full;
                    file.color_eroded_512 = p.0.color_eroded_512;
                    file.eroded_raw_512 = p.0.eroded_raw_512;
                    file.color_map_512 = p.0.color_map_512.clone();
                    file.raw_map_512 = p.0.raw_map_512;
                    
                    if !file.color_map_512.is_empty() {
                        update_noise_img(&mut ui.preview_box_topo, &file, 0);
                    }
                    if !file.color_eroded_512.is_empty() {
                        update_noise_img(&mut ui.preview_erosion_topo, &file, 1);
                    }
                    if !file.eroded_full_color.is_empty() {
                        update_hydro_prev(&mut ui.hydro_preview, true, &mut file);
                    }
                    if !file.discharge.is_empty() {
                        update_hydro_prev(&mut ui.hydro_mask_preview, false, &mut file);
                    }
                    

                    ui.seed_input.set_value(format!("{}", &file.topography.seed.unwrap().clone()).as_str());
                    ui.noise_octaves_input.set_value(format!("{}", &file.topography.noise_octaves.unwrap().clone()).as_str());
                    ui.noise_freq_input.set_value(format!("{}", &file.topography.noise_frequency.unwrap().clone()).as_str());
                    ui.noise_lacunarity_input.set_value(format!("{}", &file.topography.noise_lacunarity.unwrap().clone()).as_str());
                    ui.min_height_input.set_value(file.topography.min_height.clone() as f64);
                    ui.max_height_input.set_value(file.topography.max_height.clone() as f64);
                    match &file.topography.noise_type.clone().unwrap() {
                        NoiseTypesUi::Simplex => ui.noise_choice.set_value(0),
                        NoiseTypesUi::Perlin => ui.noise_choice.set_value(1),
                        NoiseTypesUi::BillowPerlin => ui.noise_choice.set_value(2)
                    };
                    ui.erosion_cycles_input.set_value(file.topography.erosion_cycles.clone() as f64);
                    ui.weather_seed_input.set_value(format!("{}", &file.weather.seed.unwrap().clone()).as_str());
                    for choice in ui.weather_type.clone().into_iter() {
                        if choice.label().unwrap().to_string() == file.clone().weather.koppen.unwrap().name {
                            ui.weather_type.set_item(&choice);
                            break
                        } else {
                            continue
                        }
                    }
                    ui.latitude_input.set_value(format!("{}", &file.weather.latitude.clone()).as_str());
                    ui.grid_size_input.set_value(format!("{}", &file.weather.grid_size.clone()).as_str());
                }
            }
        }
        {
            gl_widget.make_current();
            context.set_viewport(viewport);
            let rt = RenderTarget::screen(&context, viewport.width, viewport.height);
            rt
                // Clear color and depth of the render target
                .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
                // Render the triangle with the per vertex colors defined at construction
                .render(&camera, &terrain, &[&directional, &ambient]);
            rt.render(&camera, &mesh_v, &[&directional, &ambient]);

            frame += 1;
            // app::sleep(0.10);
            gl_widget.redraw();
        }
    }
}


