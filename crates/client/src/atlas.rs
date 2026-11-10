//! Mapa vetorial original; a rede regional é planejamento, separado do circuito piloto 3D.
use fyrox::{
    core::{algebra::Vector2, color::Color, pool::Handle},
    gui::{
        HorizontalAlignment, UiNode, UserInterface, VerticalAlignment,
        border::{Border, BorderBuilder},
        brush::Brush,
        canvas::CanvasBuilder,
        formatted_text::WrapMode,
        text::{Text, TextBuilder, TextMessage},
        vector_image::{Primitive, VectorImage, VectorImageBuilder},
        widget::{WidgetBuilder, WidgetMessage},
    },
    keyboard::KeyCode,
};
use game_core::south::{self, CITIES, Preference, ROADS, SERVICES};

pub struct Atlas {
    pub open: bool,
    root: Handle<Border>,
    selection: Handle<VectorImage>,
    legend: Handle<Text>,
    summary: Handle<Text>,
    regional_canvas: Handle<UiNode>,
    local_canvas: Handle<UiNode>,
    local_player: Handle<VectorImage>,
    local_roads: Handle<VectorImage>,
    local_depots: Handle<VectorImage>,
    local_names: [Handle<Text>; 3],
    local_job: Handle<Text>,
    overview: bool,
    local: bool,
    from: u8,
    to: u8,
    preference: Preference,
}
fn point(city: u8) -> Vector2<f32> {
    let [x, z] = CITIES[city as usize].position;
    Vector2::new(36.0 + (x + 4600.0) * 0.047, 66.0 + (5000.0 - z) * 0.033)
}
fn line(from: u8, to: u8, thickness: f32) -> Primitive {
    Primitive::Line {
        begin: point(from),
        end: point(to),
        thickness,
    }
}
fn dot(city: u8, radius: f32) -> Primitive {
    Primitive::Circle {
        center: point(city),
        radius,
        segments: 12,
    }
}
fn local_point(position: [f32; 2], overview: bool) -> Vector2<f32> {
    if overview {
        Vector2::new(
            250.0 + position[0] * 0.6,
            70.0 + (9100.0 - position[1]) * 0.043,
        )
    } else {
        Vector2::new(60.0 + position[0] * 1.3, 140.0 - position[1] * 0.32)
    }
}
fn local_depots(overview: bool) -> Vec<Primitive> {
    game_core::pilot::DEPOTS
        .iter()
        .filter(|d| overview || d.position[1] < 300.0)
        .map(|d| Primitive::Circle {
            center: local_point(d.position, overview),
            radius: 8.0,
            segments: 12,
        })
        .collect()
}
fn local_roads(overview: bool) -> Vec<Primitive> {
    let lines: [([f32; 2], [f32; 2]); 8] = [
        ([0.0, -1000.0], [0.0, 9000.0]),
        ([240.0, -1000.0], [240.0, 200.0]),
        ([0.0, -1000.0], [240.0, -1000.0]),
        ([0.0, 200.0], [240.0, 200.0]),
        ([0.0, 0.0], [60.0, 0.0]),
        ([0.0, -800.0], [60.0, -800.0]),
        ([0.0, 9000.0], [60.0, 9000.0]),
        ([0.0, 9000.0], [0.0, 9200.0]),
    ];
    lines
        .into_iter()
        .filter(|(a, b)| overview || a[1] < 300.0 || b[1] < 300.0)
        .flat_map(|(a, mut b)| {
            if !overview {
                b[1] = b[1].min(300.0);
            }
            let step = if overview {
                32.0
            } else {
                game_core::terrain::STEP
            };
            let count = ((a[1] - b[1]).abs() / step).ceil().max(1.0) as usize;
            let sample = move |i: usize| {
                let t = i as f32 / count as f32;
                let p = game_core::terrain::warp([
                    a[0] + (b[0] - a[0]) * t,
                    0.0,
                    a[1] + (b[1] - a[1]) * t,
                ])
                .expect("via interna válida");
                local_point([p[0], p[2]], overview)
            };
            (0..count).map(move |i| Primitive::Line {
                begin: sample(i),
                end: sample(i + 1),
                thickness: 4.0,
            })
        })
        .collect()
}
fn layer(ui: &mut UserInterface, color: Color, primitives: Vec<Primitive>) -> Handle<VectorImage> {
    VectorImageBuilder::new(
        WidgetBuilder::new()
            .with_width(510.0)
            .with_height(520.0)
            .with_foreground(Brush::Solid(color).into())
            .with_hit_test_visibility(false),
    )
    .with_primitives(primitives)
    .build(&mut ui.build_ctx())
}
fn text(
    ui: &mut UserInterface,
    x: f32,
    y: f32,
    width: f32,
    size: f32,
    value: &str,
) -> Handle<Text> {
    TextBuilder::new(
        WidgetBuilder::new()
            .with_desired_position(Vector2::new(x, y))
            .with_width(width)
            .with_foreground(Brush::Solid(Color::WHITE).into()),
    )
    .with_font_size(size.into())
    .with_wrap(WrapMode::Word)
    .with_text(value)
    .build(&mut ui.build_ctx())
}
impl Atlas {
    pub fn new(ui: &mut UserInterface) -> Self {
        let main = layer(
            ui,
            Color::opaque(235, 157, 62),
            ROADS
                .iter()
                .filter(|r| r.main)
                .map(|r| line(r.from, r.to, 3.0))
                .collect(),
        );
        let secondary = layer(
            ui,
            Color::opaque(205, 191, 102),
            ROADS
                .iter()
                .filter(|r| !r.main)
                .map(|r| line(r.from, r.to, 1.5))
                .collect(),
        );
        let cities = layer(
            ui,
            Color::opaque(230, 234, 222),
            (0..32).map(|c| dot(c, 3.5)).collect(),
        );
        let services = layer(
            ui,
            Color::opaque(121, 187, 132),
            SERVICES
                .iter()
                .map(|s| Primitive::WireCircle {
                    center: point(s.city),
                    radius: 7.0,
                    thickness: 1.5,
                    segments: 12,
                })
                .collect(),
        );
        let selection = layer(ui, Color::opaque(94, 221, 237), Vec::new());
        let title = text(
            ui,
            24.0,
            14.0,
            930.0,
            21.0,
            "SUL DO BRASIL · PLANEJAMENTO REGIONAL",
        );
        let summary = text(ui, 24.0, 510.0, 480.0, 12.0, "");
        let legend = text(ui, 525.0, 52.0, 425.0, 14.0, "");
        let regional_canvas = CanvasBuilder::new(
            WidgetBuilder::new()
                .with_width(980.0)
                .with_height(620.0)
                .with_visibility(false)
                .with_children([
                    main.transmute(),
                    secondary.transmute(),
                    cities.transmute(),
                    services.transmute(),
                    selection.transmute(),
                    title.transmute(),
                    summary.transmute(),
                    legend.transmute(),
                ]),
        )
        .build(&mut ui.build_ctx());
        let roads = layer(ui, Color::opaque(235, 157, 62), local_roads(true));
        let depots = layer(ui, Color::opaque(121, 187, 132), local_depots(true));
        let local_player = layer(ui, Color::opaque(94, 221, 237), Vec::new());
        let local_title = text(
            ui,
            24.0,
            14.0,
            930.0,
            21.0,
            "MAPA DIRIGÍVEL · POLO INICIAL E CORREDOR NORTE",
        );
        let local_legend = text(
            ui,
            525.0,
            70.0,
            410.0,
            17.0,
            "TRÊS PÁTIOS DISPONÍVEIS\n\nVerde: área de carga\nAzul: seu caminhão\nLaranja: vias dirigíveis\n\nGuarapuava: cerca de 9 km ao norte\nUnião da Vitória: polo inicial\nPorto União: 800 m ao sul do polo\n\nF: escolher destino antes da carga\nE: aceitar, carregar ou entregar\nMantenha cavalo e baú engatados e parados na área verde por 2 segundos.\n\nZ: corredor / circuito aproximado\nR: voltar ao polo e reiniciar treino\nTab: planejamento das 32 cidades\nM / Esc: voltar ao volante\n\nCorredor original, distâncias interurbanas em 1:20; não é reprodução cartográfica.",
        );
        let label = |ui: &mut UserInterface, index: usize| {
            let depot = game_core::pilot::DEPOTS[index];
            let p = local_point(depot.position, true);
            text(
                ui,
                p.x + 14.0,
                p.y - 8.0,
                200.0,
                15.0,
                CITIES[depot.city as usize].name,
            )
        };
        let north = label(ui, 0);
        let south = label(ui, 1);
        let guarapuava = label(ui, 2);
        let local_job = text(
            ui,
            24.0,
            530.0,
            480.0,
            14.0,
            "F: escolher destino no pátio · E: carregar",
        );
        let local_canvas = CanvasBuilder::new(
            WidgetBuilder::new()
                .with_width(980.0)
                .with_height(620.0)
                .with_children([
                    roads.transmute(),
                    depots.transmute(),
                    local_player.transmute(),
                    local_title.transmute(),
                    local_legend.transmute(),
                    north.transmute(),
                    south.transmute(),
                    guarapuava.transmute(),
                    local_job.transmute(),
                ]),
        )
        .build(&mut ui.build_ctx());
        let canvas = CanvasBuilder::new(
            WidgetBuilder::new()
                .with_children([regional_canvas.transmute(), local_canvas.transmute()]),
        )
        .build(&mut ui.build_ctx());
        let root = BorderBuilder::new(
            WidgetBuilder::new()
                .with_width(980.0)
                .with_height(620.0)
                .with_horizontal_alignment(HorizontalAlignment::Center)
                .with_vertical_alignment(VerticalAlignment::Center)
                .with_background(Brush::Solid(Color::opaque(24, 39, 44)).into())
                .with_child(canvas)
                .with_visibility(false),
        )
        .build(&mut ui.build_ctx());
        let result = Self {
            open: false,
            root,
            selection,
            legend,
            summary,
            regional_canvas: regional_canvas.transmute(),
            local_canvas: local_canvas.transmute(),
            local_player,
            local_roads: roads,
            local_depots: depots,
            local_names: [north, south, guarapuava],
            local_job,
            overview: true,
            local: true,
            from: south::UNION,
            to: south::PORTO,
            preference: Preference::Shortest,
        };
        result.refresh(ui);
        result
    }
    pub fn toggle(&mut self, ui: &UserInterface) {
        self.center(ui);
        self.open = !self.open;
        ui.send(self.root, WidgetMessage::Visibility(self.open));
    }
    pub fn key(&mut self, key: KeyCode, ui: &mut UserInterface) {
        if self.local && key == KeyCode::KeyZ {
            self.overview = !self.overview;
            ui[self.local_roads]
                .primitives
                .set_value_and_mark_modified(local_roads(self.overview));
            ui[self.local_depots]
                .primitives
                .set_value_and_mark_modified(local_depots(self.overview));
            ui[self.local_roads].invalidate_visual();
            ui[self.local_depots].invalidate_visual();
            for (&label, depot) in self.local_names.iter().zip(game_core::pilot::DEPOTS) {
                let p = local_point(depot.position, self.overview);
                ui.send(
                    label,
                    WidgetMessage::DesiredPosition(p + Vector2::new(14.0, -8.0)),
                );
                ui.send(
                    label,
                    WidgetMessage::Visibility(self.overview || depot.position[1] < 300.0),
                );
            }
            return;
        }
        if key == KeyCode::Tab {
            self.local = !self.local;
            ui.send(self.local_canvas, WidgetMessage::Visibility(self.local));
            ui.send(self.regional_canvas, WidgetMessage::Visibility(!self.local));
            return;
        }
        if self.local && !matches!(key, KeyCode::Escape | KeyCode::KeyM) {
            return;
        }
        match key {
            KeyCode::Escape | KeyCode::KeyM => {
                self.toggle(ui);
                return;
            }
            KeyCode::ArrowDown => self.to = (self.to + 1) % 32,
            KeyCode::ArrowUp => self.to = (self.to + 31) % 32,
            KeyCode::KeyO => self.from = self.to,
            KeyCode::Space => {
                self.preference = match self.preference {
                    Preference::Shortest => Preference::Fastest,
                    Preference::Fastest => Preference::NoTolls,
                    Preference::NoTolls => Preference::Shortest,
                }
            }
            _ => return,
        }
        self.refresh(ui);
    }
    fn refresh(&self, ui: &mut UserInterface) {
        let route = south::route(self.from, self.to, self.preference, None)
            .expect("rede estática conectada");
        let mut primitives = route
            .cities
            .windows(2)
            .map(|w| line(w[0], w[1], 4.0))
            .collect::<Vec<_>>();
        primitives.extend([dot(self.from, 6.0), dot(self.to, 8.0)]);
        ui[self.selection]
            .primitives
            .set_value_and_mark_modified(primitives);
        ui[self.selection].invalidate_visual();
        let list = CITIES
            .iter()
            .enumerate()
            .map(|(i, c)| {
                format!(
                    "{} {} · {}{}",
                    if i == self.to as usize { ">" } else { "  " },
                    c.state.name(),
                    c.name,
                    if SERVICES.iter().any(|s| s.city as usize == i) {
                        "  +"
                    } else {
                        ""
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        ui.send(self.legend, TextMessage::Text(list));
        let preference = match self.preference {
            Preference::Shortest => "curta",
            Preference::Fastest => "rápida",
            Preference::NoTolls => "sem pedágios",
        };
        let itinerary = route
            .cities
            .iter()
            .take(3)
            .map(|&i| CITIES[i as usize].name)
            .collect::<Vec<_>>()
            .join(" > ");
        ui.send(self.summary, TextMessage::Text(format!(
            "{} > {} · rota {}\n{:.1} km esquemáticos · {} pedágios propostos\n{}\nCima/Baixo: destino · O: origem · Espaço: rota · Tab: circuito\n+ apoio planejado · corredores ainda não construídos em 3D",
            CITIES[self.from as usize].name, CITIES[self.to as usize].name,
            preference, route.meters / 1000.0, route.tolls,
            if route.cities.len() > 3 { format!("{itinerary} > ...") } else { itinerary })));
    }
    pub fn locate(&self, ui: &mut UserInterface, position: [f32; 2]) {
        self.center(ui);
        let upper = if self.overview { 9200.0 } else { 300.0 };
        let in_pilot =
            (-300.0..=400.0).contains(&position[0]) && (-1100.0..=upper).contains(&position[1]);
        let points = if in_pilot {
            vec![Primitive::Circle {
                center: local_point(position, self.overview),
                radius: 5.0,
                segments: 12,
            }]
        } else {
            Vec::new()
        };
        ui[self.local_player]
            .primitives
            .set_value_and_mark_modified(points);
        ui[self.local_player].invalidate_visual();
    }
    pub fn job(&self, ui: &UserInterface, from: u8, to: u8, message: &str) {
        ui.send(
            self.local_job,
            TextMessage::Text(format!(
                "Frete: {} > {}\nF: escolher destino no pátio · E: carregar/entregar\n{}",
                CITIES[from as usize].name, CITIES[to as usize].name, message
            )),
        );
    }
    fn center(&self, ui: &UserInterface) {
        let size = ui.screen_size();
        ui.send(
            self.root,
            WidgetMessage::DesiredPosition(Vector2::new(
                ((size.x - 980.0) * 0.5).max(0.0),
                ((size.y - 620.0) * 0.5).max(0.0),
            )),
        );
    }
}
