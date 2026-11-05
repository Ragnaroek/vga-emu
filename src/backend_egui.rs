use eframe::{EframePumpStatus, EframeWinitApplication, UserEvent};
use egui::{Color32, ColorImage, Key, TextureHandle, TextureOptions};
use std::sync::{Arc, Mutex};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::{
    CRTReg, RenderBackend, VGABuilder, VGAEmu,
    backend::{PixelBuffer, is_linear, render_linear, render_planar},
    input::{InputMonitoring, NumCode},
    util::set_de,
};
use std::sync::{RwLock, RwLockWriteGuard};

pub struct EguiRenderBackend {
    backend: Option<EguiBackend>,
}

impl EguiRenderBackend {
    pub fn new() -> EguiRenderBackend {
        EguiRenderBackend { backend: None }
    }
}

struct EguiBackend {
    app: EframeWinitApplication<'static>,
    event_loop: EventLoop<UserEvent>,
    width: usize,
    height: usize,
    fb: Arc<Mutex<Vec<Color32>>>,
    input_monitoring: Arc<RwLock<InputMonitoring>>,
}

impl EguiBackend {
    pub fn input_monitoring<'a>(&'a mut self) -> RwLockWriteGuard<'a, InputMonitoring> {
        self.input_monitoring
            .write()
            .expect("write lock to InputMonitoring")
    }
}

impl RenderBackend for EguiRenderBackend {
    fn init(&mut self, width: usize, height: usize, builder: &VGABuilder) -> Result<(), String> {
        let event_loop = EventLoop::<UserEvent>::with_user_event()
            .build()
            .map_err(|e| e.to_string())?;
        event_loop.set_control_flow(ControlFlow::Poll);

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([width as f32, height as f32])
                .with_title(builder.title.clone()),
            ..Default::default()
        };

        let fb = Arc::new(Mutex::new(vec![Color32::BLACK; width * height]));
        let fb_app = fb.clone();

        let input_monitoring = Arc::new(RwLock::new(InputMonitoring::new()));
        let im_app = input_monitoring.clone();

        let app = eframe::create_native(
            &builder.title,
            native_options,
            Box::new(move |cc| Ok(Box::new(VGAApp::new(cc, width, height, fb_app, im_app)))),
            &event_loop,
        );

        self.backend = Some(EguiBackend {
            app,
            event_loop,
            width,
            height,
            fb,
            input_monitoring,
        });

        Ok(())
    }

    fn draw_frame(&mut self, vga: &mut VGAEmu) -> bool {
        let offset_delta = vga.regs.get_crt_data(CRTReg::Offset) as usize;
        if offset_delta == 0 {
            panic!("illegal CRT offset: {}", offset_delta);
        }

        let vmode = vga.regs.get_video_mode();
        let linear = is_linear(vmode);
        //TODO: inaccurate and currently a hack. This must be somehow inferred from the register states
        //but I haven't figured out how yet
        let v_stretch = if vmode == 0x13 { 2 } else { 1 };
        let mem_offset = vga.mem_offset();

        let be = self.backend.as_mut().expect("init");
        {
            let mut fb = be.fb.lock().unwrap();

            set_de(vga, true); //display enable is currently only set for whole frame (not toggled for horizontal retrace)
            if linear {
                render_linear(
                    vga,
                    mem_offset,
                    offset_delta,
                    be.height,
                    v_stretch,
                    &mut *fb,
                );
            } else {
                render_planar(vga, mem_offset, offset_delta, be.height, &mut *fb, be.width);
            }
        }
        set_de(vga, false);

        let r = match be
            .app
            .pump_eframe_app(&mut be.event_loop, Some(std::time::Duration::ZERO))
        {
            EframePumpStatus::Continue(_) => false,
            EframePumpStatus::Exit(_) => true,
        };

        r
    }

    fn input_monitoring<'a>(&'a mut self) -> RwLockWriteGuard<'a, InputMonitoring> {
        let be = self.backend.as_mut().expect("init");
        be.input_monitoring()
    }
}

struct VGAApp {
    width: usize,
    height: usize,
    fb: Arc<Mutex<Vec<Color32>>>,
    im: Arc<RwLock<InputMonitoring>>,
    texture: TextureHandle,
}

impl VGAApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>, width: usize, height: usize,
        fb: Arc<Mutex<Vec<Color32>>>, im: Arc<RwLock<InputMonitoring>>,
    ) -> VGAApp {
        let initial_fb = { fb.lock().expect("fb lock").clone() };

        let texture = cc.egui_ctx.load_texture(
            "framebuffer",
            ColorImage {
                size: [width, height],
                pixels: initial_fb,
                ..Default::default()
            },
            TextureOptions::NEAREST,
        );

        VGAApp {
            texture,
            fb,
            im,
            width,
            height,
        }
    }

    pub fn set_key(&self, key: Key) {
        let mut im = self.im.write().expect("write lock");
        // TODO map full key set
        if key == Key::Enter {
            im.keyboard.buttons[NumCode::Return as usize] = true;
            im.keyboard.update_last_value(NumCode::Return);
        }
    }

    pub fn clear_key(&self, key: Key) {
        let mut im = self.im.write().expect("write lock");
        // TODO map full key set
        if key == Key::Enter {
            im.keyboard.buttons[NumCode::Return as usize] = true;
        }
    }
}

impl eframe::App for VGAApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.input(|i| {
            for ev in &i.events {
                match ev {
                    egui::Event::Key { key, pressed, .. } => {
                        if *pressed {
                            self.set_key(*key);
                        } else {
                            self.clear_key(*key);
                        }
                    }
                    _ => {}
                }
            }
        });

        let pixels = self.fb.lock().expect("fb lock").clone();
        self.texture.set(
            ColorImage {
                size: [self.width, self.height],
                pixels,
                ..Default::default()
            },
            TextureOptions::NEAREST,
        );

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.add(egui::Image::new(&self.texture).shrink_to_fit());
                });
            });

        ui.request_repaint();
    }
}

impl PixelBuffer for Vec<Color32> {
    const PIXEL_WIDTH: usize = 1;
    fn set_rgb(&mut self, offset: usize, r: u8, g: u8, b: u8) {
        self[offset] = Color32::from_rgb(r, g, b);
    }
}
