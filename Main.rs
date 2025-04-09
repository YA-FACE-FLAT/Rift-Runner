use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

#[derive(Component, Clone, Copy, Debug)]
struct HexCoord {
    q: i32,
    r: i32,
}

impl HexCoord {
    fn new(q: i32, r: i32) -> Self {
        HexCoord { q, r }
    }

    fn to_world(&self) -> Vec2 {
        let x = 3.0_f32.sqrt() * 30.0 * (self.q as f32 + self.r as f32 / 2.0);
        let y = 1.5 * 30.0 * self.r as f32;
        Vec2::new(x, y)
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
}

#[derive(Resource)]
struct GameState {
    fields: HashMap<HexCoord, i32>, // 1=pulse, 3=weave, 4=temporal
    ethereals: HashMap<HexCoord, i32>,
    core_shard: HexCoord,
    rift_energy: i32,
    cycle: i32,
    foes_dissolved: i32,
    core_slowed: bool,
    planet: Planet,
    spawn_timer: Timer,
}

#[derive(Component)]
struct CoreShard;

#[derive(Component)]
struct EnergyField(i32);

#[derive(Component)]
struct Ethereal {
    essence: i32,
    acid_timer: Timer,
}

#[derive(Component)]
struct HexTile;

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
            1..=5 => ("Slime Pits", "Gloopers", "Acid Pools"),
            6..=10 => ("Triad Moons", "Staregazers", "Jumpscare Shadows"),
            _ => ("Green Abyss", "Eyekings", "Triple Threat"),
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

fn setup(mut commands: Commands, mut game_state: ResMut<GameState>) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Spawn hexagonal tiles
    for hex in &game_state.planet.hexes {
        let pos = hex.to_world();
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.2, 0.2, 0.2),
                    custom_size: Some(Vec2::new(50.0, 50.0)),
                    ..default()
                },
                transform: Transform::from_translation(pos.extend(0.0)),
                ..default()
            },
            HexTile,
            *hex,
        ));
    }

    // Spawn Core Shard
    let core_pos = game_state.core_shard.to_world();
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.0, 0.8, 1.0),
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            transform: Transform::from_translation(core_pos.extend(1.0)),
            ..default()
        },
        CoreShard,
        game_state.core_shard,
    ));
}

fn spawn_field(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    if let Some(cursor_pos) = window.cursor_position() {
        if keyboard_input.just_pressed(KeyCode::KeyP) && game_state.rift_energy >= 15 {
            let world_pos = cursor_pos - Vec2::new(window.width() / 2.0, window.height() / 2.0);
            let hex_q = (world_pos.x / (3.0_f32.sqrt() * 30.0) - world_pos.y / (3.0 * 30.0)).round() as i32;
            let hex_r = (world_pos.y / (1.5 * 30.0)).round() as i32;
            let hex = HexCoord::new(hex_q, hex_r);
            if game_state.planet.hexes.contains(&hex) && !game_state.fields.contains_key(&hex) && !game_state.ethereals.contains_key(&hex) {
                game_state.fields.insert(hex, 1);
                game_state.rift_energy -= 15;
                let pos = hex.to_world();
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(1.0, 0.0, 0.0),
                            custom_size: Some(Vec2::new(25.0, 25.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(pos.extend(2.0)),
                        ..default()
                    },
                    EnergyField(1),
                    hex,
                ));
                info!("Pulse Field deployed at ({}, {})", hex.q, hex.r);
            }
        }
        // Add similar logic for Weave (W) and Temporal (T) fields if desired
    }
}

fn update_game(
    mut commands: Commands,
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut core_query: Query<(&mut Transform, &HexCoord), With<CoreShard>>,
    mut field_query: Query<(&mut Transform, &EnergyField, &HexCoord)>,
    mut ethereal_query: Query<(Entity, &mut Transform, &mut Ethereal, &HexCoord)>,
) {
    let mut rng = rand::thread_rng();

    // Spawn Ethereals
    game_state.spawn_timer.tick(time.delta());
    if game_state.spawn_timer.finished() {
        let hex = game_state.planet.hexes[rng.gen_range(0..game_state.planet.hexes.len())];
        if !game_state.fields.contains_key(&hex) && !game_state.ethereals.contains_key(&hex) && hex != game_state.core_shard {
            let essence = game_state.planet.foe_strength + rng.gen_range(0..game_state.cycle);
            game_state.ethereals.insert(hex, essence);
            let pos = hex.to_world();
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.0, 1.0, 0.0),
                        custom_size: Some(Vec2::new(20.0, 20.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(pos.extend(3.0)),
                    ..default()
                },
                Ethereal {
                    essence,
                    acid_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                },
                hex,
            ));
            info!("{} spawned at ({}, {}) with {} essence", game_state.planet.foe_type, hex.q, hex.r, essence);

            // Jumpscare chance
            if game_state.planet.foe_type != "Gloopers" && rng.gen_bool(0.2) {
                info!("!!! JUMPSCARE !!! {} at ({}, {})", game_state.planet.foe_type, hex.q, hex.r);
            }
        }
        game_state.spawn_timer.reset();
    }

    // Move Core Shard
    if !game_state.core_slowed && rng.gen_bool(0.1) {
        let (mut transform, core_hex) = core_query.single_mut();
        let neighbors = core_hex.neighbors();
        let valid_moves: Vec<HexCoord> = neighbors.into_iter()
            .filter(|n| game_state.planet.hexes.contains(n) && !game_state.fields.contains_key(n) && !game_state.ethereals.contains_key(n))
            .collect();
        if !valid_moves.is_empty() {
            game_state.core_shard = valid_moves[rng.gen_range(0..valid_moves.len())];
            transform.translation = game_state.core_shard.to_world().extend(1.0);
            info!("Core Shard moved to ({}, {})", game_state.core_shard.q, game_state.core_shard.r);
        }
    }

    // Update Fields and Ethereals
    for (mut transform, field, hex) in field_query.iter_mut() {
        match field.0 {
            1 => { // Pulse Field
                for (entity, eth_transform, mut ethereal, eth_hex) in ethereal_query.iter_mut() {
                    if hex.distance(eth_hex) <= 2 {
                        let damage = rng.gen_range(1..=4);
                        ethereal.essence -= damage;
                        if ethereal.essence <= 0 {
                            commands.entity(entity).despawn();
                            game_state.ethereals.remove(eth_hex);
                            game_state.foes_dissolved += 1;
                            info!("Ethereal dissolved at ({}, {})", eth_hex.q, eth_hex.r);
                        }
                    }
                }
            }
            // Add Weave and Temporal field logic here if expanded
            _ => {}
        }
    }

    // Ethereal movement and acid spit
    for (entity, mut transform, mut ethereal, hex) in ethereal_query.iter_mut() {
        let neighbors = hex.neighbors();
        let mut next = *hex;
        let mut min_dist = hex.distance(&game_state.core_shard);
        for n in &neighbors {
            if game_state.planet.hexes.contains(n) && !game_state.fields.contains_key(n) {
                let dist = n.distance(&game_state.core_shard);
                if dist < min_dist {
                    min_dist = dist;
                    next = *n;
                }
            }
        }
        if next != *hex && !game_state.ethereals.contains_key(&next) {
            game_state.ethereals.remove(hex);
            game_state.ethereals.insert(next, ethereal.essence);
            transform.translation = next.to_world().extend(3.0);
        }

        // Acid spit
        ethereal.acid_timer.tick(time.delta());
        if ethereal.acid_timer.finished() && (game_state.planet.foe_type != "Staregazers" || rng.gen_bool(0.5)) && rng.gen_bool(0.25) {
            if hex.distance(&game_state.core_shard) <= 2 {
                game_state.core_slowed = true;
                info!("{}: Acid spit! Core slowed!", game_state.planet.foe_type);
            } else {
                for (f_transform, field, f_hex) in field_query.iter() {
                    if hex.distance(f_hex) <= 2 && rng.gen_bool(0.3) {
                        commands.entity(f_transform.id()).despawn();
                        game_state.fields.remove(f_hex);
                        info!("{}: Acid spit corroded field at ({}, {})!", game_state.planet.foe_type, f_hex.q, f_hex.r);
                        break;
                    }
                }
            }
            ethereal.acid_timer.reset();
        }
    }

    // Simple cycle progression (for prototype)
    if game_state.foes_dissolved >= game_state.cycle * 3 {
        game_state.cycle += 1;
        info!("Cycle {} begins!", game_state.cycle);
    }

    // Game over check
    if game_state.ethereals.contains_key(&game_state.core_shard) {
        panic!("Game Over! Core Shard consumed!");
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GameState {
            fields: HashMap::new(),
            ethereals: HashMap::new(),
            core_shard: HexCoord::new(0, 0),
            rift_energy: 50,
            cycle: 1,
            foes_dissolved: 0,
            core_slowed: false,
            planet: Planet::new(1, HexCoord::new(0, 0)),
            spawn_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_field, update_game))
        .run();
}
