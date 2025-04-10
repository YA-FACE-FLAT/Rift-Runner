use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
struct HexCoord {
    q: i32,
    r: i32,
}

impl HexCoord {
    fn new(q: i32, r: i32) -> Self {
        HexCoord { q, r }
    }

    fn distance(&self, other: &HexCoord) -> i32 {
        ((self.q - other.q).abs() + (self.r - other.r).abs() + (self.q + self.r - other.q - other.r).abs()) / 2
    }

    fn neighbors(&self) -> Vec<HexCoord> {
        vec![
            HexCoord::new(self.q + 1, self.r),
            HexCoord::new(self.q - 1, self.r),
            HexCoord::new(self.q, self.r + 1),
            HexCoord::new(self.q, self.r - 1),
            HexCoord::new(self.q + 1, self.r - 1),
            HexCoord::new(self.q - 1, self.r + 1),
        ]
    }

    fn to_world(&self) -> Vec2 {
        let x = 50.0 * 3.0_f32.sqrt() * (self.q as f32 + self.r as f32 / 2.0);
        let y = 50.0 * 1.5 * self.r as f32;
        Vec2::new(x, y)
    }
}

#[derive(Resource)]
struct GameState {
    fields: HashMap<HexCoord, i32>,
    ethereals: HashMap<HexCoord, i32>,
    stasis_fields: HashMap<HexCoord, bool>,
    planets: Vec<Planet>,
    current_planet: usize,
    core_shard: HexCoord,
    rift_energy: i32,
    cycle: i32,
    foes_dissolved: i32,
    core_slowed: bool,
}

impl GameState {
    fn new() -> Self {
        let mut planets = Vec::new();
        planets.push(Planet::new(1, HexCoord::new(0, 0)));
        GameState {
            fields: HashMap::new(),
            ethereals: HashMap::new(),
            stasis_fields: HashMap::new(),
            planets,
            current_planet: 0,
            core_shard: HexCoord::new(0, 0),
            rift_energy: 50,
            cycle: 1,
            foes_dissolved: 0,
            core_slowed: false,
        }
    }

    fn get_field_cost(&self, field_type: i32) -> i32 {
        let mut rng = rand::thread_rng();
        match field_type {
            1 => 15 + self.cycle * 2 + rng.gen_range(-3..=3),
            3 => 40 + self.cycle * 5 + rng.gen_range(-4..=4),
            4 => 55 + self.cycle * 8 + rng.gen_range(-5..=5),
            _ => 0,
        }
    }

    fn is_on_planet(&self, coord: &HexCoord) -> bool {
        self.planets[self.current_planet].hexes.contains(coord)
    }

    fn jump_to_next_planet(&mut self) {
        let mut rng = rand::thread_rng();
        self.current_planet += 1;
        if self.current_planet >= self.planets.len() {
            let last_center = self.planets.last().unwrap().center;
            let new_center = HexCoord::new(last_center.q + rng.gen_range(2..=4), last_center.r + rng.gen_range(-2..=2));
            self.planets.push(Planet::new(self.cycle, new_center));
            self.core_shard = new_center;
            self.fields.clear();
            self.ethereals.clear();
            self.stasis_fields.clear();
            self.core_slowed = false;
        }
    }

    fn check_cycle_progression(&mut self) {
        if self.foes_dissolved >= self.cycle * 3 {
            self.cycle += 1;
            if self.cycle > 15 {
                // Handle victory condition visually later
                info!("Cosmic Victory Achieved!");
                std::process::exit(0);
            }
            self.jump_to_next_planet();
            let mut rng = rand::thread_rng();
            self.rift_energy += rng.gen_range(50..=100);
        }
    }
}

#[derive(Component)]
struct Planet {
    name: String,
    center: HexCoord,
    hexes: Vec<HexCoord>,
    foe_type: String,
    foe_strength: i32,
    effect: String,
}

impl Planet {
    fn new(cycle: i32, center: HexCoord) -> Self {
        let mut rng = rand::thread_rng();
        let (name, foe_type, effect) = match cycle {
            1..=5 => ("Slime Pits", "Gloopers", "Acid Pools: Foes spit acid"),
            6..=10 => ("Triad Moons", "Staregazers", "Jumpscare Shadows: Sudden spawns"),
            _ => ("Green Abyss", "Eyekings", "Triple Threat: Acid + Jumpscares"),
        };
        let hexes = vec![
            center,
            HexCoord::new(center.q + 1, center.r),
            HexCoord::new(center.q - 1, center.r),
            HexCoord::new(center.q, center.r + 1),
            HexCoord::new(center.q, center.r - 1),
            HexCoord::new(center.q + 1, center.r - 1),
            HexCoord::new(center.q - 1, center.r + 1),
        ];
        Planet {
            name: name.to_string(),
            center,
            hexes,
            foe_type: foe_type.to_string(),
            foe_strength: cycle * rng.gen_range(1..=3),
            effect: effect.to_string(),
        }
    }
}

#[derive(Component)]
struct CoreShard;

#[derive(Component)]
struct Field(i32);

#[derive(Component)]
struct Ethereal(i32);

#[derive(Component)]
struct Stasis;

#[derive(Component)]
struct HexTile;

#[derive(Resource)]
struct AnimationTimer(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rift Runner: Cosmic Conduits".into(),
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(GameState::new())
        .insert_resource(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            spawn_ethereals,
            update_game,
            handle_input,
            render_system,
            animate_system,
            update_ui,
        ))
        .run();
}

fn setup(mut commands: Commands, mut game_state: ResMut<GameState>, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // Spawn planet hexes with outline
    let planet = &game_state.planets[game_state.current_planet];
    for &hex in &planet.hexes {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.2, 0.2, 0.2),
                    custom_size: Some(Vec2::new(50.0, 50.0)),
                    ..default()
                },
                transform: Transform::from_translation(hex.to_world().extend(0.0)),
                ..default()
            },
            HexTile,
            hex,
        ));
    }

    // Spawn Core Shard
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLUE,
                custom_size: Some(Vec2::new(40.0, 40.0)),
                ..default()
            },
            transform: Transform::from_translation(game_state.core_shard.to_world().extend(1.0)),
            ..default()
        },
        CoreShard,
        game_state.core_shard,
    ));

    // UI setup
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Cycle: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 20.0,
                color: Color::YELLOW,
            }),
            TextSection::new(
                " | Rift Energy: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 20.0,
                color: Color::YELLOW,
            }),
            TextSection::new(
                " | Foes Dissolved: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 20.0,
                color: Color::YELLOW,
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        Name::new("UI"),
    ));
}

fn spawn_ethereals(mut game_state: ResMut<GameState>, mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let spawn_chance = 0.6 + (game_state.cycle as f64) / 15.0;
    let planet = &game_state.planets[game_state.current_planet];
    for &hex in &planet.hexes {
        if rng.gen_bool(spawn_chance) && !game_state.fields.contains_key(&hex) && !game_state.ethereals.contains_key(&hex) && hex != game_state.core_shard {
            let essence = planet.foe_strength + rng.gen_range(0..game_state.cycle);
            game_state.ethereals.insert(hex, essence);
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::GREEN,
                        custom_size: Some(Vec2::new(30.0, 30.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(hex.to_world().extend(2.0)),
                    ..default()
                },
                Ethereal(essence),
                hex,
            ));
            break;
        }
    }
    if planet.foe_type != "Gloopers" && rng.gen_bool(0.2) {
        let hex = planet.hexes[rng.gen_range(0..planet.hexes.len())];
        if !game_state.fields.contains_key(&hex) && !game_state.ethereals.contains_key(&hex) && hex != game_state.core_shard {
            let essence = planet.foe_strength + rng.gen_range(1..=3);
            game_state.ethereals.insert(hex, essence);
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::GREEN,
                        custom_size: Some(Vec2::new(30.0, 30.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(hex.to_world().extend(2.0)),
                    ..default()
                },
                Ethereal(essence),
                hex,
            ));
        }
    }
}

fn handle_input(
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera.single();
    let mut deploy_field = |field_type: i32| {
        if let Some(cursor_pos) = window.cursor_position()
            .and_then(|pos| camera.viewport_to_world_2d(camera_transform, pos))
        {
            let q = (cursor_pos.x / (50.0 * 3.0_f32.sqrt()) - cursor_pos.y / (50.0 * 1.5)).round() as i32;
            let r = (cursor_pos.y / (50.0 * 1.5)).round() as i32;
            let coord = HexCoord::new(q, r);
            let planet = &game_state.planets[game_state.current_planet];
            if planet.hexes.contains(&coord) && !game_state.fields.contains_key(&coord) {
                let cost = game_state.get_field_cost(field_type);
                if game_state.rift_energy >= cost {
                    game_state.fields.insert(coord, field_type);
                    game_state.rift_energy -= cost;
                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: match field_type {
                                    1 => Color::YELLOW,
                                    3 => Color::PURPLE,
                                    4 => Color::CYAN,
                                    _ => Color::WHITE,
                                },
                                custom_size: Some(Vec2::new(35.0, 35.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(coord.to_world().extend(1.0)),
                            ..default()
                        },
                        Field(field_type),
                        coord,
                    ));
                }
            }
        }
    };

    if mouse.just_pressed(MouseButton::Left) {
        if keys.pressed(KeyCode::KeyP) {
            deploy_field(1); // Pulse
        } else if keys.pressed(KeyCode::KeyW) {
            deploy_field(3); // Weave
        } else if keys.pressed(KeyCode::KeyT) {
            deploy_field(4); // Temporal
        }
    }

    if keys.just_pressed(KeyCode::KeyC) {
        // Simplified EntropyCore logic for demo
        let mut rng = rand::thread_rng();
        let energy = rng.gen_range(3..=7);
        game_state.rift_energy += energy;
        info!("Core activated: +{} rift energy!", energy);
    }
}

fn update_game(
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, Option<&Field>, Option<&Ethereal>, Option<&CoreShard>, &HexCoord)>,
    time: Res<Time>,
) {
    let mut rng = rand::thread_rng();
    let planet = &game_state.planets[game_state.current_planet];

    // Core Shard movement
    if !game_state.core_slowed && rng.gen_bool(0.3) {
        let neighbors = game_state.core_shard.neighbors();
        let valid_moves: Vec<HexCoord> = neighbors.into_iter()
            .filter(|n| game_state.is_on_planet(n) && !game_state.fields.contains_key(n) && !game_state.ethereals.contains_key(n))
            .collect();
        if !valid_moves.is_empty() {
            let new_pos = valid_moves[rng.gen_range(0..valid_moves.len())];
            game_state.core_shard = new_pos;
            for (entity, mut transform, _, _, core, _) in query.iter_mut() {
                if core.is_some() {
                    transform.translation = new_pos.to_world().extend(1.0);
                }
            }
        }
    } else if game_state.core_slowed && rng.gen_bool(0.5) {
        game_state.core_slowed = false;
    }

    // Field actions
    let mut to_remove = Vec::new();
    let mut fields_to_corrode = Vec::new();
    for (coord, &field_type) in &game_state.fields {
        match field_type {
            1 => {
                let radius = rng.gen_range(1..=2);
                for neighbor in coord.neighbors() {
                    if let Some(&essence) = game_state.ethereals.get(&neighbor) {
                        let damage = rng.gen_range(1..=4);
                        let new_essence = essence - damage;
                        if new_essence <= 0 {
                            to_remove.push(neighbor);
                        } else {
                            game_state.ethereals.insert(neighbor, new_essence);
                        }
                    }
                }
            }
            3 => {
                if rng.gen_bool(0.4) {
                    let energy = rng.gen_range(10..=25);
                    game_state.rift_energy += energy;
                }
            }
            4 => {
                if rng.gen_bool(0.3) {
                    let mut ethereal_moves = Vec::new();
                    for (eth_coord, &essence) in &game_state.ethereals {
                        if eth_coord.distance(coord) <= 2 {
                            let neighbors = eth_coord.neighbors();
                            let dest = neighbors[rng.gen_range(0..neighbors.len())];
                            if game_state.is_on_planet(&dest) && !game_state.fields.contains_key(&dest) && !game_state.ethereals.contains_key(&dest) {
                                ethereal_moves.push((*eth_coord, dest, essence));
                                break;
                            }
                        }
                    }
                    for &(from, to, essence) in ðereal_moves {
                        game_state.ethereals.remove(&from);
                        game_state.ethereals.insert(to, essence);
                        for (entity, mut transform, _, ethereal, _, &hex) in query.iter_mut() {
                            if ethereal.is_some() && hex == from {
                                transform.translation = to.to_world().extend(2.0);
                            }
                        }
                    }
                    if ethereal_moves.is_empty() && rng.gen_bool(0.2) {
                        let neighbors = coord.neighbors();
                        for n in neighbors {
                            if game_state.is_on_planet(&n) && !game_state.stasis_fields.contains_key(&n) {
                                game_state.stasis_fields.insert(n, true);
                                commands.spawn((
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: Color::GRAY,
                                            custom_size: Some(Vec2::new(25.0, 25.0)),
                                            ..default()
                                        },
                                        transform: Transform::from_translation(n.to_world().extend(1.5)),
                                        ..default()
                                    },
                                    Stasis,
                                    n,
                                ));
                                break;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    for coord in to_remove {
        game_state.ethereals.remove(&coord);
        game_state.foes_dissolved += 1;
        for (entity, _, _, ethereal, _, &hex) in query.iter() {
            if ethereal.is_some() && hex == coord {
                commands.entity(entity).despawn();
            }
        }
    }

    // Ethereal actions
    let mut ethereal_moves = Vec::new();
    for (coord, &essence) in &game_state.ethereals {
        if (planet.foe_type != "Staregazers" || rng.gen_bool(0.5)) && rng.gen_bool(0.25) {
            let targets = coord.neighbors();
            for target in targets {
                if target == game_state.core_shard {
                    game_state.core_slowed = true;
                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::RED,
                                custom_size: Some(Vec2::new(20.0, 20.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(target.to_world().extend(3.0)),
                            ..default()
                        },
                        TimerComponent(Timer::from_seconds(0.5, TimerMode::Once)),
                    ));
                    break;
                } else if game_state.fields.contains_key(&target) && rng.gen_bool(0.3) {
                    fields_to_corrode.push(target);
                    break;
                }
            }
        }

        let neighbors = coord.neighbors();
        let mut next = *coord;
        let mut min_dist = coord.distance(&game_state.core_shard);
        for n in &neighbors {
            if game_state.is_on_planet(n) && !game_state.fields.contains_key(n) && !game_state.ethereals.contains_key(n) {
                let dist = n.distance(&game_state.core_shard);
                if dist < min_dist {
                    min_dist = dist;
                    next = *n;
                }
            }
        }
        if next != *coord {
            if game_state.stasis_fields.contains_key(&next) {
                game_state.stasis_fields.remove(&next);
                for (entity, _, _, _, _, &hex) in query.iter() {
                    if hex == next {
                        commands.entity(entity).despawn();
                    }
                }
            } else {
                ethereal_moves.push((*coord, next, essence));
            }
        }
    }
    for (from, to, essence) in ethereal_moves {
        game_state.ethereals.remove(&from);
        game_state.ethereals.insert(to, essence);
        for (entity, mut transform, _, ethereal, _, &hex) in query.iter_mut() {
            if ethereal.is_some() && hex == from {
                transform.translation = to.to_world().extend(2.0);
            }
        }
    }
    for coord in fields_to_corrode {
        game_state.fields.remove(&coord);
        for (entity, _, field, _, _, &hex) in query.iter() {
            if field.is_some() && hex == coord {
                commands.entity(entity).despawn();
            }
        }
    }

    // Planet effects
    match planet.foe_type.as_str() {
        "Gloopers" => if rng.gen_bool(0.2) {
            let boost = rng.gen_range(10..=20);
            game_state.rift_energy += boost;
        },
        "Staregazers" => if rng.gen_bool(0.15) {
            let mut moved = false;
            for (coord, field_type) in game_state.fields.clone() {
                let neighbors = coord.neighbors();
                let dest = neighbors[rng.gen_range(0..neighbors.len())];
                if game_state.is_on_planet(&dest) && !game_state.fields.contains_key(&dest) && !game_state.ethereals.contains_key(&dest) {
                    game_state.fields.remove(&coord);
                    game_state.fields.insert(dest, field_type);
                    for (entity, mut transform, field, _, _, &hex) in query.iter_mut() {
                        if field.is_some() && hex == coord {
                            transform.translation = dest.to_world().extend(1.0);
                        }
                    }
                    moved = true;
                    break;
                }
            }
            if !moved { spawn_ethereals(game_state.reborrow(), commands.reborrow()); }
        },
        "Eyekings" => if rng.gen_bool(0.25) { spawn_ethereals(game_state.reborrow(), commands.reborrow()); },
        _ => {}
    }

    game_state.check_cycle_progression();
}

fn render_system(
    game_state: Res<GameState>,
    mut query: Query<(&mut Sprite, &HexCoord, Option<&CoreShard>, Option<&Field>, Option<&Ethereal>)>,
) {
    for (mut sprite, _, core, field, ethereal) in query.iter_mut() {
        if core.is_some() {
            sprite.color = if game_state.core_slowed { Color::GRAY } else { Color::BLUE };
        } else if let Some(&Field(field_type)) = field {
            sprite.color = match field_type {
                1 => Color::YELLOW,
                3 => Color::PURPLE,
                4 => Color::CYAN,
                _ => Color::WHITE,
            };
        } else if ethereal.is_some() {
            sprite.color = Color::GREEN;
        }
    }
}

fn animate_system(
    time: Res<Time>,
    mut timer: ResMut<AnimationTimer>,
    mut query: Query<(Entity, &mut Transform, Option<&Ethereal>, Option<&TimerComponent>), With<Sprite>>,
    mut commands: Commands,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for (entity, mut transform, ethereal, timer_opt) in query.iter_mut() {
            if ethereal.is_some() {
                transform.scale = Vec3::new(1.0 + 0.1 * time.elapsed_seconds().sin(), 1.0 + 0.1 * time.elapsed_seconds().sin(), 1.0);
            }
            if let Some(timer) = timer_opt {
                if timer.0.tick(time.delta()).just_finished() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

#[derive(Component)]
struct TimerComponent(Timer);

fn update_ui(game_state: Res<GameState>, mut query: Query<&mut Text, With<Name>>) {
    for mut text in query.iter_mut() {
        if text.entity_name() == Some(&Name::new("UI")) {
            text.sections[1].value = game_state.cycle.to_string();
            text.sections[3].value = game_state.rift_energy.to_string();
            text.sections[5].value = game_state.foes_dissolved.to_string();
        }
    }
}
