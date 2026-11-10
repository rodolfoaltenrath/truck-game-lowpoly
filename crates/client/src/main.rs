#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
mod diagnostics;
mod network;
mod settings;
mod visual;
use clap::Parser;
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector2, Vector3},
        color::Color,
        pool::Handle,
        reflect::prelude::*,
        visitor::prelude::*,
    },
    dpi::LogicalSize,
    engine::{GraphicsContextParams, executor::Executor},
    event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::EventLoop,
    gui::{
        HorizontalAlignment, Thickness, UserInterface, VerticalAlignment,
        brush::Brush,
        text::{Text, TextBuilder, TextMessage},
        widget::WidgetBuilder,
    },
    keyboard::{KeyCode, PhysicalKey},
    plugin::{Plugin, PluginContext, error::GameResult},
    renderer::QualitySettings,
    scene::{
        Scene,
        base::BaseBuilder,
        camera::{Camera, CameraBuilder},
        light::{BaseLightBuilder, directional::DirectionalLightBuilder},
        transform::TransformBuilder,
    },
    window::WindowAttributes,
};
use std::{
    collections::{BTreeMap, HashSet},
    path::PathBuf,
    time::Instant,
};
use truck_sim::{DT, Input, Simulation, Snapshot};
use visual::{VehicleVisual, rotation};

#[derive(Parser, Clone)]
#[command(about = "Jogo de caminhão nativo Rust/OpenGL — Windows e Linux")]
struct Options {
    /// Joga sem servidor.
    #[arg(long)]
    solo: bool,
    /// Endereço WebSocket da sala LAN.
    #[arg(long, default_value = "ws://127.0.0.1:39100")]
    server: String,
    #[arg(long, default_value = "lan")]
    room: String,
    /// Encerra depois de N quadros renderizados (validação automatizada).
    #[arg(long,value_parser=clap::value_parser!(u32).range(1..=36000))]
    smoke_frames: Option<u32>,
    /// Acelera em linha reta durante o smoke para verificar integração física/render.
    #[arg(long, requires = "smoke_frames")]
    smoke_drive: bool,
    /// Relatório JSON do teste nativo.
    #[arg(long)]
    report: Option<PathBuf>,
    /// Captura somente a janela do jogo ao concluir --smoke-frames.
    #[arg(long, requires = "smoke_frames")]
    screenshot: Option<PathBuf>,
    /// Diretório de dados; use uma pasta junto ao executável para modo portátil.
    #[arg(long)]
    data_dir: Option<PathBuf>,
}
#[derive(Default, Visit, Reflect)]
#[reflect(non_cloneable)]
struct Game {
    #[visit(skip)]
    #[reflect(hidden)]
    options: Option<Options>,
    #[visit(skip)]
    #[reflect(hidden)]
    state: Option<State>,
}
impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Game").finish_non_exhaustive()
    }
}
struct State {
    scene: Handle<Scene>,
    camera: Handle<Camera>,
    hud: Handle<Text>,
    vehicle: VehicleVisual,
    sim: Simulation,
    previous: Snapshot,
    current: Snapshot,
    keys: HashSet<KeyCode>,
    accumulator: f32,
    start: Instant,
    frames: u32,
    paint: u8,
    quality: bool,
    camera_mode: u8,
    help: bool,
    network: Option<network::Network>,
    remotes: BTreeMap<u32, (u8, VehicleVisual)>,
    saved_at: Instant,
    orbit: f32,
    pitch: f32,
    distance: f32,
    dragging: bool,
    cursor: Option<(f64, f64)>,
    gpu: String,
}
impl Plugin for Game {
    fn init(&mut self, _: Option<&str>, ctx: PluginContext) -> GameResult {
        ctx.user_interfaces
            .add(UserInterface::new(Vector2::new(1100.0, 700.0)));
        let mut scene = Scene::new();
        visual::world(&mut scene);
        DirectionalLightBuilder::new(
            BaseLightBuilder::new(
                BaseBuilder::new().with_name("Sol").with_local_transform(
                    TransformBuilder::new()
                        .with_local_rotation(UnitQuaternion::from_euler_angles(-0.7, -0.5, 0.0))
                        .build(),
                ),
            )
            .with_color(visual::color(0xfff3d6))
            .with_intensity(1.0),
        )
        .build(&mut scene.graph);
        let camera = CameraBuilder::new(BaseBuilder::new().with_name("Câmera"))
            .with_z_near(0.15)
            .with_z_far(1600.0)
            .with_fov(65.0_f32.to_radians())
            .build(&mut scene.graph);
        let saved =
            settings::Settings::load(std::path::Path::new("settings.json")).unwrap_or_else(|e| {
                eprintln!("Preferências recusadas: {e}; usando padrão.");
                settings::Settings::default()
            });
        let vehicle = VehicleVisual::new(&mut scene.graph, saved.paint);
        let scene = ctx.scenes.add(scene);
        let hud = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(18.0))
                .with_horizontal_alignment(HorizontalAlignment::Left)
                .with_vertical_alignment(VerticalAlignment::Top)
                .with_foreground(Brush::Solid(Color::WHITE).into()),
        )
        .with_font_size(17.0.into())
        .with_shadow(true)
        .with_text("Iniciando...")
        .build(&mut ctx.user_interfaces.first_mut().build_ctx());
        let mut sim = Simulation::new();
        sim.restore(truck_sim::Pose {
            position: saved.position,
            rotation: saved.rotation,
        })
        .expect("preferências validadas");
        let current = sim.snapshot();
        let options = self.options.as_ref().expect("opções iniciais");
        let network = if options.solo {
            None
        } else {
            Some(
                network::Network::new(&options.server, &options.room)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?,
            )
        };
        self.state = Some(State {
            scene,
            camera,
            hud,
            vehicle,
            sim,
            previous: current.clone(),
            current,
            keys: HashSet::new(),
            accumulator: 0.0,
            start: Instant::now(),
            frames: 0,
            paint: saved.paint,
            quality: saved.shadows,
            camera_mode: saved.camera,
            help: true,
            network,
            remotes: BTreeMap::new(),
            saved_at: Instant::now(),
            orbit: 0.59,
            pitch: 0.32,
            distance: 22.0,
            dragging: false,
            cursor: None,
            gpu: String::new(),
        });
        Ok(())
    }
    fn on_graphics_context_initialized(&mut self, ctx: PluginContext) -> GameResult {
        let graphics = ctx.graphics_context.as_initialized_mut();
        let mut quality = QualitySettings::low();
        quality.csm_settings.enabled = self.state.as_ref().is_some_and(|s| s.quality);
        quality.hdr_settings.use_hdr = false;
        graphics.renderer.set_quality_settings(&quality)?;
        graphics
            .renderer
            .set_backbuffer_clear_color(visual::color(0x9dc4e3));
        let gpu = diagnostics::gpu(graphics);
        eprintln!("Renderizador nativo: {gpu}");
        if let Some(s) = self.state.as_mut() {
            s.gpu = gpu;
        }
        Ok(())
    }
    fn on_os_event(&mut self, event: &Event<()>, ctx: PluginContext) -> GameResult {
        let Some(s) = self.state.as_mut() else {
            return Ok(());
        };
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::Focused(false) => {
                    s.keys.clear();
                    s.dragging = false;
                    s.cursor = None;
                }
                WindowEvent::MouseInput {
                    state,
                    button: MouseButton::Right,
                    ..
                } => s.dragging = *state == ElementState::Pressed,
                WindowEvent::CursorMoved { position, .. } => {
                    if s.dragging
                        && let Some((x, y)) = s.cursor
                    {
                        s.orbit -= (position.x - x) as f32 * 0.005;
                        s.pitch = (s.pitch + (position.y - y) as f32 * 0.005).clamp(0.08, 1.2);
                    }
                    s.cursor = Some((position.x, position.y));
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let amount = match delta {
                        MouseScrollDelta::LineDelta(_, v) => *v,
                        MouseScrollDelta::PixelDelta(p) => p.y as f32 / 40.0,
                    };
                    s.distance = (s.distance - amount * 1.5).clamp(6.0, 45.0);
                }
                WindowEvent::CloseRequested => ctx.loop_controller.exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        if event.state == ElementState::Released {
                            s.keys.remove(&key);
                        } else {
                            s.keys.insert(key);
                            if !event.repeat {
                                match key {
                                    KeyCode::Escape => ctx.loop_controller.exit(),
                                    KeyCode::KeyR => {
                                        s.sim.reset();
                                        s.current = s.sim.snapshot();
                                        s.previous = s.current.clone();
                                    }
                                    KeyCode::KeyT => {
                                        s.sim.toggle_hitch();
                                    }
                                    KeyCode::KeyC => s.camera_mode = (s.camera_mode + 1) % 3,
                                    KeyCode::KeyH => s.help = !s.help,
                                    KeyCode::KeyP => {
                                        s.paint = (s.paint + 1) % 6;
                                        let graph = &mut ctx.scenes[s.scene].graph;
                                        let new = VehicleVisual::new(graph, s.paint);
                                        let old = std::mem::replace(&mut s.vehicle, new);
                                        old.remove(graph);
                                    }
                                    KeyCode::KeyG => {
                                        s.quality = !s.quality;
                                        let mut q = QualitySettings::low();
                                        q.csm_settings.enabled = s.quality;
                                        q.hdr_settings.use_hdr = false;
                                        q.fxaa = false;
                                        ctx.graphics_context
                                            .as_initialized_mut()
                                            .renderer
                                            .set_quality_settings(&q)?;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn update(&mut self, ctx: &mut PluginContext) -> GameResult {
        let Some(s) = self.state.as_mut() else {
            return Ok(());
        };
        let pressed = |keys: &[KeyCode]| keys.iter().any(|k| s.keys.contains(k)) as u8 as f32;
        let mut input = Input {
            throttle: pressed(&[KeyCode::KeyW, KeyCode::ArrowUp]),
            brake: pressed(&[KeyCode::KeyS, KeyCode::ArrowDown]),
            steer: pressed(&[KeyCode::KeyD, KeyCode::ArrowRight])
                - pressed(&[KeyCode::KeyA, KeyCode::ArrowLeft]),
            handbrake: s.keys.contains(&KeyCode::Space),
        };
        if self.options.as_ref().is_some_and(|o| o.smoke_drive) {
            input.throttle = 1.0;
        }
        s.accumulator = (s.accumulator + ctx.dt).min(DT * 5.0);
        while s.accumulator >= DT {
            s.previous = s.current.clone();
            s.sim.step(input);
            s.current = s.sim.snapshot();
            s.accumulator -= DT;
        }
        if let Some(net) = s.network.as_mut() {
            net.update(&s.current, s.paint);
        }
        if s.saved_at.elapsed().as_secs() >= 2 {
            s.save();
            s.saved_at = Instant::now();
        }
        Ok(())
    }
    fn before_rendering(&mut self, ctx: PluginContext) -> GameResult {
        let Some(s) = self.state.as_mut() else {
            return Ok(());
        };
        let graph = &mut ctx.scenes[s.scene].graph;
        // Física a 60 Hz; interpolação de translação e orientação para qualquer taxa de vídeo.
        let alpha = (*ctx.lag / DT + s.accumulator / DT).clamp(0.0, 1.0);
        let mut view = s.current.clone();
        for (p, a, b) in [
            (&mut view.truck, s.previous.truck, s.current.truck),
            (&mut view.trailer, s.previous.trailer, s.current.trailer),
        ] {
            p.position =
                std::array::from_fn(|i| a.position[i] + (b.position[i] - a.position[i]) * alpha);
            let q = rotation(a.rotation).slerp(&rotation(b.rotation), alpha);
            let q = q.quaternion();
            p.rotation = [q.i, q.j, q.k, q.w];
        }
        for (w, a) in view.wheels.iter_mut().zip(&s.previous.wheels).chain(
            view.trailer_wheels
                .iter_mut()
                .zip(&s.previous.trailer_wheels),
        ) {
            w.center = std::array::from_fn(|i| a.center[i] + (w.center[i] - a.center[i]) * alpha);
            w.rotation = a.rotation + (w.rotation - a.rotation) * alpha;
        }
        s.vehicle.update(graph, &view);
        if let Some(net) = s.network.as_mut() {
            let views = net.views();
            let ids = views.iter().map(|v| v.player).collect::<HashSet<_>>();
            let expired = s
                .remotes
                .keys()
                .copied()
                .filter(|id| !ids.contains(id))
                .collect::<Vec<_>>();
            for id in expired {
                if let Some((_, visual)) = s.remotes.remove(&id) {
                    visual.remove(graph);
                }
            }
            for remote in views {
                if s.remotes
                    .get(&remote.player)
                    .is_some_and(|(paint, _)| *paint != remote.paint)
                    && let Some((_, visual)) = s.remotes.remove(&remote.player)
                {
                    visual.remove(graph);
                }
                let (_, visual) = s
                    .remotes
                    .entry(remote.player)
                    .or_insert_with(|| (remote.paint, VehicleVisual::new(graph, remote.paint)));
                visual.remote(graph, &remote);
            }
        }
        let position = Vector3::from(view.truck.position);
        let orientation = rotation(view.truck.rotation);
        let (offset, target) = match s.camera_mode {
            1 => (Vector3::new(0.0, 2.0, 3.8), Vector3::new(0.0, 1.8, 35.0)),
            2 => (Vector3::new(0.0, 35.0, -5.0), Vector3::new(0.0, 0.0, 0.0)),
            _ => (
                Vector3::new(
                    s.orbit.sin() * s.distance * s.pitch.cos(),
                    s.pitch.sin() * s.distance,
                    -s.orbit.cos() * s.distance * s.pitch.cos(),
                ),
                Vector3::new(0.0, 0.8, -2.0),
            ),
        };
        let eye = position + orientation * offset;
        let target = position + orientation * target;
        graph[s.camera]
            .local_transform_mut()
            .set_position(eye)
            .set_rotation(UnitQuaternion::face_towards(&(target - eye), &Vector3::y()));
        s.frames += 1;
        let graphics = ctx.graphics_context.as_initialized_mut();
        let fps = graphics.renderer.get_statistics().frames_per_second;
        if s.frames % 15 == 0 {
            let help = if s.help {
                "\nW/S: acelerar / frear / ré   A/D: direção\nEspaço: freio de mão   T: engate   R: reiniciar\nC: câmera   P: pintura   G: sombras   H: ajuda   Esc: sair"
            } else {
                ""
            };
            let status = s
                .network
                .as_ref()
                .map_or_else(|| "solo".into(), |n| n.status());
            let text = format!(
                "SUL DO BRASIL · PROTÓTIPO NATIVO\n{:03.0} km/h  |  marcha {}  |  {:04.0} rpm\n{} FPS  ·  {}  ·  reboque {}\n{}{}",
                view.speed.abs() * 3.6,
                view.gear,
                view.rpm,
                fps,
                if s.quality { "sombras" } else { "leve" },
                if view.coupled { "engatado" } else { "solto" },
                status,
                help
            );
            ctx.user_interfaces
                .first()
                .send(s.hud, TextMessage::Text(text));
        }
        if self
            .options
            .as_ref()
            .and_then(|o| o.smoke_frames)
            .is_some_and(|n| s.frames >= n)
        {
            if let Some(path) = self.options.as_ref().and_then(|o| o.screenshot.as_ref()) {
                diagnostics::capture(graphics, path)
                    .map_err(|e| e.into_boxed_dyn_error() as Box<dyn std::error::Error + Send>)?;
            }
            if let Some(path) = self.options.as_ref().and_then(|o| o.report.as_ref()) {
                let report = serde_json::json!({"platform":std::env::consts::OS,"native":true,"gpu":s.gpu,"frames":s.frames,"elapsed_seconds":s.start.elapsed().as_secs_f64(),"fps":fps,"truck_position":view.truck.position,"wheel_contacts":view.wheels.iter().filter(|w|w.contact).count(),"coupled":view.coupled,"connected":s.network.as_ref().is_some_and(|n|n.connected()),"remotes":s.remotes.len()});
                std::fs::write(
                    path,
                    serde_json::to_vec_pretty(&report).expect("relatório interno"),
                )
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
            }
            ctx.loop_controller.exit();
        }
        Ok(())
    }
}
impl State {
    fn save(&self) {
        let saved = settings::Settings {
            version: 1,
            position: self.current.truck.position,
            rotation: self.current.truck.rotation,
            paint: self.paint,
            shadows: self.quality,
            camera: self.camera_mode,
        };
        if let Err(e) = saved.save(std::path::Path::new("settings.json")) {
            eprintln!("Não foi possível salvar: {e}");
        }
    }
}
impl Drop for Game {
    fn drop(&mut self) {
        if let Some(s) = self.state.as_ref() {
            s.save();
        }
    }
}
fn main() -> anyhow::Result<()> {
    let mut options = Options::parse();
    if !options.solo {
        network::validate(&options.server, &options.room)?;
    }
    if let Some(report) = options.report.as_mut()
        && report.is_relative()
    {
        *report = std::env::current_dir()?.join(&*report);
    }
    if let Some(path) = options.screenshot.as_mut()
        && path.is_relative()
    {
        *path = std::env::current_dir()?.join(&*path);
    }
    let data_dir = options
        .data_dir
        .clone()
        .map_or_else(settings::data_dir, Ok)?;
    for path in [options.report.as_ref(), options.screenshot.as_ref()]
        .into_iter()
        .flatten()
    {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::create_dir_all(&data_dir)?;
    std::env::set_current_dir(data_dir)?;
    let event_loop = EventLoop::new()?;
    let mut executor = Executor::from_params(
        Some(event_loop),
        GraphicsContextParams {
            window_attributes: WindowAttributes::default()
                .with_title("Sul do Brasil — caminhão nativo")
                .with_inner_size(LogicalSize::new(1100.0, 700.0)),
            vsync: true,
            msaa_sample_count: None,
            graphics_server_constructor: Default::default(),
            named_objects: false,
        },
    );
    executor.set_desired_update_rate(60.0);
    executor.set_throttle_threshold(10.0);
    executor.set_resource_hot_reloading_enabled(false);
    executor.add_plugin(Game {
        options: Some(options),
        state: None,
    });
    executor.run();
    Ok(())
}
