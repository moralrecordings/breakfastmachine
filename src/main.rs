use glow::HasContext;
use imgui::Condition;
use imgui::Context;
use imgui::Ui;
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use sdl2::{
    event::Event,
    rect::Rect,
    video::{GLProfile, Window},
};
use sharded_slab::{Clear, Pool};

// - create a pool of Parts using the slab allocator
// - avoid components for now, instead just overload Parts?

struct Part {
    preset_id: u32,
    bbox: Rect,
    align: u32,
}

impl Default for Part {
    fn default() -> Self {
        Part {
            preset_id: 0,
            bbox: Rect::new(0, 0, 32, 32),
            align: 1,
        }
    }
}

impl Clear for Part {
    fn clear(&mut self) {}
}

struct MachineState {
    parts: Pool<Part>,
}

impl Default for MachineState {
    fn default() -> Self {
        MachineState { parts: Pool::new() }
    }
}

struct UiState {
    show_app_open: bool,
    show_app_preferences: bool,
}

impl Default for UiState {
    fn default() -> Self {
        UiState {
            show_app_open: false,
            show_app_preferences: false,
        }
    }
}

// Create a new glow context.
fn glow_context(window: &Window) -> glow::Context {
    unsafe {
        glow::Context::from_loader_function(|s| window.subsystem().gl_get_proc_address(s) as _)
    }
}

fn draw_menu_bar(ui: &Ui, state: &mut UiState) {
    if let Some(menu_bar) = ui.begin_main_menu_bar() {
        if let Some(menu) = ui.begin_menu("File") {
            ui.menu_item_config("Open")
                .shortcut("CTRL+O")
                .build_with_ref(&mut state.show_app_open);
            menu.end();
        };
        if let Some(menu) = ui.begin_menu("Edit") {
            ui.menu_item_config("Preferences")
                .shortcut("CTRL+P")
                .build_with_ref(&mut state.show_app_preferences);
            menu.end();
        };
        menu_bar.end();
    }
}

fn main() {
    /* initialize SDL and its video subsystem */
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    /* hint SDL to initialize an OpenGL 3.3 core profile context */
    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_profile(GLProfile::Core);

    /* create a new window, be sure to call opengl method on the builder when using glow! */
    let window = video_subsystem
        .window("Breakfast Machine", 1280, 720)
        .allow_highdpi()
        .opengl()
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    /* create a new OpenGL context and make it current */
    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).unwrap();

    /* enable vsync to cap framerate */
    window.subsystem().gl_set_swap_interval(1).unwrap();

    /* create new glow and imgui contexts */
    let gl = glow_context(&window);

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    /* setup platform and renderer, and fonts to imgui */
    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    let mut platform = SdlPlatform::init(&mut imgui);
    let mut renderer = AutoRenderer::initialize(gl, &mut imgui).unwrap();
    let mut event_pump = sdl.event_pump().unwrap();

    let mut state = UiState::default();

    let mut machine = MachineState::default();

    'main: loop {
        for event in event_pump.poll_iter() {
            /* pass all events to imgui platfrom */
            platform.handle_event(&mut imgui, &event);

            if let Event::Quit { .. } = event {
                break 'main;
            }
        }

        /* call prepare_frame before calling imgui.new_frame() */
        platform.prepare_frame(&mut imgui, &window, &event_pump);

        let ui = imgui.new_frame();

        let window_size = window.size();

        draw_menu_bar(ui, &mut state);
        /*
        ui.begin_main_menu_bar();
        let window_ui = ui.window("Breakfast Machine")
            .resizable(false)
            .movable(false)
            .title_bar(false)
            .menu_bar(true)
            .position([0.0, 0.0], Condition::Always)
            .size([window_size.0 as f32, window_size.1 as f32], Condition::Always)
            .build(|| {

                ui.text_wrapped("Choo choo");
            });
        */
        ui.show_demo_window(&mut true);

        let draw_data = imgui.render();

        unsafe { renderer.gl_context().clear(glow::COLOR_BUFFER_BIT) };
        renderer.render(draw_data).unwrap();

        window.gl_swap_window();
    }
}
