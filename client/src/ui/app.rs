use glium::glutin::ContextBuilder;
use glium::glutin::dpi::PhysicalPosition;
use glium::glutin::event::Event;
use glium::glutin::event::WindowEvent;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::monitor::MonitorHandle;
use glium::glutin::window::WindowBuilder;
use glium::glutin::dpi::LogicalSize;
use glium::Display;
use glium::Surface;
use imgui::Context;
use imgui::FontConfig;
use imgui::FontGlyphRanges;
use imgui::FontSource;
use imgui::Ui;
use imgui_glium_renderer::Renderer;
use imgui_winit_support::HiDpiMode;
use imgui_winit_support::WinitPlatform;
use std::time::Instant;
use std::env;
use crate::ui::clipboard;

pub struct Application {
    event_loop: EventLoop<()>,
    display: Display,
    imgui: Context,
    platform: WinitPlatform,
    renderer: Renderer,
}

impl Application {
    pub fn new(font_size: f32) -> Application {
        let event_loop = EventLoop::new();
        let context = ContextBuilder::new()
            .with_vsync(true);
        let builder = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(0.0, 0.0));
        let display = Display::new(builder, context, &event_loop)
            .expect("Failed to initialize display");

        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        let font_size = if font_size.le(&0.0) {
            17.0
        } else {
            font_size
        };

        if let Some(backend) = clipboard::init() {
            imgui.set_clipboard_backend(backend);
        } else {
            eprintln!("Failed to initialize clipboard");
        }

        let mut platform = WinitPlatform::init(&mut imgui);
        {
            let gl_window = display.gl_window();
            let window = gl_window.window();
            window.set_visible(false);

            let dpi_mode = if let Ok(factor) = env::var(
                "IMGUI_EXAMPLE_FORCE_DPI_FACTOR"
            ) {
                match factor.parse::<f64>() {
                    Ok(f) => HiDpiMode::Locked(f),
                    Err(e) => panic!("Invalid scaling factor: {}", e),
                }
            } else {
                HiDpiMode::Default
            };

            platform.attach_window(imgui.io_mut(), window, dpi_mode);
        }

        imgui.fonts().add_font(
            &[
                FontSource::TtfData {
                    data: include_bytes!("../../resources/msyh.ttc"),
                    size_pixels: font_size,
                    config: Some(FontConfig {
                        rasterizer_multiply: 1.5,
                        oversample_h: 4,
                        oversample_v: 4,
                        glyph_ranges: FontGlyphRanges::chinese_simplified_common(),
                        ..FontConfig::default()
                    }),
                },
                FontSource::TtfData {
                    data: include_bytes!("../../resources/msyhbd.ttc"),
                    size_pixels: font_size,
                    config: Some(FontConfig {
                        oversample_h: 4,
                        oversample_v: 4,
                        glyph_ranges: FontGlyphRanges::chinese_simplified_common(),
                        ..FontConfig::default()
                    }),
                },
            ]
        );

        let renderer = Renderer::init(
            &mut imgui, 
            &display
        ).expect("Failed to initialize renderer");

        Application {
            event_loop,
            display,
            imgui,
            platform,
            renderer,
        }
    }

    pub fn show_window<F: FnMut(&mut bool, &mut Display, &mut Ui) + 'static>(
        self, title: &'static str, width: f64, height: f64, resizable: bool, mut run_ui: F) {
        let Application {
            event_loop,
            mut display,
            mut imgui,
            mut platform,
            mut renderer,
            ..
        } = self;
        let mut last_frame = Instant::now();
        let monitor_handle = event_loop.primary_monitor().unwrap();
        let mut first_renderer_flag = true;

        event_loop.run(
            move |event, _, control_flow| {
                match event {
                    Event::NewEvents(_) => {                        
                        let now = Instant::now();

                        imgui.io_mut().update_delta_time(now - last_frame);
                        last_frame = now;

                        Self::init_display(
                            &mut first_renderer_flag,
                            &monitor_handle,
                            &display,
                            title,
                            width, 
                            height, 
                            resizable
                        );
                    }
                    Event::MainEventsCleared => {
                        let gl_window = display.gl_window();

                        platform
                            .prepare_frame(imgui.io_mut(), gl_window.window())
                            .expect("Failed to prepare frame");
                        gl_window.window().request_redraw();
                    }
                    Event::RedrawRequested(_) => {
                        let ui = imgui.frame();                        

                        let mut run = true;
                        run_ui(&mut run, &mut display, ui);

                        if !run {
                            *control_flow = ControlFlow::Exit;
                        }

                        let gl_window = display.gl_window();
                        let mut target = display.draw();
                        target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
                        platform.prepare_render(ui, gl_window.window());
                        
                        let draw_data = imgui.render();
                        renderer
                            .render(&mut target, draw_data)
                            .expect("Rendering failed");

                        target.finish().expect("Failed to swap buffers");
                    }
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    event => {
                        let gl_window = display.gl_window();
                        platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
                    }
                }
            }
        )
    }

    fn init_display(first: &mut bool, monitor: &MonitorHandle, display: &Display, title: &str, 
        width: f64, height: f64, resizable: bool) {
        if !*first {
            return;
        }

        let size = LogicalSize::new(width, height);
        display.gl_window().window().set_inner_size(size);
        
        let monitor_size = monitor.size();
        let monitor_pos = monitor.position();
        let window_size = display.gl_window().window().outer_size();

        let pos = PhysicalPosition {
            x: monitor_pos.x + (monitor_size.width as i32 - window_size.width as i32) / 2,
            y: monitor_pos.y + (monitor_size.height as i32 - window_size.height as i32) / 2,
        };

        display.gl_window().window().set_title(title);
        display.gl_window().window().set_outer_position(pos);
        display.gl_window().window().set_resizable(resizable);
        display.gl_window().window().set_visible(true);
        *first = false;
    }
}