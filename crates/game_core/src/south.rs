//! Rede original de planejamento baseada em docs/MAPA_SUL.md, sem dados cartográficos externos.
//! Coordenadas comprimidas para o diagrama; não representam estradas 3D já construídas.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Pr,
    Sc,
    Rs,
}
impl State {
    pub fn name(self) -> &'static str {
        match self {
            Self::Pr => "PR",
            Self::Sc => "SC",
            Self::Rs => "RS",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct City {
    pub name: &'static str,
    pub state: State,
    pub position: [f32; 2],
}
const fn city(name: &'static str, state: State, x: f32, z: f32) -> City {
    City {
        name,
        state,
        position: [x, z],
    }
}
pub const UNION: u8 = 10;
pub const PORTO: u8 = 21;
pub const CITIES: [City; 32] = [
    city("Curitiba", State::Pr, 2200.0, 1800.0),
    city("Londrina", State::Pr, 1200.0, 4800.0),
    city("Maringá", State::Pr, -300.0, 4700.0),
    city("Cascavel", State::Pr, -2700.0, 3200.0),
    city("Foz do Iguaçu", State::Pr, -4000.0, 2400.0),
    city("Ponta Grossa", State::Pr, 1400.0, 2600.0),
    city("Guarapuava", State::Pr, -700.0, 2200.0),
    city("Apucarana", State::Pr, 700.0, 4300.0),
    city("Umuarama", State::Pr, -2300.0, 4700.0),
    city("Paranaguá", State::Pr, 3300.0, 1700.0),
    city("União da Vitória", State::Pr, 0.0, 0.0),
    city("Florianópolis", State::Sc, 3000.0, -2300.0),
    city("Joinville", State::Sc, 2700.0, 0.0),
    city("Blumenau", State::Sc, 2100.0, -1000.0),
    city("Itajaí", State::Sc, 3000.0, -1200.0),
    city("Chapecó", State::Sc, -2400.0, -1000.0),
    city("Lages", State::Sc, 900.0, -2200.0),
    city("Criciúma", State::Sc, 1900.0, -3300.0),
    city("Tubarão", State::Sc, 2400.0, -2900.0),
    city("São Miguel do Oeste", State::Sc, -3600.0, -1100.0),
    city("Jaraguá do Sul", State::Sc, 2000.0, -300.0),
    city("Porto União", State::Sc, 0.0, -240.0),
    city("Porto Alegre", State::Rs, 1100.0, -5200.0),
    city("Caxias do Sul", State::Rs, 900.0, -3800.0),
    city("Pelotas", State::Rs, 0.0, -7200.0),
    city("Rio Grande", State::Rs, 200.0, -8000.0),
    city("Santa Maria", State::Rs, -1700.0, -5400.0),
    city("Passo Fundo", State::Rs, -900.0, -3300.0),
    city("Cruz Alta", State::Rs, -2000.0, -4000.0),
    city("Uruguaiana", State::Rs, -4400.0, -5800.0),
    city("Bagé", State::Rs, -1900.0, -6900.0),
    city("Ijuí", State::Rs, -2700.0, -3400.0),
];
#[derive(Clone, Copy, Debug)]
pub struct Road {
    pub from: u8,
    pub to: u8,
    pub main: bool,
    pub toll: bool,
}
const fn road(from: u8, to: u8, main: bool, toll: bool) -> Road {
    Road {
        from,
        to,
        main,
        toll,
    }
}
// Pedágios são escolhas de gameplay provisórias, não localização de concessões reais.
pub const ROADS: &[Road] = &[
    road(4, 3, true, false),
    road(3, 6, true, true),
    road(6, 0, true, true),
    road(0, 9, true, false),
    road(3, 8, false, false),
    road(8, 2, false, false),
    road(2, 7, true, false),
    road(7, 1, true, false),
    road(7, 5, false, false),
    road(5, 0, true, false),
    road(5, 6, false, false),
    road(6, UNION, true, false),
    road(UNION, PORTO, false, false),
    road(PORTO, 16, true, false),
    road(UNION, 0, false, false),
    road(PORTO, 15, true, false),
    road(15, 19, true, false),
    road(15, 16, true, false),
    road(PORTO, 20, false, false),
    road(20, 12, false, false),
    road(20, 13, false, false),
    road(13, 14, false, false),
    road(0, 12, true, true),
    road(12, 14, true, false),
    road(14, 11, true, true),
    road(11, 18, true, false),
    road(18, 17, true, false),
    road(17, 22, true, true),
    road(11, 16, true, false),
    road(16, 23, true, false),
    road(23, 22, true, true),
    road(19, 31, false, false),
    road(31, 28, false, false),
    road(28, 27, true, false),
    road(15, 27, true, false),
    road(31, 26, false, false),
    road(26, 29, true, false),
    road(28, 26, false, false),
    road(27, 23, true, false),
    road(27, 22, false, false),
    road(26, 22, true, false),
    road(29, 22, true, false),
    road(29, 30, false, false),
    road(30, 24, true, false),
    road(24, 25, true, false),
    road(26, 30, false, false),
    road(22, 24, true, false),
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preference {
    Shortest,
    Fastest,
    NoTolls,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Route {
    pub cities: Vec<u8>,
    pub roads: Vec<usize>,
    pub meters: f32,
    pub seconds: f32,
    pub tolls: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteError {
    UnknownCity,
    UnknownRoad,
    Unreachable,
}
pub fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
impl Road {
    pub fn meters(&self) -> f32 {
        distance(
            CITIES[self.from as usize].position,
            CITIES[self.to as usize].position,
        )
    }
    pub fn seconds(&self) -> f32 {
        self.meters() / if self.main { 22.0 } else { 13.0 }
    }
}
/// Busca limitada a 32 cidades; permite simular uma estrada fechada sem alterar a rede.
pub fn route(
    from: u8,
    to: u8,
    preference: Preference,
    closed: Option<usize>,
) -> Result<Route, RouteError> {
    if from as usize >= CITIES.len() || to as usize >= CITIES.len() {
        return Err(RouteError::UnknownCity);
    }
    if closed.is_some_and(|i| i >= ROADS.len()) {
        return Err(RouteError::UnknownRoad);
    }
    let mut cost = [f32::INFINITY; 32];
    let mut visited = [false; 32];
    let mut previous = [None; 32];
    cost[from as usize] = 0.0;
    for _ in 0..32 {
        let Some(current) = (0..32)
            .filter(|&i| !visited[i] && cost[i].is_finite())
            .min_by(|&a, &b| cost[a].total_cmp(&cost[b]))
        else {
            break;
        };
        if current == to as usize {
            break;
        }
        visited[current] = true;
        for (index, road) in ROADS.iter().enumerate() {
            if closed == Some(index) || preference == Preference::NoTolls && road.toll {
                continue;
            }
            let next = if road.from as usize == current {
                road.to as usize
            } else if road.to as usize == current {
                road.from as usize
            } else {
                continue;
            };
            let weight = if preference == Preference::Fastest {
                road.seconds()
            } else {
                road.meters()
            };
            let candidate = cost[current] + weight;
            if candidate < cost[next] {
                cost[next] = candidate;
                previous[next] = Some((current, index));
            }
        }
    }
    if !cost[to as usize].is_finite() {
        return Err(RouteError::Unreachable);
    }
    let mut result = Route {
        cities: vec![to],
        roads: Vec::new(),
        meters: 0.0,
        seconds: 0.0,
        tolls: 0,
    };
    let mut current = to as usize;
    while current != from as usize {
        let (parent, index) = previous[current].ok_or(RouteError::Unreachable)?;
        let road = ROADS[index];
        result.roads.push(index);
        result.cities.push(parent as u8);
        result.meters += road.meters();
        result.seconds += road.seconds();
        result.tolls += usize::from(road.toll);
        current = parent;
    }
    result.cities.reverse();
    result.roads.reverse();
    Ok(result)
}
#[derive(Clone, Copy, Debug)]
pub struct Services {
    pub city: u8,
    pub workshop: bool,
}
// Todos oferecem combustível e descanso; o polo também prevê garagem e fretes.
pub const SERVICES: [Services; 12] = [
    Services {
        city: UNION,
        workshop: true,
    },
    Services {
        city: 3,
        workshop: true,
    },
    Services {
        city: 7,
        workshop: false,
    },
    Services {
        city: 0,
        workshop: true,
    },
    Services {
        city: 15,
        workshop: true,
    },
    Services {
        city: 16,
        workshop: true,
    },
    Services {
        city: 12,
        workshop: false,
    },
    Services {
        city: 18,
        workshop: false,
    },
    Services {
        city: 27,
        workshop: true,
    },
    Services {
        city: 26,
        workshop: true,
    },
    Services {
        city: 22,
        workshop: true,
    },
    Services {
        city: 24,
        workshop: true,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalogue_has_all_states_and_unique_connected_edges() {
        for (state, count) in [(State::Pr, 11), (State::Sc, 11), (State::Rs, 10)] {
            assert_eq!(CITIES.iter().filter(|c| c.state == state).count(), count);
        }
        for (i, r) in ROADS.iter().enumerate() {
            assert!(r.from != r.to && r.meters().is_finite() && r.meters() > 0.0);
            assert!(
                !ROADS[..i]
                    .iter()
                    .any(|s| (s.from == r.from && s.to == r.to)
                        || (s.from == r.to && s.to == r.from))
            );
        }
        for from in 0..32 {
            for to in 0..32 {
                let trip = route(from, to, Preference::Shortest, None).unwrap();
                assert_eq!(trip.cities.first(), Some(&from));
                assert_eq!(trip.cities.last(), Some(&to));
                assert!(trip.cities.len() <= 32);
                assert_eq!(trip.cities.len(), trip.roads.len() + 1);
                let toll_free = route(from, to, Preference::NoTolls, None).unwrap();
                assert_eq!(toll_free.tolls, 0);
                assert!(trip.meters <= toll_free.meters + 0.01);
                assert!(
                    route(from, to, Preference::Fastest, None).unwrap().seconds
                        <= trip.seconds + 0.01
                );
            }
        }
    }
    #[test]
    fn hub_crossing_is_not_the_only_interstate_exit() {
        let closed = ROADS
            .iter()
            .position(|r| r.from == UNION && r.to == PORTO)
            .unwrap();
        let trip = route(UNION, PORTO, Preference::Shortest, Some(closed)).unwrap();
        assert!(trip.cities.len() > 2);
        assert!(!trip.roads.contains(&closed));
    }
    #[test]
    fn route_limits_and_same_city() {
        assert_eq!(
            route(255, 0, Preference::Shortest, None),
            Err(RouteError::UnknownCity)
        );
        assert_eq!(
            route(0, 0, Preference::Shortest, Some(usize::MAX)),
            Err(RouteError::UnknownRoad)
        );
        let trip = route(UNION, UNION, Preference::Shortest, None).unwrap();
        assert_eq!(trip.cities, [UNION]);
        assert_eq!(trip.meters, 0.0);
        for service in SERVICES {
            assert!((service.city as usize) < CITIES.len());
        }
    }
}
