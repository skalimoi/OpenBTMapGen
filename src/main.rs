use crate::topo_settings::TopoSettings;
use fltk::app::Sender;
use map_range::MapRange;
use fltk::image::SharedImage;
use fltk::{*, prelude::*};
use image_crate::{DynamicImage, EncodableLayout, ImageBuffer, Luma, Pixel};
use noise::utils::NoiseMapBuilder;
use noise::{Fbm, MultiFractal, Perlin, Seedable};
use rand::{Rng, thread_rng};

use std::sync::Arc;
use fltk::window::GlutWindow;
use three_d::{AmbientLight, Camera, ClearState, ColorMaterial, Context, CpuMaterial, CpuMesh, CpuTexture, Cull, DirectionalLight, FromCpuMaterial, Gm, LightingModel, Mat4, Mesh, PhysicalMaterial, RenderTarget, Srgba, Terrain, Vec3, Viewport};
use std::default::Default;
use std::fs::File;
use std::fs;
use std::mem::{replace, swap};
use std::path::PathBuf;

use fltk::dialog::FileDialogOptions;
use fltk::enums::Shortcut;
use ron::de::from_reader;
use serde::{Deserialize, Serialize};


use topo_settings::NoiseTypesUi;
use topography::{DEFAULT_TOPOSETTINGS, DIMENSIONS};
use weather_pane::DEFAULT_WEATHERSETTINGS;
use crate::utils::get_height;
use crate::weather::{Climate, GenData, koppen_afam, koppen_as, koppen_aw, koppen_bsh, koppen_bsk, koppen_bwh, koppen_bwk, koppen_cfa, koppen_cfb, koppen_cfc, koppen_cwa, koppen_cwb, koppen_cwc, koppen_dfa, koppen_dfb, koppen_dfc, koppen_dsc, koppen_et};
use crate::weather_settings::WeatherSettings;
use crate::WeatherVisualization::Init;

mod erosion;
mod topo_settings;
mod ui;
mod utils;
mod weather;
mod weather_settings;
mod topography;
mod hydro;
mod weather_pane;


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
    ExportMap,
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
    w.add_emit(
        "&File/Export map...\t",
        Shortcut::Ctrl | 'e',
        menu::MenuFlag::Normal,
        *sender,
        Message::ExportMap
    );
}

fn new_do(program_data: &mut FileData) {
    let clean = FileData {
        topography: DEFAULT_TOPOSETTINGS,
        weather: DEFAULT_WEATHERSETTINGS,
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
        
    let _ = replace::<FileData>(program_data, clean);
}

fn set_data(loaded_data: &mut FileData, data: &mut FileData) {
    let _ = swap::<FileData>(data, &mut loaded_data.clone());
}

fn export_do(program_data: &mut FileData) {
    let mut nfc = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseSaveDir);
    nfc.set_option(FileDialogOptions::SaveAsConfirm);
    nfc.show();
    let dir = nfc.filename();
    let dir_string = dir.to_str().unwrap().to_string();
    println!("{}", dir_string.clone());
    fs::create_dir(dir_string.clone() + "/terrain/").expect("Error creating terrain directory.");
    fs::create_dir(dir_string.clone() + "/weather/").expect("Error creating weather directory.");
    fs::create_dir(dir_string.clone() + "/textures/").expect("Error creating textures directory.");
    
    // TODO: check for empty vecs and throw error
    // TODO: lossless rgb saving for texture creation when?
    let s = ron::ser::to_string(&program_data.weather_data).expect("Error serializing weather data.");
    fs::write(dir_string.clone() + "/weather/forecast.ron", s).expect("Error writing weather data.");
    let i: ImageBuffer<Luma<u16>, Vec<u16>> = image_crate::ImageBuffer::from_raw(8192, 8192, program_data.eroded_full.clone()).unwrap();
    i.save(dir_string.clone() + "/terrain/map.png").unwrap();
    let d: ImageBuffer<Luma<u8>, Vec<u8>> = image_crate::ImageBuffer::from_raw(8192, 8192, program_data.discharge.clone()).unwrap();
    d.save(dir_string.clone() + "/terrain/hydro.png").unwrap();
}

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

fn seed_input_do(w: &mut impl InputExt, data: &mut FileData) {
    if w.changed() {
        data.topography.set_seed(Some(w.value().parse().unwrap()));

        match data.topography.noise_type {
            Some(NoiseTypesUi::Simplex) => {
                topography::update_simplex_noise(DIMENSIONS, data);
            }
            Some(NoiseTypesUi::Perlin) => {
                topography::update_perlin_noise(data, DIMENSIONS);
            }
            Some(NoiseTypesUi::BillowPerlin) => {
                topography::update_billow_noise(DIMENSIONS, data);
            }
            _ => {}
        };
    }
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
            topography::update_simplex_noise(DIMENSIONS, data);
        }
        Some(NoiseTypesUi::Perlin) => {
            topography::update_perlin_noise(data, DIMENSIONS);
        }
        Some(NoiseTypesUi::BillowPerlin) => {
            topography::update_billow_noise(DIMENSIONS, data);
        }
        _ => {}
    };
}

fn aux_choice_do(data: &mut FileData) {
    match data.topography.noise_type.unwrap() {
        NoiseTypesUi::Perlin => {
            data.topography.set_type(Some(NoiseTypesUi::Perlin));
            topography::update_perlin_noise(data, DIMENSIONS);
        }
        NoiseTypesUi::Simplex => {
            data.topography.set_type(Some(NoiseTypesUi::Simplex));
            topography::update_simplex_noise(DIMENSIONS, data);
        }
        NoiseTypesUi::BillowPerlin => {
            data.topography.set_type(Some(NoiseTypesUi::BillowPerlin));
            topography::update_billow_noise(DIMENSIONS, data);
        }
    }
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
        seed: Some(100000),
        koppen: Some(koppen_cfa()),
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
    let win_icon = image::PngImage::load("icons/win.png").unwrap();
    win.set_icon(Some(win_icon));
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

    topography::noise_choice_do(&mut ui.noise_choice, &s);

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
                Message::ExportMap => { export_do(&mut file) }
                Message::MinHeightInput => { topo_settings.min_height = ui.min_height_input.value() as i32 }
                Message::MaxHeightInput => { topo_settings.max_height = ui.max_height_input.value() as i32 }
                Message::SimplexChoice => {
                    topography::change_noise_type(NoiseTypesUi::Simplex, &mut file);
                    aux_choice_do(&mut file);
                    topography::update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::PerlinChoice => {
                    topography::change_noise_type(NoiseTypesUi::Perlin, &mut file);
                    aux_choice_do(&mut file);
                    topography::update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::BillowChoice => {
                    topography::change_noise_type(NoiseTypesUi::BillowPerlin, &mut file);
                    aux_choice_do(&mut file);
                    topography::update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::SeedRandom => {
                    seed_random_do(
                        &mut ui.seed_random_button,
                        &mut file,
                        &mut ui.seed_input,
                    );
                    topography::update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::SeedInput => {
                    seed_input_do(&mut ui.seed_input, &mut file);
                    topography::update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::OctaveInput => {
                    topography::octaves_input_do(&mut ui.noise_octaves_input, &mut file);
                    topography::update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::FreqInput => {
                    topography::frequency_input_do(&mut ui.noise_freq_input, &mut file);
                    topography::update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::LacInput => {
                    topography::lacunarity_input_do(&mut ui.noise_lacunarity_input, &mut file);
                    topography::update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::MtnSlider => {
                    topography::mtn_slider_do(&mut ui.high_elev_slider, &mut file);
                    topography::update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::SeaSlider => {
                    topography::sea_slider_do(&mut ui.sea_elev_slider, &mut file);
                    topography::update_noise_img(&mut ui.preview_box_topo, &mut file, 0);
                }
                Message::CycleInput => {
                    topography::cycle_input_do(&mut ui.erosion_cycles_input, &mut file)
                }
                Message::ErodeButton => {
                    topography::erode_terrain_preview(&mut file);
                    topography::update_noise_img(&mut ui.preview_erosion_topo, &mut file, 1);
                }
                Message::FullPreview => {
                    hydro::hydro_preview_do(&mut file);
                    hydro::erode_heightmap_full(&mut file);
                    hydro::update_hydro_prev(&mut ui.hydro_preview, true, &mut file);
                    hydro::update_hydro_prev(&mut ui.hydro_mask_preview, false, &mut file);
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
                    weather_pane::update_grid_at_time(view_state.hour, &mut file.weather_data, &mut mesh_v, &view_state);
                }
                Message::WeatherSeedInput => {
                    weather_pane::weather_seed_do(&mut ui.weather_seed_input, &mut file);
                }
                Message::WeatherLatInput => {
                    weather_pane::weather_lat_do(&mut ui.latitude_input, &mut file);
                }
                Message::WeatherGridSize => {
                    weather_pane::weather_grid_size_do(&mut ui.grid_size_input, &mut file);
                }
                Message::WeatherClimInput => {
                    weather_pane::weather_climate_do(&mut ui.weather_type, &mut file, &climates);
                }
                Message::WeatherRandomSeed => {
                    weather_pane::weather_seed_random_do(&mut ui.weather_noise_random_seed, &mut ui.weather_seed_input, &mut file);
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
                    weather_pane::set_view_state(&mut view_state, WeatherVisualization::Humidity);
                    weather_pane::update_grid_at_time(view_state.hour, &mut file.weather_data, &mut mesh_v, &view_state);
                    ui.legend_box.set_image(Some(SharedImage::load("icons/humidity_legend.png").unwrap()));
                    ui.legend_box.redraw_label();
                },
                Message::ViewPressure => {
                    weather_pane::set_view_state(&mut view_state, WeatherVisualization::Pressure);
                    weather_pane::update_grid_at_time(view_state.hour, &mut file.weather_data, &mut mesh_v, &view_state);
                    ui.legend_box.set_image(Some(SharedImage::load("icons/pressure_legend.png").unwrap()));
                    ui.legend_box.redraw_label();
                },
                Message::ViewTemperature => {
                    weather_pane::set_view_state(&mut view_state, WeatherVisualization::Temperature);
                    weather_pane::update_grid_at_time(view_state.hour, &mut file.weather_data, &mut mesh_v, &view_state);
                    ui.legend_box.set_image(Some(SharedImage::load("icons/temp_legend.png").unwrap()));
                    ui.legend_box.redraw_label();
                },
                Message::ViewWind => {
                    weather_pane::set_view_state(&mut view_state, WeatherVisualization::Wind);
                    weather_pane::update_grid_at_time(view_state.hour, &mut file.weather_data, &mut mesh_v, &view_state);
                },
                Message::DaySlider => {
                    weather_pane::set_hour(&mut ui.day_vis_slider, &mut view_state);
                    weather_pane::update_grid_at_time(view_state.hour, &mut file.weather_data, &mut mesh_v, &view_state);
                }
                Message::SaveFile => {
                    save_file_do(&mut file, &mut is_file_workspace, &mut workspace_path, &mut file_name);
                    ui.main_window.set_label(format!("OpenBattlesim Map Generator - {}", workspace_path).as_str());
                }
                Message::SaveFileAs => {
                    save_file_do(&mut file, &mut false, &mut workspace_path, &mut file_name);
                    let _ = replace::<bool>(&mut is_file_workspace, true);
                }
                Message::NewScenario => {
                    new_do(&mut file);
                    ui.main_window.set_label("OpenBattlesim Map Generator - Untitled scenario");
                    let _ = replace::<bool>(&mut is_file_workspace, false);
                }
                Message::OpenFile => {
                    let mut p = open_file_do(&mut file);
                    let s = p.1.to_str().unwrap().to_string();
                    workspace_path = s;
                    ui.main_window.set_label(format!("OpenBattlesim Map Generator - {}", workspace_path).as_str());
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
                        topography::update_noise_img(&mut ui.preview_box_topo, &file, 0);
                    }
                    if !file.color_eroded_512.is_empty() {
                        topography::update_noise_img(&mut ui.preview_erosion_topo, &file, 1);
                    }
                    if !file.eroded_full_color.is_empty() {
                        hydro::update_hydro_prev(&mut ui.hydro_preview, true, &mut file);
                    }
                    if !file.discharge.is_empty() {
                        hydro::update_hydro_prev(&mut ui.hydro_mask_preview, false, &mut file);
                    }
                    

                    ui.seed_input.set_value(format!("{}", &file.topography.seed.unwrap().clone()).as_str());
                    ui.seed_input.redraw();
                    ui.noise_octaves_input.set_value(format!("{}", &file.topography.noise_octaves.unwrap().clone()).as_str());
                    ui.noise_octaves_input.redraw();
                    ui.noise_freq_input.set_value(format!("{}", &file.topography.noise_frequency.unwrap().clone()).as_str());
                    ui.noise_freq_input.redraw();
                    ui.noise_lacunarity_input.set_value(format!("{}", &file.topography.noise_lacunarity.unwrap().clone()).as_str());
                    ui.noise_lacunarity_input.redraw();
                    ui.min_height_input.set_value(file.topography.min_height.clone() as f64);
                    ui.min_height_input.redraw();
                    ui.max_height_input.set_value(file.topography.max_height.clone() as f64);
                    ui.max_height_input.redraw();
                    match &file.topography.noise_type.clone().unwrap() {
                        NoiseTypesUi::Simplex => ui.noise_choice.set_value(0),
                        NoiseTypesUi::Perlin => ui.noise_choice.set_value(1),
                        NoiseTypesUi::BillowPerlin => ui.noise_choice.set_value(2)
                    };
                    ui.noise_choice.redraw();
                    ui.erosion_cycles_input.set_value(file.topography.erosion_cycles.clone() as f64);
                    ui.erosion_cycles_input.redraw();
                    ui.weather_seed_input.set_value(format!("{}", &file.weather.seed.unwrap().clone()).as_str());
                    ui.weather_seed_input.redraw();
                    for choice in ui.weather_type.clone().into_iter() {
                        if choice.label().unwrap().to_string() == file.clone().weather.koppen.unwrap().name {
                            ui.weather_type.set_item(&choice);
                            break
                        } else {
                            continue
                        }
                    }
                    ui.weather_type.redraw();
                    ui.latitude_input.set_value(format!("{}", &file.weather.latitude.clone()).as_str());
                    ui.latitude_input.redraw();
                    ui.grid_size_input.set_value(format!("{}", &file.weather.grid_size.clone()).as_str());
                    ui.grid_size_input.redraw();

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


