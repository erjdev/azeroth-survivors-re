//! Incremental local detours. Every edge is validated by the runtime body/floor sweep.
//! This is not a baked navmesh: large rooms and crowded route queues need further validation.
use super::units::flat;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

const SPACING: f32 = 2.0;
const MAX_NODES: usize = 512;
const EDGE_BUDGET: usize = 8;
const MAX_REQUESTS: usize = 64;
const REQUESTS_PER_FRAME: usize = 8;
const DIRECTIONS: [(i32, i32); 8] = [
    (1, 0),
    (0, 1),
    (-1, 0),
    (0, -1),
    (1, 1),
    (-1, 1),
    (-1, -1),
    (1, -1),
];

struct Node {
    at: Vec3,
    parent: Option<usize>,
    cost: f32,
    closed: bool,
}

struct Search {
    origin: Vec3,
    goal: Vec3,
    nodes: Vec<Node>,
    cells: HashMap<(i32, i32, i32), usize>,
    expanding: Option<(usize, usize)>,
}

enum Outcome {
    Pending,
    Found(VecDeque<Vec3>),
    Failed,
}

impl Search {
    fn new(origin: Vec3, goal: Vec3) -> Self {
        let mut search = Self {
            origin,
            goal,
            nodes: vec![Node {
                at: origin,
                parent: None,
                cost: 0.,
                closed: false,
            }],
            cells: HashMap::new(),
            expanding: None,
        };
        search.cells.insert(search.key(origin), 0);
        search
    }

    fn key(&self, at: Vec3) -> (i32, i32, i32) {
        let p = at - self.origin;
        (
            (p.x / SPACING).round() as i32,
            (p.z / SPACING).round() as i32,
            (p.y * 2.).round() as i32,
        )
    }

    fn advance(
        &mut self,
        budget: usize,
        mut edge: impl FnMut(Vec3, Vec3) -> Option<Vec3>,
    ) -> Outcome {
        for _ in 0..budget {
            let (index, direction) = match self.expanding {
                Some(current) => current,
                None => {
                    let best = self
                        .nodes
                        .iter()
                        .enumerate()
                        .filter(|(_, n)| !n.closed)
                        .min_by(|(_, a), (_, b)| {
                            (a.cost + a.at.distance(self.goal))
                                .total_cmp(&(b.cost + b.at.distance(self.goal)))
                        })
                        .map(|(i, _)| i);
                    let Some(index) = best else {
                        return Outcome::Failed;
                    };
                    self.nodes[index].closed = true;
                    (index, 0)
                }
            };
            let at = self.nodes[index].at;
            // A goal connection costs an edge probe too, and must reach the correct storey.
            if direction == 0 && flat(at - self.goal).length() <= SPACING * 1.5 {
                self.expanding = Some((index, 1));
                if let Some(end) = edge(at, self.goal).filter(|p| p.distance(self.goal) < 0.2) {
                    let mut path = VecDeque::from([end]);
                    let mut cursor = index;
                    while let Some(parent) = self.nodes[cursor].parent {
                        path.push_front(self.nodes[cursor].at);
                        cursor = parent;
                    }
                    return Outcome::Found(path);
                }
                continue;
            }
            // Slot zero is reserved for the optional goal probe.
            let d = if direction == 0 { 0 } else { direction - 1 };
            let (x, z) = DIRECTIONS[d];
            self.expanding = if d == 7 { None } else { Some((index, d + 2)) };
            let wanted = at + Vec3::new(x as f32 * SPACING, 0., z as f32 * SPACING);
            let Some(next) = edge(at, wanted).filter(|p| flat(*p - wanted).length() < 0.1) else {
                continue;
            };
            let cost = self.nodes[index].cost + at.distance(next);
            let key = self.key(next);
            if let Some(&existing) = self.cells.get(&key) {
                if cost < self.nodes[existing].cost {
                    self.nodes[existing].cost = cost;
                    self.nodes[existing].parent = Some(index);
                    self.nodes[existing].closed = false;
                }
            } else {
                if self.nodes.len() == MAX_NODES {
                    return Outcome::Failed;
                }
                self.cells.insert(key, self.nodes.len());
                self.nodes.push(Node {
                    at: next,
                    cost,
                    parent: Some(index),
                    closed: false,
                });
            }
        }
        Outcome::Pending
    }
}

#[derive(Default)]
struct Route {
    stuck: f32,
    cooldown: f32,
    radius: f32,
    goal: Vec3,
    path: VecDeque<Vec3>,
    search: Option<Search>,
}

#[derive(Default)]
pub(super) struct Navigation {
    routes: HashMap<Entity, Route>,
    queue: VecDeque<Entity>,
    map: Option<usize>,
}

impl Navigation {
    /// Up to eight requests receive eight edge probes each per frame, round-robin.
    pub fn advance(
        &mut self,
        dt: f32,
        map: usize,
        live: &[(Entity, Vec3, f32)],
        mut edge: impl FnMut(Vec3, Vec3, f32) -> Option<Vec3>,
    ) {
        if self.map != Some(map) {
            self.routes.clear();
            self.queue.clear();
            self.map = Some(map);
        }
        let live: HashSet<_> = live.iter().map(|(entity, _, _)| *entity).collect();
        self.routes.retain(|entity, route| {
            route.cooldown = (route.cooldown - dt).max(0.);
            live.contains(entity)
        });
        self.queue
            .retain(|entity| self.routes.get(entity).is_some_and(|r| r.search.is_some()));
        let turns = self.queue.len().min(REQUESTS_PER_FRAME);
        for _ in 0..turns {
            let Some(entity) = self.queue.pop_front() else {
                continue;
            };
            let route = self.routes.get_mut(&entity).unwrap();
            let Some(search) = route.search.as_mut() else {
                continue;
            };
            match search.advance(EDGE_BUDGET, |a, b| edge(a, b, route.radius)) {
                Outcome::Pending => self.queue.push_back(entity),
                Outcome::Found(path) => {
                    route.path = path;
                    route.search = None;
                }
                Outcome::Failed => {
                    route.search = None;
                    route.cooldown = 1.;
                }
            }
        }
    }

    pub fn waypoint(&mut self, entity: Entity, at: Vec3, goal: Vec3) -> Option<Vec3> {
        let route = self.routes.get_mut(&entity)?;
        if route.goal.distance(goal) > 6. {
            route.path.clear();
            route.search = None;
            route.stuck = 0.;
            route.goal = goal;
        }
        while route.path.front().is_some_and(|p| p.distance(at) < 0.5) {
            route.path.pop_front();
        }
        route.path.front().copied()
    }

    pub fn forget(&mut self, entity: Entity) {
        self.routes.remove(&entity);
    }

    pub fn observe(
        &mut self,
        entity: Entity,
        at: Vec3,
        goal: Vec3,
        radius: f32,
        blocked: bool,
        dt: f32,
    ) {
        if !blocked && !self.routes.contains_key(&entity) {
            return;
        }
        let route = self.routes.entry(entity).or_default();
        route.stuck = if blocked { route.stuck + dt } else { 0. };
        if route.stuck < 0.35 || route.search.is_some() || route.cooldown > 0. {
            return;
        }
        route.path.clear();
        // Remove stale queue entries before bounding pending work.
        self.queue.retain(|e| *e != entity);
        if self.queue.len() >= MAX_REQUESTS {
            return;
        }
        route.radius = radius;
        route.goal = goal;
        route.stuck = 0.;
        route.search = Some(Search::new(at, goal));
        self.queue.push_back(entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incremental_search_detours_without_crossing_a_wall() {
        let mut search = Search::new(Vec3::ZERO, Vec3::X * 8.);
        let mut probes = 0;
        let mut found = None;
        for _ in 0..300 {
            let before = probes;
            let outcome = search.advance(3, |a, b| {
                probes += 1;
                let crosses = (a.x - 3.) * (b.x - 3.) <= 0.;
                let z = if (b.x - a.x).abs() > 0.01 {
                    a.z + (b.z - a.z) * (3. - a.x) / (b.x - a.x)
                } else {
                    a.z
                };
                (!crosses || z.abs() > 3.).then_some(b)
            });
            assert!(probes - before <= 3);
            if let Outcome::Found(path) = outcome {
                found = Some(path);
                break;
            }
        }
        let path = found.expect("route around finite wall");
        assert!(path.iter().any(|p| p.z.abs() >= 4.));
        assert_eq!(path.back(), Some(&(Vec3::X * 8.)));
    }

    #[test]
    fn missing_support_and_wrong_storeys_do_not_produce_routes() {
        let mut missing = Search::new(Vec3::ZERO, Vec3::X * 8.);
        assert!(matches!(missing.advance(32, |_, _| None), Outcome::Failed));
        let mut stacked = Search::new(Vec3::ZERO, Vec3::new(0., 20., 0.));
        let mut outcome = Outcome::Pending;
        for _ in 0..MAX_NODES * 2 {
            outcome = stacked.advance(8, |_, b| Some(Vec3::new(b.x, 0., b.z)));
            if !matches!(outcome, Outcome::Pending) {
                break;
            }
        }
        assert!(matches!(outcome, Outcome::Failed));
    }
    #[test]
    fn scheduler_bounds_probes_and_discards_dead_or_changed_map_requests() {
        let mut nav = Navigation::default();
        let a = Entity::from_bits(1);
        let b = Entity::from_bits(2);
        let live = [(a, Vec3::ZERO, 0.6), (b, Vec3::ZERO, 1.2)];
        nav.advance(0.1, 0, &live, |_, _, _| unreachable!());
        for entity in [a, b] {
            nav.observe(entity, Vec3::ZERO, Vec3::X * 20., 0.6, true, 0.4);
        }
        let mut probes = 0;
        nav.advance(0.1, 0, &live, |_, b, _| {
            probes += 1;
            Some(b)
        });
        assert_eq!(probes, EDGE_BUDGET * 2);
        assert_eq!(nav.queue.front(), Some(&a));
        nav.advance(0.1, 0, &live[..1], |_, b, _| Some(b));
        assert!(!nav.routes.contains_key(&b));
        assert!(!nav.queue.contains(&b));
        nav.waypoint(a, Vec3::ZERO, Vec3::Z * 20.);
        assert!(nav.routes[&a].search.is_none());
        nav.advance(0.1, 1, &live, |_, _, _| unreachable!());
        assert!(nav.routes.is_empty());
        assert!(nav.queue.is_empty());
    }
    #[test]
    fn crowded_queue_services_eight_enemies_per_frame_without_starvation() {
        let mut nav = Navigation::default();
        let live: Vec<_> = (1..=64)
            .map(|id| (Entity::from_bits(id), Vec3::ZERO, 0.6))
            .collect();
        nav.advance(0.1, 0, &live, |_, _, _| unreachable!());
        for &(entity, at, radius) in &live {
            nav.observe(entity, at, Vec3::X * 8., radius, true, 0.4);
        }
        for _ in 0..120 {
            let mut probes = 0;
            nav.advance(1.0 / 60.0, 0, &live, |_, b, _| {
                probes += 1;
                Some(b)
            });
            assert!(probes <= REQUESTS_PER_FRAME * EDGE_BUDGET);
        }
        assert!(nav.queue.is_empty());
        assert!(live
            .iter()
            .all(|(entity, _, _)| !nav.routes[entity].path.is_empty()));
    }

    fn route_around_solid(collider: avian3d::prelude::Collider, transform: Transform, fence: bool) {
        use super::super::Floor;
        use avian3d::prelude::*;
        use benilla_world::collision::MoverTraceExclusions;
        use bevy::ecs::system::RunSystemOnce;
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            PhysicsPlugins::new(PostUpdate),
        ));
        app.init_asset::<Mesh>();
        app.init_resource::<MoverTraceExclusions>();
        app.world_mut().spawn((
            RigidBody::Static,
            Collider::trimesh(
                vec![
                    Vec3::new(-20., 0., -20.),
                    Vec3::new(20., 0., -20.),
                    Vec3::new(20., 0., 20.),
                    Vec3::new(-20., 0., 20.),
                ],
                vec![[0, 2, 1], [0, 3, 2]],
            ),
            Transform::default(),
        ));
        app.world_mut()
            .spawn((RigidBody::Static, collider, transform));
        app.finish();
        app.cleanup();
        app.update();
        app.world_mut()
            .run_system_once(move |floor: Floor| {
                let goal = Vec3::X * 8.;
                assert!(floor.can_spawn(goal, 0.6));
                if fence {
                    assert!(
                        !floor.can_spawn_from(Vec3::ZERO, goal, 0.6),
                        "do not spawn beyond an unverified fence connection"
                    );
                }
                assert!(floor.can_spawn_from(Vec3::X * 6., goal, 0.6));
                assert!(floor.travel(Vec3::ZERO, goal, 0.6).x < 7.5);
                let mut search = Search::new(Vec3::ZERO, goal);
                let mut found = None;
                for _ in 0..400 {
                    match search.advance(8, |a, b| {
                        let end = floor.travel(a, b, 0.6);
                        (flat(end - b).length() < 0.1 && floor.can_spawn(end, 0.6)).then_some(end)
                    }) {
                        Outcome::Found(path) => {
                            found = Some(path);
                            break;
                        }
                        Outcome::Failed => panic!("finite wall must have a route"),
                        Outcome::Pending => (),
                    }
                }
                let path = found.expect("bounded search completes");
                let mut at = Vec3::ZERO;
                for waypoint in path {
                    for _ in 0..100 {
                        if flat(waypoint - at).length() <= 0.05 {
                            break;
                        }
                        let delta = flat(waypoint - at).clamp_length_max(0.15);
                        let next = floor.travel(at, at + delta, 0.6);
                        assert!(
                            flat(next - at).length() > 0.01,
                            "route playback stalled at {at:?}"
                        );
                        assert!(floor.can_spawn(next, 0.6));
                        at = next;
                    }
                }
                assert!(at.distance(goal) < 0.1);
            })
            .unwrap();
    }
    #[test]
    fn route_edges_and_playback_use_real_body_sweeps() {
        route_around_solid(
            avian3d::prelude::Collider::cuboid(1., 4., 6.),
            Transform::from_xyz(4., 2., 0.),
            true,
        );
    }

    #[test]
    fn authored_chests_allow_a_walkable_detour() {
        let Some(data) = std::env::var_os("WOW_DATA").filter(|path| !path.is_empty()) else {
            eprintln!("SKIP authored chest routes: set WOW_DATA explicitly");
            return;
        };
        let mut chain = benilla_formats::open_chain(std::path::Path::new(&data)).unwrap();
        let catalog = benilla_formats::load_gameobject_catalog(&mut chain).unwrap();
        use super::super::campaign::{chest_display, ChestTier};
        for tier in [ChestTier::C, ChestTier::B, ChestTier::A] {
            let display = chest_display(tier);
            let path = catalog.model_path(display).unwrap();
            assert!(
                path.to_ascii_lowercase().contains("chest"),
                "{display}: {path}"
            );
            let hull = benilla_formats::load_m2_collision_hull(&mut chain, path).unwrap();
            assert!(!hull.is_empty(), "{path}");
            let vertices = hull
                .positions
                .iter()
                .map(|p| benilla_assets::coords::wow_to_bevy(*p) * 1.15)
                .collect();
            let collider = avian3d::prelude::Collider::trimesh(
                vertices,
                hull.indices.as_chunks::<3>().0.to_vec(),
            );
            route_around_solid(collider, Transform::from_xyz(4., 0., 0.), false);
        }
    }
}
