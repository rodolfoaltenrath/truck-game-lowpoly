#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
mod atlas;
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
        widget::{WidgetBuilder, WidgetMessage},
    },
    keyboard::{KeyCode, PhysicalKey},
    plugin::{Plugin, PluginContext, error::GameResult},
    scene::{
        Scene,
        base::BaseBuilder,
        camera::{Camera, CameraBuilder, Exposure},
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
    /// Alterna aceleração, direção, freio e ré para testar estabilidade.
    #[arg(long, requires = "smoke_frames", conflicts_with_all = ["smoke_drive", "smoke_map"])]
    smoke_maneuvers: bool,
    /// Ângulo externo para revisão visual automatizada; não altera a câmera normal.
    #[arg(long, requires = "smoke_frames", value_parser = ["front", "side", "rear"])]
    smoke_view: Option<String>,
    /// Abre o mapa no smoke, para inspeção da interface sem dirigir.
    #[arg(long, requires = "smoke_frames", conflicts_with = "smoke_drive", value_parser = ["local", "regional"])]
    smoke_map: Option<String>,
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
    atlas: atlas::Atlas,
    freight: game_core::pilot::TrainingFreight,
    freight_message: String,
    vehicle: VehicleVisual,
    scenery: visual::Scenery,
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
    exit_reason: Option<&'static str>,
}
impl Plugin for Game {
    fn init(&mut self, _: Option<&str>, ctx: PluginContext) -> GameResult {
        ctx.user_interfaces
            .add(UserInterface::new(Vector2::new(1100.0, 700.0)));
        let mut scene = Scene::new();
        let scenery = visual::world(&mut scene);
        visual::lighting::environment(&mut scene);
        DirectionalLightBuilder::new(
            BaseLightBuilder::new(
                BaseBuilder::new().with_name("Sol").with_local_transform(
                    TransformBuilder::new()
                        .with_local_rotation(UnitQuaternion::from_euler_angles(-0.7, -0.5, 0.0))
                        .build(),
                ),
            )
            .with_color(visual::color(0xfff3d6))
            .with_intensity(1.8),
        )
        .build(&mut scene.graph);
        let camera = CameraBuilder::new(BaseBuilder::new().with_name("Câmera"))
            .with_z_near(0.15)
            .with_z_far(1600.0)
            .with_fov(65.0_f32.to_radians())
            .with_exposure(Exposure::Manual(1.0))
            .build(&mut scene.graph);
        let saved =
            settings::Settings::load(std::path::Path::new("settings.json")).unwrap_or_else(|e| {
                eprintln!("Preferências recusadas: {e}; usando padrão.");
                settings::Settings::default()
            });
        let mut vehicle = VehicleVisual::new(&mut scene.graph, saved.paint);
        vehicle
            .enable_detailed(&mut scene.graph, saved.paint)
            .map_err(|e| e.into_boxed_dyn_error() as Box<dyn std::error::Error + Send>)?;
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
        let mut atlas = atlas::Atlas::new(ctx.user_interfaces.first_mut());
        let mut sim = Simulation::new();
        sim.restore(truck_sim::Pose {
            position: saved.position,
            rotation: saved.rotation,
        })
        .expect("preferências validadas");
        let current = sim.snapshot();
        let freight = game_core::pilot::TrainingFreight::from_nearest(observation(&current))
            .expect("pose inicial validada");
        let options = self.options.as_ref().expect("opções iniciais");
        if let Some(mode) = &options.smoke_map {
            atlas.toggle(ctx.user_interfaces.first());
            if mode == "regional" {
                atlas.key(KeyCode::Tab, ctx.user_interfaces.first_mut());
            }
        }
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
            atlas,
            freight,
            freight_message: String::new(),
            vehicle,
            scenery,
            sim,
            previous: current.clone(),
            current,
            keys: HashSet::new(),
            accumulator: 0.0,
            start: Instant::now(),
            frames: 0,
            paint: saved.paint,
            quality: saved.shadows,
            camera_mode: if options.smoke_view.is_some() {
                0
            } else {
                saved.camera
            },
            help: true,
            network,
            remotes: BTreeMap::new(),
            saved_at: Instant::now(),
            orbit: match options.smoke_view.as_deref() {
                Some("front") => 2.55,
                Some("side") => std::f32::consts::FRAC_PI_2,
                _ => 0.59,
            },
            pitch: 0.32,
            distance: if options.smoke_view.as_deref() == Some("front") {
                14.0
            } else {
                22.0
            },
            dragging: false,
            cursor: None,
            gpu: String::new(),
            exit_reason: None,
        });
        Ok(())
    }
    fn on_graphics_context_initialized(&mut self, ctx: PluginContext) -> GameResult {
        let graphics = ctx.graphics_context.as_initialized_mut();
        let quality = visual::lighting::quality(self.state.as_ref().is_some_and(|s| s.quality));
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
                WindowEvent::CloseRequested => {
                    s.exit_reason = Some("fechamento solicitado pela janela/sistema");
                    ctx.loop_controller.exit();
                }
                WindowEvent::KeyboardInput { .. }
                    if self
                        .options
                        .as_ref()
                        .is_some_and(|o| o.smoke_frames.is_some()) => {}
                WindowEvent::KeyboardInput { event, .. } => {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        if s.atlas.open {
                            s.keys.clear();
                            if event.state == ElementState::Pressed && !event.repeat {
                                if key == KeyCode::KeyR {
                                    s.reset_to_hub();
                                    s.atlas.toggle(ctx.user_interfaces.first());
                                } else if key == KeyCode::KeyF {
                                    s.cycle_destination();
                                } else if key == KeyCode::KeyE {
                                    s.interact_freight();
                                } else {
                                    s.atlas.key(key, ctx.user_interfaces.first_mut());
                                }
                            }
                            return Ok(());
                        }
                        if event.state == ElementState::Released {
                            s.keys.remove(&key);
                        } else {
                            s.keys.insert(key);
                            if !event.repeat {
                                match key {
                                    KeyCode::KeyM => {
                                        s.keys.clear();
                                        s.dragging = false;
                                        s.cursor = None;
                                        s.atlas.toggle(ctx.user_interfaces.first());
                                    }
                                    KeyCode::Escape => {
                                        s.exit_reason = Some("tecla Escape");
                                        ctx.loop_controller.exit();
                                    }
                                    KeyCode::KeyR => {
                                        s.reset_to_hub();
                                    }
                                    KeyCode::KeyT => {
                                        s.sim.toggle_hitch();
                                    }
                                    KeyCode::KeyE => {
                                        s.interact_freight();
                                    }
                                    KeyCode::KeyF => {
                                        s.cycle_destination();
                                    }
                                    KeyCode::KeyC => s.camera_mode = (s.camera_mode + 1) % 3,
                                    KeyCode::KeyH => s.help = !s.help,
                                    KeyCode::KeyP => {
                                        s.paint = (s.paint + 1) % 6;
                                        let graph = &mut ctx.scenes[s.scene].graph;
                                        let mut new = VehicleVisual::new(graph, s.paint);
                                        new.enable_detailed(graph, s.paint).map_err(|e| {
                                            e.into_boxed_dyn_error()
                                                as Box<dyn std::error::Error + Send>
                                        })?;
                                        let old = std::mem::replace(&mut s.vehicle, new);
                                        old.remove(graph);
                                    }
                                    KeyCode::KeyG => {
                                        s.quality = !s.quality;
                                        let q = visual::lighting::quality(s.quality);
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
        if s.atlas.open {
            input = Input {
                brake: 1.0,
                handbrake: true,
                ..Input::default()
            };
        }
        if let Some(options) = &self.options
            && options.smoke_frames.is_some()
        {
            input = if options.smoke_maneuvers {
                maneuver_input(s.frames)
            } else if options.smoke_drive {
                Input {
                    throttle: 1.0,
                    ..Input::default()
                }
            } else {
                Input {
                    handbrake: true,
                    ..Input::default()
                }
            };
        }
        s.accumulator = (s.accumulator + ctx.dt).min(DT * 5.0);
        while s.accumulator >= DT {
            s.previous = s.current.clone();
            s.sim.step(input);
            s.current = s.sim.snapshot();
            if let Err(error) = s.freight.tick(DT, observation(&s.current)) {
                s.freight_message = freight_error(error).into();
            }
            s.accumulator -= DT;
        }
        if let Some(net) = s.network.as_mut() {
            net.update(&s.current, s.paint);
        }
        if s.saved_at.elapsed().as_secs() >= 2 {
            s.save();
            if s.exit_reason.is_none() {
                diagnostics::marker(&format!(
                    "executando · quadros {} · posição {:?} · velocidade {:.2} m/s · input throttle={:.1} brake={:.1} steer={:.2} handbrake={}\nGPU: {}\nSe o processo fechou e este estado permaneceu, não houve saída normal registrada.",
                    s.frames,
                    s.current.truck.position,
                    s.current.speed,
                    input.throttle,
                    input.brake,
                    input.steer,
                    input.handbrake,
                    s.gpu
                ));
            }
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
        s.scenery.update(graph, view.truck.position);
        if let Some(net) = s.network.as_mut() {
            let mut views = net.views();
            let distance_squared = |position: [f32; 3]| {
                (Vector3::from(position) - Vector3::from(view.truck.position)).norm_squared()
            };
            views.sort_by(|a, b| {
                distance_squared(a.position).total_cmp(&distance_squared(b.position))
            });
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
            for (rank, remote) in views.into_iter().enumerate() {
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
                visual
                    .detail_range(
                        graph,
                        remote.paint,
                        rank < 2 && distance_squared(remote.position) < 65.0 * 65.0,
                    )
                    .map_err(|e| e.into_boxed_dyn_error() as Box<dyn std::error::Error + Send>)?;
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
            ctx.user_interfaces
                .first()
                .send(s.hud, WidgetMessage::Visibility(!s.atlas.open));
            if s.atlas.open {
                s.atlas.locate(
                    ctx.user_interfaces.first_mut(),
                    [view.truck.position[0], view.truck.position[2]],
                );
                s.atlas.job(
                    ctx.user_interfaces.first(),
                    s.freight.origin().city,
                    s.freight.destination().city,
                    &s.freight_message,
                );
            }
            let help = if s.help {
                "\nW/S: acelerar / frear / ré   A/D: direção\nEspaço: freio de mão   T: engate   R: voltar ao pátio\nE: frete   F: escolher destino no pátio   M: mapa\nC: câmera   P: pintura   G: sombras   H: ajuda   Esc: sair"
            } else {
                ""
            };
            let status = s
                .network
                .as_ref()
                .map_or_else(|| "solo".into(), |n| n.status());
            let landmark = if (-2150.0..-1250.0).contains(&view.truck.position[2]) {
                "\nSERRA DO RIO DO RASTRO · trecho inspirado · curvas: 20 km/h\nAcesso leste, 1,4 km ao sul do polo · R: retornar ao pátio"
            } else {
                ""
            };
            let text = format!(
                "SUL DO BRASIL · PROTÓTIPO NATIVO\n{:03.0} km/h  |  marcha {}  |  {:04.0} rpm\n{} FPS  ·  {}  ·  reboque {} · {} regiões\n{}\n{}{}{}",
                view.speed.abs() * 3.6,
                view.gear,
                view.rpm,
                fps,
                if s.quality { "acabamento" } else { "leve" },
                if view.coupled { "engatado" } else { "solto" },
                s.scenery.resident_chunks(),
                status,
                s.freight_text(),
                landmark,
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
                let report = serde_json::json!({"camera_position":[eye.x,eye.y,eye.z],"camera_target":[target.x,target.y,target.z],"platform":std::env::consts::OS,"native":true,"gpu":s.gpu,"frames":s.frames,"elapsed_seconds":s.start.elapsed().as_secs_f64(),"fps":fps,"truck_position":view.truck.position,"wheel_contacts":view.wheels.iter().filter(|w|w.contact).count(),"coupled":view.coupled,"connected":s.network.as_ref().is_some_and(|n|n.connected()),"remotes":s.remotes.len()});
                std::fs::write(
                    path,
                    serde_json::to_vec_pretty(&report).expect("relatório interno"),
                )
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
            }
            s.exit_reason = Some("smoke concluído: limite de quadros atingido");
            ctx.loop_controller.exit();
        }
        Ok(())
    }
}
impl State {
    fn interact_freight(&mut self) {
        self.freight_message = match self.freight.interact(observation(&self.current)) {
            Ok(()) => String::new(),
            Err(error) => freight_error(error).into(),
        };
    }
    fn cycle_destination(&mut self) {
        self.freight_message = match self.freight.cycle_destination(observation(&self.current)) {
            Ok(()) => String::new(),
            Err(error) => freight_error(error).into(),
        };
    }
    fn reset_to_hub(&mut self) {
        let defaults = settings::Settings::default();
        self.sim
            .restore(truck_sim::Pose {
                position: defaults.position,
                rotation: defaults.rotation,
            })
            .expect("posição inicial interna");
        self.freight = game_core::pilot::TrainingFreight::default();
        self.freight_message.clear();
        self.current = self.sim.snapshot();
        self.previous = self.current.clone();
        self.accumulator = 0.0;
        self.keys.clear();
    }
    fn freight_text(&self) -> String {
        use game_core::pilot::Stage;
        let target = self.freight.target();
        let distance =
            game_core::south::distance(observation(&self.current).truck, target.position);
        let phase = match self.freight.stage() {
            Stage::Available => "F: destino · E: carregar · mantenha parado na área verde",
            Stage::Loading => "carregando · mantenha o conjunto parado",
            Stage::InTransit => "siga ao outro pátio · E para descarregar",
            Stage::Unloading => "descarregando · mantenha o conjunto parado",
            Stage::Complete => "entrega concluída · E para preparar o frete de retorno",
        };
        let progress = if matches!(self.freight.stage(), Stage::Loading | Stage::Unloading) {
            format!(" · {:.0}%", self.freight.progress() * 100.0)
        } else {
            String::new()
        };
        format!(
            "FRETE LOCAL · {} > {} · alvo {} · {:.0} m em linha reta\n{}{} · {} entregas no treino\n{}",
            game_core::south::CITIES[self.freight.origin().city as usize].name,
            game_core::south::CITIES[self.freight.destination().city as usize].name,
            game_core::south::CITIES[target.city as usize].name,
            distance,
            phase,
            progress,
            self.freight.deliveries(),
            self.freight_message
        )
    }
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
fn observation(snapshot: &Snapshot) -> game_core::pilot::Observation {
    game_core::pilot::Observation {
        truck: [snapshot.truck.position[0], snapshot.truck.position[2]],
        trailer: [snapshot.trailer.position[0], snapshot.trailer.position[2]],
        speed: snapshot.speed,
        coupled: snapshot.coupled,
    }
}
fn freight_error(error: game_core::pilot::FreightError) -> &'static str {
    use game_core::pilot::FreightError;
    match error {
        FreightError::OutsideDock => "Estacione cavalo e baú dentro da área verde do pátio.",
        FreightError::Moving => "Pare o caminhão para movimentar a carga.",
        FreightError::TrailerMissing => "Engate o baú antes de movimentar a carga.",
        FreightError::Busy => "Aguarde a operação atual.",
        FreightError::InvalidObservation | FreightError::InvalidStep => {
            "Operação recusada: estado inválido."
        }
    }
}
impl Drop for Game {
    fn drop(&mut self) {
        if let Some(s) = self.state.as_ref() {
            s.save();
            if !std::thread::panicking() {
                diagnostics::marker(s.exit_reason.unwrap_or("executor encerrado sem solicitação registrada; verificar console e código de saída"));
            }
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
    diagnostics::install_crash_handler();
    let event_loop = EventLoop::new()?;
    let mut executor = Executor::from_params(
        Some(event_loop),
        GraphicsContextParams {
            window_attributes: WindowAttributes::default()
                .with_title(if options.smoke_frames.is_some() {
                    "TESTE AUTOMÁTICO · fecha sozinho · Sul do Brasil"
                } else {
                    "Sul do Brasil — caminhão nativo"
                })
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
fn maneuver_input(frame: u32) -> Input {
    match (frame / 90) % 6 {
        0 => Input {
            throttle: 1.0,
            ..Input::default()
        },
        1 => Input {
            throttle: 0.6,
            steer: 0.55,
            ..Input::default()
        },
        2 => Input {
            brake: 1.0,
            steer: 0.4,
            ..Input::default()
        },
        3 => Input {
            brake: 1.0,
            ..Input::default()
        },
        4 => Input {
            throttle: 1.0,
            steer: -0.55,
            ..Input::default()
        },
        _ => Input {
            handbrake: true,
            ..Input::default()
        },
    }
}
