use rand::Rng;
use std::io;
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
}

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

    fn contains(&self, coord: &HexCoord) -> bool {
        self.hexes.contains(coord)
    }
}

struct RiftRunner {
    fields: HashMap<HexCoord, i32>,         // 1=pulse, 3=weave, 4=temporal
    ethereals: HashMap<HexCoord, i32>,
    stasis_fields: HashMap<HexCoord, bool>,
    planets: Vec<Planet>,
    current_planet: usize,
    core_shard: HexCoord,
    rift_energy: i32,
    cycle: i32,
    foes_dissolved: i32,
    core_slowed: bool,                      // Acid slow effect
}

impl RiftRunner {
    fn new() -> Self {
        let mut planets = Vec::new();
        planets.push(Planet::new(1, HexCoord::new(0, 0)));
        RiftRunner {
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

    fn deploy_field(&mut self, q: i32, r: i32, field_type: i32) {
        let coord = HexCoord::new(q, r);
        let cost = self.get_field_cost(field_type);
        let mut rng = rand::thread_rng();
        if self.rift_energy >= cost && !self.fields.contains_key(&coord) && self.is_on_planet(&coord) {
            self.fields.insert(coord, field_type);
            self.rift_energy -= cost;
            let dialogue = match field_type {
                1 => format!("Pulse Field: 'Blasting at {} volts!'", rng.gen_range(10..=20)),
                3 => format!("Weave Field: 'Knitting {} rift strands!'", rng.gen_range(15..=35)),
                4 => format!("Temporal Field: 'Twisting {} seconds!'", rng.gen_range(60..=120)),
                _ => "Unknown".to_string(),
            };
            println!("{} Cost: {}", dialogue, cost);
        } else {
            println!("Invalid deployment! Energy: {} Needed: {} On Planet: {}", self.rift_energy, cost, self.is_on_planet(&coord));
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

    fn spawn_ethereal(&mut self) {
        let mut rng = rand::thread_rng();
        let spawn_chance = 0.6 + (self.cycle as f64) / 15.0;
        let planet = &self.planets[self.current_planet];
        for hex in &planet.hexes {
            if rng.gen_bool(spawn_chance) && !self.fields.contains_key(hex) && !self.ethereals.contains_key(hex) && *hex != self.core_shard {
                let essence = planet.foe_strength + rng.gen_range(0..self.cycle);
                self.ethereals.insert(*hex, essence);
                println!("{}: 'Grrr! Three eyes on you at ({}, {}) with {} essence!'", planet.foe_type, hex.q, hex.r, essence);
                break;
            }
        }
        // Jumpscare spawn
        if planet.foe_type != "Gloopers" && rng.gen_bool(0.2) {
            let hex = planet.hexes[rng.gen_range(0..planet.hexes.len())];
            if !self.fields.contains_key(&hex) && !self.ethereals.contains_key(&hex) && hex != self.core_shard {
                let essence = planet.foe_strength + rng.gen_range(1..=3);
                self.ethereals.insert(hex, essence);
                println!("!!! JUMPSCARE !!! {}: 'BOO! Surprise at ({}, {})!'", planet.foe_type, hex.q, hex.r);
                println!("Core Shard: 'AAAH! Nearly dropped my circuits!'");
            }
        }
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
            println!("Core Shard: 'Woo-hoo! Bouncing to {}!'", self.planets[self.current_planet].name);
        }
    }

    fn is_on_planet(&self, coord: &HexCoord) -> bool {
        self.planets[self.current_planet].contains(coord)
    }

    fn display(&self) {
        let planet = &self.planets[self.current_planet];
        println!(
            "Cycle: {} | Planet: {} | Rift Energy: {} | Foes Dissolved: {}",
            self.cycle, planet.name, self.rift_energy, self.foes_dissolved
        );
        println!(
            "Costs: P:{} W:{} T:{} | Effect: {}",
            self.get_field_cost(1), self.get_field_cost(3), self.get_field_cost(4), planet.effect
        );

        let mut min_q = planet.center.q - 2;
        let mut max_q = planet.center.q + 2;
        let mut min_r = planet.center.r - 2;
        let mut max_r = planet.center.r + 2;

        for r in min_r..=max_r {
            let offset = if r % 2 == 0 { "   " } else { "" };
            print!("{}", offset);
            for q in min_q..=max_q {
                let coord = HexCoord::new(q, r);
                let display = if coord == self.core_shard {
                    if self.core_slowed { "CS*" } else { "CS " }.to_string()
                } else if let Some(&field_type) = self.fields.get(&coord) {
                    match field_type {
                        1 => "P  ",
                        3 => "W  ",
                        4 => "T  ",
                        _ => "?  ",
                    }.to_string()
                } else if let Some(&essence) = self.ethereals.get(&coord) {
                    format!("E{:1} ", essence.min(9))
                } else if self.stasis_fields.contains_key(&coord) {
                    "S  ".to_string()
                } else if planet.contains(&coord) {
                    ".  ".to_string()
                } else {
                    "   ".to_string()
                };
                print!("{}", display);
            }
            println!();
        }
        println!();
    }

    fn update(&mut self) {
        let mut rng = rand::thread_rng();
        let planet = &self.planets[self.current_planet];

        // Move Core Shard
        if !self.core_slowed && rng.gen_bool(0.3) {
            let neighbors = self.core_shard.neighbors();
            let valid_moves: Vec<HexCoord> = neighbors.into_iter()
                .filter(|n| self.is_on_planet(n) && !self.fields.contains_key(n) && !self.ethereals.contains_key(n))
                .collect();
            if !valid_moves.is_empty() {
                self.core_shard = valid_moves[rng.gen_range(0..valid_moves.len())];
                println!("Core Shard: 'Zip! Now at ({}, {})!'", self.core_shard.q, self.core_shard.r);
            }
        } else if self.core_slowed && rng.gen_bool(0.5) {
            self.core_slowed = false;
            println!("Core Shard: 'Phew, acid’s wearing off!'");
        }

        // Field actions
        let mut to_remove = Vec::new();
        let mut fields_to_corrode = Vec::new();
        for (coord, field_type) in &self.fields {
            match field_type {
                1 => {
                    let radius = rng.gen_range(1..=2);
                    for neighbor in coord.neighbors() {
                        if self.ethereals.contains_key(&neighbor) && neighbor.distance(coord) <= radius {
                            let damage = rng.gen_range(1..=4);
                            let essence = self.ethereals.get_mut(&neighbor).unwrap();
                            *essence -= damage;
                            println!("Pulse Field: 'Zapped {} essence at ({}, {})!'", damage, neighbor.q, neighbor.r);
                            if *essence <= 0 {
                                to_remove.push(neighbor);
                            }
                        }
                    }
                }
                3 => {
                    if rng.gen_bool(0.4) {
                        let energy = rng.gen_range(10..=25);
                        self.rift_energy += energy;
                        println!("Weave Field: 'Wove {} rift energy!'", energy);
                        if rng.gen_bool(0.45) {
                            for hex in &planet.hexes {
                                if self.ethereals.contains_key(hex) && rng.gen_bool(0.3) {
                                    let essence = self.ethereals.get_mut(hex).unwrap();
                                    *essence -= 1;
                                    println!("Weave Field: 'Disrupted at ({}, {})!'", hex.q, hex.r);
                                    if *essence <= 0 {
                                        to_remove.push(*hex);
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
                4 => {
                    if rng.gen_bool(0.3) {
                        let mut moved = false;
                        for (eth_coord, essence) in self.ethereals.iter_mut() {
                            if eth_coord.distance(coord) <= 2 {
                                let neighbors = eth_coord.neighbors();
                                let dest = neighbors[rng.gen_range(0..neighbors.len())];
                                if self.is_on_planet(&dest) && !self.fields.contains_key(&dest) && !self.ethereals.contains_key(&dest) {
                                    let essence_val = *essence;
                                    self.ethereals.remove(eth_coord);
                                    self.ethereals.insert(dest, essence_val);
                                    println!("Temporal Field: 'Shifted to ({}, {})!'", dest.q, dest.r);
                                    moved = true;
                                    break;
                                }
                            }
                        }
                        if !moved && rng.gen_bool(0.2) {
                            let neighbors = coord.neighbors();
                            for n in neighbors {
                                if self.is_on_planet(&n) && !self.stasis_fields.contains_key(&n) {
                                    self.stasis_fields.insert(n, true);
                                    println!("Temporal Field: 'Stasis at ({}, {})!'", n.q, n.r);
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
            self.ethereals.remove(&coord);
            self.foes_dissolved += 1;
            println!("{}: 'Melted at ({}, {})!'", planet.foe_type, coord.q, coord.r);
        }

        // Ethereal actions (movement + acid spit)
        let mut ethereal_moves = Vec::new();
        for (coord, essence) in &self.ethereals {
            // Acid spit
            if (planet.foe_type != "Staregazers" || rng.gen_bool(0.5)) && rng.gen_bool(0.25) {
                let targets = coord.neighbors();
                for target in targets {
                    if target == self.core_shard {
                        self.core_slowed = true;
                        println!("{}: 'Acid spit! Core slowed at ({}, {})!'", planet.foe_type, target.q, target.r);
                        println!("Core Shard: 'Ow! Sticky goo!'");
                        break;
                    } else if self.fields.contains_key(&target) && rng.gen_bool(0.3) {
                        fields_to_corrode.push(target);
                        println!("{}: 'Acid spit corrodes field at ({}, {})!'", planet.foe_type, target.q, target.r);
                        break;
                    }
                }
            }

            // Movement
            let neighbors = coord.neighbors();
            let mut next = *coord;
            let mut min_dist = coord.distance(&self.core_shard);
            for n in &neighbors {
                if self.is_on_planet(n) && !self.fields.contains_key(n) {
                    let dist = n.distance(&self.core_shard);
                    if dist < min_dist {
                        min_dist = dist;
                        next = *n;
                    }
                }
            }
            if next != *coord {
                if self.stasis_fields.contains_key(&next) {
                    self.stasis_fields.remove(&next);
                    println!("{}: 'Stasis popped at ({}, {})!'", planet.foe_type, next.q, next.r);
                } else if !self.ethereals.contains_key(&next) {
                    ethereal_moves.push((*coord, next, *essence));
                }
            }
        }
        for (from, to, essence) in ethereal_moves {
            self.ethereals.remove(&from);
            self.ethereals.insert(to, essence);
        }
        for coord in fields_to_corrode {
            self.fields.remove(&coord);
            println!("Field: 'Argh! Corroded away!'");
        }

        // Planet effect
        match planet.foe_type.as_str() {
            "Gloopers" => if rng.gen_bool(0.2) {
                let boost = rng.gen_range(10..=20);
                self.rift_energy += boost;
                println!("{}: 'Slimy pools gift +{} energy!'", planet.name, boost);
            },
            "Staregazers" => if rng.gen_bool(0.15) {
                let mut moved = false;
                for (coord, field_type) in self.fields.clone() {
                    let neighbors = coord.neighbors();
                    let dest = neighbors[rng.gen_range(0..neighbors.len())];
                    if self.is_on_planet(&dest) && !self.fields.contains_key(&dest) && !self.ethereals.contains_key(&dest) {
                        self.fields.remove(&coord);
                        self.fields.insert(dest, field_type);
                        println!("{}: 'Shadows shift field to ({}, {})!'", planet.name, dest.q, dest.r);
                        moved = true;
                        break;
                    }
                }
                if !moved { self.spawn_ethereal(); }
            },
            "Eyekings" => if rng.gen_bool(0.25) { self.spawn_ethereal(); },
            _ => {}
        }

        self.check_cycle_progression();
    }

    fn check_cycle_progression(&mut self) {
        if self.foes_dissolved >= self.cycle * 3 {
            self.cycle += 1;
            if self.cycle > 15 {
                self.display_cosmic_victory();
                std::process::exit(0);
            }
            self.jump_to_next_planet();
            println!("Overmind: 'Cycle {}! Don’t blink, Runner!'", self.cycle);
            self.rift_energy += rng.gen_range(50..=100);
            println!("Core Shard: 'Yikes, that was gooey! +{} energy!'", self.rift_energy);
        }
    }

    fn display_cosmic_victory(&self) {
        println!("==========================================");
        println!("       COSMIC VICTORY - CYCLE 15          ");
        println!("  The Three-Eyed Menace Is Dissolved!     ");
        println!("  Foes Dissolved: {}", self.foes_dissolved);
        println!("  Final Rift Energy: {}", self.rift_energy);
        println!("  Overmind: 'You’ve got three eyes of your own now, champ!'");
        println!("==========================================");
    }

    fn is_game_over(&self) -> bool {
        if self.ethereals.contains_key(&self.core_shard) {
            println!("Core Shard: 'Gah! Eaten by green goo!'");
            true
        } else {
            false
        }
    }

    fn add_rift_energy(&mut self, energy: i32) {
        self.rift_energy += energy;
        if energy > 0 {
            println!("Core Shard: 'Zap! +{} rift juice!'", energy);
        }
    }
}

struct EntropyCore {
    quanta: [String; 5],
}

impl EntropyCore {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let types = vec!["alpha", "beta", "gamma", "delta", "omega"];
        let mut quanta = ["".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()];
        for i in 0..5 {
            quanta[i] = types[rng.gen_range(0..types.len())].to_string();
        }
        EntropyCore { quanta }
    }

    fn play(&mut self) -> i32 {
        let mut rng = rand::thread_rng();
        println!("Entropy Core:");
        for (i, quantum) in self.quanta.iter().enumerate() {
            print!("{}:{} ", i, quantum);
        }
        println!("\nEnter two indices (0-4) to entangle quanta, or 'skip'");
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        let input = input.trim();

        if input == "skip" {
            return 0;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() != 2 {
            println!("Invalid input");
            return 0;
        }

        let idx1: usize = parts[0].parse().unwrap_or(5);
        let idx2: usize = parts[1].parse().unwrap_or(5);

        if idx1 < 5 && idx2 < 5 && idx1 != idx2 && self.quanta[idx1] == self.quanta[idx2] {
            println!("Entanglement: '{} entangled with {}% coherence!'", self.quanta[idx1], rng.gen_range(75..=100));
            self.quanta[idx1] = "XX".to_string();
            self.quanta[idx2] = "XX".to_string();
            let energy = rng.gen_range(3..=7);
            let bonus = if rng.gen_bool(0.25) { rng.gen_range(1..=4) } else { 0 };
            println!("Bonus energy: {}", bonus);
            return energy + bonus;
        } else {
            println!("Entanglement failed: No coherence!");
            return 0;
        }
    }
}

fn main() {
    let mut runner = RiftRunner::new();
    let mut rng = rand::thread_rng();

    println!("Welcome to Rift Runner: Cosmic Conduits");
    println!("Commands: p/w/t q r (field types), core, quit");
    println!("P=Pulse Field, W=Weave Field, T=Temporal Field");
    println!("CS=Core Shard (CS*=Slowed), S=Stasis, E#=Ethereal (essence), .=Empty");
    println!("Overmind: 'Runner, those three-eyed freaks are spitting mad!'");

    while !runner.is_game_over() {
        runner.display();

        if rng.gen_bool(0.7) {
            runner.spawn_ethereal();
        }

        println!("Enter command: ");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        let input = input.trim();

        if input == "quit" {
            break;
        } else if input == "core" {
            let mut core = EntropyCore::new();
            let energy = core.play();
            runner.add_rift_energy(energy);
            println!("Gained {} rift energy!", energy);
        } else {
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() == 3 {
                let field_type = match parts[0] {
                    "p" => 1,
                    "w" => 3,
                    "t" => 4,
                    _ => 0,
                };
                if field_type != 0 {
                    if let (Ok(q), Ok(r)) = (parts[1].parse::<i32>(), parts[2].parse::<i32>()) {
                        runner.deploy_field(q, r, field_type);
                    } else {
                        println!("Invalid coordinates");
                    }
                } else {
                    println!("Invalid field type");
                }
            } else {
                println!("Invalid command");
            }
        }

        runner.update();
    }

    if runner.is_game_over() {
        println!("Game Over! The cosmos is goo!");
        println!("GAME OVER!");
    }
}
