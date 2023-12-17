
use fltk::browser::*;
use fltk::button::*;
use fltk::dialog::*;
use fltk::enums::*;
use fltk::frame::*;
use fltk::group::*;
use fltk::image::*;
use fltk::input::*;
use fltk::menu::*;
use fltk::misc::*;
use fltk::output::*;
use fltk::prelude::*;
use fltk::table::*;
use fltk::text::*;
use fltk::tree::*;
use fltk::valuator::*;
use fltk::widget::*;
use fltk::window::*;

#[derive(Debug, Clone)]
pub struct UserInterface {
    pub main_window: Window,
    pub menu_bar: MenuBar,
    pub preview_pane: Group,
    pub final_preview_box: Group,
    pub preview_scroll_v: Scrollbar,
    pub preview_scroll_h: Scrollbar,
    pub topography_pane: Group,
    pub first_group: Group,
    pub console_output_ero: Output,
    pub topo_menu_box: Group,
    pub seed_input: Input,
    pub seed_random_button: Button,
    pub erode_terrain_button: Button,
    pub noise_opts_group: Group,
    pub noise_choice: Choice,
    pub noise_octaves_input: Input,
    pub noise_freq_input: Input,
    pub noise_lacunarity_input: Input,
    pub terrain_settings_box: Group,
    pub high_elev_slider: ValueSlider,
    pub sea_elev_slider: ValueSlider,
    pub min_height_input: ValueInput,
    pub max_height_input: ValueInput,
    pub erosion_opts_box: Group,
    pub erosion_cycles_input: ValueInput,
    pub topo_preview: Group,
    pub topo_ero_preview: Group,
    pub preview_box_topo: Frame,
    pub hydrography_pane: Group,
    pub hydro_preview: Group,
    pub hydro_mask_preview: Group,
    pub weather_pane: Group,
    pub weather_preview: Group,
    pub weather_seed_input: Input,
    pub weather_noise_random_seed: Button,
    pub weather_noise_choice: Choice,
    pub weather_noise_octaves_input: Input,
    pub weather_type: Choice,
    pub up_type: Choice,
    pub down_type: Choice,
    pub left_type: Choice,
    pub right_type: Choice,
    pub latitude_input: Input,
    pub grid_size_input: Input,
    pub generate_weather_button: Button,
    pub weather_viewer_toolbox: Frame,
    pub misc_pane: Group,
}

impl UserInterface {
    pub fn make_window() -> Self {
	let mut main_window = Window::new(877, 258, 870, 579, None);
	main_window.set_label("OpenBattlesim Map Generator");
	main_window.set_type(WindowType::Double);
	let mut menu_bar = MenuBar::new(0, 0, 870, 30, None);
	let idx = menu_bar.add_choice("File/Open Scenario...");
	let idx = menu_bar.add_choice("File/Save Scenario");
	let idx = menu_bar.add_choice("File/Save Scenario as...");
	let idx = menu_bar.add_choice("Edit/Set Scenario Parameters...");
	menu_bar.end();
	let mut fl2rust_widget_0 = Tabs::new(-3, 29, 898, 567, None);
	let mut preview_pane = Group::new(0, 60, 870, 520, None);
	preview_pane.set_label("Preview");
	fl2rust_widget_0.resizable(&preview_pane);
	preview_pane.hide();
	let mut final_preview_box = Group::new(14, 80, 841, 475, None);
	final_preview_box.set_label("Final preview");
	preview_pane.resizable(&final_preview_box);
	final_preview_box.set_color(Color::by_index(32));
	final_preview_box.set_frame(FrameType::ThinDownFrame);
	let mut preview_scroll_v = Scrollbar::new(835, 80, 20, 455, None);
	let mut preview_scroll_h = Scrollbar::new(15, 535, 820, 20, None);
	preview_scroll_h.set_type(ScrollbarType::Horizontal);
	final_preview_box.end();
	preview_pane.end();
	let mut topography_pane = Group::new(0, 55, 870, 525, None);
	topography_pane.set_label("Topography");
	let mut first_group = Group::new(0, 60, 760, 520, None);
	first_group.set_frame(FrameType::BorderBox);
	let mut console_output_ero = Output::new(260, 81, 310, 485, None);
	first_group.resizable(&console_output_ero);
	first_group.end();
	let mut topo_menu_box = Group::new(585, 55, 285, 525, None);
	topo_menu_box.set_align(unsafe {std::mem::transmute(192)});
	topo_menu_box.set_frame(FrameType::FlatBox);
	let mut seed_input = Input::new(630, 56, 165, 24, None);
	seed_input.set_label("Seed:");
	let mut seed_random_button = Button::new(800, 55, 64, 25, None);
	seed_random_button.set_label("Random");
	seed_random_button.set_frame(FrameType::GtkUpBox);
	seed_random_button.set_down_frame(FrameType::GtkDownBox);
	let mut erode_terrain_button = Button::new(595, 535, 265, 25, None);
	erode_terrain_button.set_label("Erode terrain");
	erode_terrain_button.set_color(Color::by_index(247));
	erode_terrain_button.set_selection_color(Color::by_index(247));
	erode_terrain_button.set_frame(FrameType::ThinUpBox);
	erode_terrain_button.set_down_frame(FrameType::ThinDownBox);
	topo_menu_box.end();
	let mut noise_opts_group = Group::new(595, 95, 265, 175, None);
	noise_opts_group.set_color(Color::by_index(37));
	noise_opts_group.set_frame(FrameType::BorderFrame);
	let mut noise_choice = Choice::new(719, 120, 95, 25, None);
	noise_choice.set_label("Noise type:");
	noise_choice.set_down_frame(FrameType::BorderBox);
	noise_choice.end();
	let mut noise_octaves_input = Input::new(720, 155, 95, 25, None);
	noise_octaves_input.set_label("Octaves:");
	let mut noise_freq_input = Input::new(720, 190, 95, 25, None);
	noise_freq_input.set_label("Frequency:");
	let mut noise_lacunarity_input = Input::new(720, 226, 95, 25, None);
	noise_lacunarity_input.set_label("Lacunarity:");
	noise_opts_group.end();
	let mut fl2rust_widget_1 = Frame::new(596, 85, 110, 25, None);
	fl2rust_widget_1.set_label("Noise settings");
	fl2rust_widget_1.set_frame(FrameType::FlatBox);
	let mut terrain_settings_box = Group::new(595, 285, 265, 170, None);
	terrain_settings_box.set_color(Color::by_index(37));
	terrain_settings_box.set_frame(FrameType::BorderFrame);
	let mut high_elev_slider = ValueSlider::new(615, 315, 225, 20, None);
	high_elev_slider.set_label("Mountain %");
	high_elev_slider.set_type(SliderType::HorizontalNice);
	high_elev_slider.set_align(unsafe {std::mem::transmute(1)});
	high_elev_slider.set_frame(FrameType::ThinDownBox);
	high_elev_slider.set_text_size(14);
	high_elev_slider.set_maximum(100 as _);
	high_elev_slider.set_step(1 as _, 1);
	let mut sea_elev_slider = ValueSlider::new(615, 356, 225, 21, None);
	sea_elev_slider.set_label("Sea level %");
	sea_elev_slider.set_type(SliderType::HorizontalNice);
	sea_elev_slider.set_align(unsafe {std::mem::transmute(1)});
	sea_elev_slider.set_frame(FrameType::ThinDownBox);
	sea_elev_slider.set_text_size(14);
	sea_elev_slider.set_maximum(100 as _);
	sea_elev_slider.set_step(1 as _, 1);
	let mut min_height_input = ValueInput::new(740, 388, 80, 25, None);
	min_height_input.set_label("Minimum height (m)");
	let mut max_height_input = ValueInput::new(740, 420, 80, 25, None);
	max_height_input.set_label("Maximum height (m)");
	terrain_settings_box.end();
	let mut fl2rust_widget_2 = Frame::new(596, 271, 110, 25, None);
	fl2rust_widget_2.set_label("Terrain settings");
	fl2rust_widget_2.set_frame(FrameType::FlatBox);
	let mut erosion_opts_box = Group::new(595, 465, 265, 55, None);
	erosion_opts_box.set_color(Color::by_index(37));
	erosion_opts_box.set_frame(FrameType::BorderFrame);
	let mut erosion_cycles_input = ValueInput::new(660, 486, 65, 24, None);
	erosion_cycles_input.set_label("Cycles:");
	erosion_opts_box.end();
	let mut fl2rust_widget_3 = Frame::new(595, 455, 110, 25, None);
	fl2rust_widget_3.set_label("Erosion settings");
	fl2rust_widget_3.set_frame(FrameType::FlatBox);
	let mut topo_preview = Group::new(15, 80, 230, 230, None);
	topo_preview.set_label("Original Preview");
	topo_preview.set_align(unsafe {std::mem::transmute(193)});
	topo_preview.set_color(Color::by_index(0));
	topo_preview.set_frame(FrameType::EngravedFrame);
	topo_preview.set_label_color(Color::by_index(8));
	topo_preview.end();
	let mut topo_ero_preview = Group::new(15, 335, 230, 230, None);
	topo_ero_preview.set_label("Erosion Preview");
	topo_ero_preview.set_color(Color::by_index(0));
	topo_ero_preview.set_frame(FrameType::EngravedFrame);
	topo_ero_preview.set_label_color(Color::by_index(8));
	topo_ero_preview.end();
	let mut preview_box_topo = Frame::new(15, 80, 230, 230, None);
	topography_pane.end();
	let mut hydrography_pane = Group::new(0, 60, 895, 520, None);
	hydrography_pane.set_label("Hydrography");
	hydrography_pane.hide();
	let mut fl2rust_widget_4 = Group::new(0, 60, 870, 520, None);
	fl2rust_widget_4.set_frame(FrameType::BorderBox);
	fl2rust_widget_4.end();
	let mut hydro_preview = Group::new(19, 125, 400, 400, None);
	hydro_preview.set_label("Preview");
	hydro_preview.set_color(Color::by_index(0));
	hydro_preview.set_frame(FrameType::EngravedFrame);
	hydro_preview.set_label_color(Color::by_index(8));
	hydro_preview.end();
	let mut hydro_mask_preview = Group::new(450, 125, 400, 400, None);
	hydro_mask_preview.set_label("Watershed Preview");
	hydro_mask_preview.set_color(Color::by_index(0));
	hydro_mask_preview.set_frame(FrameType::EngravedFrame);
	hydro_mask_preview.set_label_color(Color::by_index(8));
	hydro_mask_preview.end();
	hydrography_pane.end();
	let mut weather_pane = Group::new(0, 60, 870, 520, None);
	weather_pane.set_label("Weather");
	weather_pane.hide();
	let mut fl2rust_widget_5 = Group::new(0, 60, 870, 520, None);
	fl2rust_widget_5.set_frame(FrameType::BorderBox);
	let mut weather_preview = Group::new(28, 99, 445, 445, None);
	weather_preview.set_label("Grid Preview");
	weather_preview.set_color(Color::by_index(37));
	weather_preview.set_frame(FrameType::BorderFrame);
	weather_preview.set_label_color(Color::by_index(8));
	weather_preview.end();
	let mut fl2rust_widget_6 = Group::new(500, 60, 370, 520, None);
	fl2rust_widget_6.set_align(unsafe {std::mem::transmute(192)});
	fl2rust_widget_6.set_frame(FrameType::FlatBox);
	let mut weather_seed_input = Input::new(605, 61, 165, 24, None);
	weather_seed_input.set_label("Seed:");
	let mut weather_noise_random_seed = Button::new(775, 60, 64, 25, None);
	weather_noise_random_seed.set_label("Random");
	weather_noise_random_seed.set_frame(FrameType::GtkUpBox);
	weather_noise_random_seed.set_down_frame(FrameType::GtkDownBox);
	let mut fl2rust_widget_7 = Group::new(570, 100, 265, 105, None);
	fl2rust_widget_7.set_color(Color::by_index(37));
	fl2rust_widget_7.set_frame(FrameType::BorderFrame);
	let mut weather_noise_choice = Choice::new(694, 125, 95, 25, None);
	weather_noise_choice.set_label("Noise type:");
	weather_noise_choice.set_down_frame(FrameType::BorderBox);
	weather_noise_choice.end();
	let mut weather_noise_octaves_input = Input::new(695, 160, 95, 25, None);
	weather_noise_octaves_input.set_label("Octaves:");
	fl2rust_widget_7.end();
	let mut fl2rust_widget_8 = Frame::new(571, 90, 110, 25, None);
	fl2rust_widget_8.set_label("Noise settings");
	fl2rust_widget_8.set_frame(FrameType::FlatBox);
	let mut fl2rust_widget_9 = Group::new(570, 233, 265, 264, None);
	fl2rust_widget_9.set_color(Color::by_index(37));
	fl2rust_widget_9.set_frame(FrameType::BorderFrame);
	let mut weather_type = Choice::new(694, 258, 95, 25, None);
	weather_type.set_label("Köppen type:");
	weather_type.set_down_frame(FrameType::BorderBox);
	weather_type.end();
	let mut up_type = Choice::new(694, 365, 95, 25, None);
	up_type.set_label("Up:");
	up_type.set_down_frame(FrameType::BorderBox);
	let idx = up_type.add_choice("Mountains");
	let idx = up_type.add_choice("Water");
	let idx = up_type.add_choice("Plains");
	up_type.end();
	let mut down_type = Choice::new(694, 401, 95, 25, None);
	down_type.set_label("Down:");
	down_type.set_down_frame(FrameType::BorderBox);
	let idx = down_type.add_choice("Mountains");
	let idx = down_type.add_choice("Water");
	let idx = down_type.add_choice("Plains");
	down_type.end();
	let mut left_type = Choice::new(694, 436, 95, 25, None);
	left_type.set_label("Left:");
	left_type.set_down_frame(FrameType::BorderBox);
	let idx = left_type.add_choice("Mountains");
	let idx = left_type.add_choice("Water");
	let idx = left_type.add_choice("Plains");
	left_type.end();
	let mut right_type = Choice::new(694, 471, 95, 25, None);
	right_type.set_label("Right:");
	right_type.set_down_frame(FrameType::BorderBox);
	let idx = right_type.add_choice("Mountains");
	let idx = right_type.add_choice("Water");
	let idx = right_type.add_choice("Plains");
	right_type.end();
	let mut latitude_input = Input::new(695, 293, 95, 25, None);
	latitude_input.set_label("Latitude:");
	let mut grid_size_input = Input::new(695, 329, 95, 24, None);
	grid_size_input.set_label("Grid size:");
	fl2rust_widget_9.end();
	let mut fl2rust_widget_10 = Frame::new(571, 223, 110, 25, None);
	fl2rust_widget_10.set_label("Climate settings");
	fl2rust_widget_10.set_frame(FrameType::FlatBox);
	let mut generate_weather_button = Button::new(570, 519, 265, 25, None);
	generate_weather_button.set_label("Generate weather grid");
	generate_weather_button.set_color(Color::by_index(247));
	generate_weather_button.set_selection_color(Color::by_index(247));
	generate_weather_button.set_frame(FrameType::ThinUpBox);
	generate_weather_button.set_down_frame(FrameType::ThinDownBox);
	fl2rust_widget_6.end();
	let mut weather_viewer_toolbox = Frame::new(498, 99, 45, 445, None);
	weather_viewer_toolbox.set_color(Color::by_index(38));
	weather_viewer_toolbox.set_frame(FrameType::BorderFrame);
	fl2rust_widget_5.end();
	weather_pane.end();
	let mut misc_pane = Group::new(0, 60, 870, 520, None);
	misc_pane.set_label("Misc");
	misc_pane.hide();
	misc_pane.end();
	fl2rust_widget_0.end();
	main_window.end();
	main_window.show();
	Self {
	    main_window,
	    menu_bar,
	    preview_pane,
	    final_preview_box,
	    preview_scroll_v,
	    preview_scroll_h,
	    topography_pane,
	    first_group,
	    console_output_ero,
	    topo_menu_box,
	    seed_input,
	    seed_random_button,
	    erode_terrain_button,
	    noise_opts_group,
	    noise_choice,
	    noise_octaves_input,
	    noise_freq_input,
	    noise_lacunarity_input,
	    terrain_settings_box,
	    high_elev_slider,
	    sea_elev_slider,
	    min_height_input,
	    max_height_input,
	    erosion_opts_box,
	    erosion_cycles_input,
	    topo_preview,
	    topo_ero_preview,
	    preview_box_topo,
	    hydrography_pane,
	    hydro_preview,
	    hydro_mask_preview,
	    weather_pane,
	    weather_preview,
	    weather_seed_input,
	    weather_noise_random_seed,
	    weather_noise_choice,
	    weather_noise_octaves_input,
	    weather_type,
	    up_type,
	    down_type,
	    left_type,
	    right_type,
	    latitude_input,
	    grid_size_input,
	    generate_weather_button,
	    weather_viewer_toolbox,
	    misc_pane,
	}
    }
}


