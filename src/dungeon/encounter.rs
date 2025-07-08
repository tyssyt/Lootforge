use crate::{combat::enemy::EnemyType, prelude::*};

use EncounterDifficulty::*;

#[apply(UnitEnum)]
pub enum EncounterDifficulty {
    Easy,
    Medium,
    Hard,
    Boss
}

#[derive(Debug)]
pub struct Encounter {
    pub difficulty: EncounterDifficulty,
    pub enemies: Vec<EnemyType>,
}

impl Encounter {
    pub fn generate_floor(depth: u16, rng: &mut impl Rng) -> Vec<Self> {
        if depth < 10 {
            Self::genrate_early_game(depth, rng)
        } else if depth % 10 == 0 {
            todo!("generate Boss encounter")
        } else {
            Self::generate_normal(depth, rng)
        }
    }

    fn genrate_early_game(depth: u16, rng: &mut impl Rng) -> Vec<Self> {
        match depth {
            1 => vec![
                Self { difficulty: Easy, enemies: vec![EnemyType::Small; 1] },
                Self { difficulty: Easy, enemies: vec![EnemyType::Small; 2] },
                Self { difficulty: Easy, enemies: vec![EnemyType::Small; 3] },
            ],
            2 => vec![
                Self { difficulty: Easy, enemies: vec![EnemyType::Small; 2] },
                Self { difficulty: Easy, enemies: vec![EnemyType::Small; 3] },
                Self { difficulty: Easy, enemies: vec![EnemyType::Small; 4] },
            ],
            3 => vec![
                Self { difficulty: Easy, enemies: vec![EnemyType::Medium; 1] },
                Self { difficulty: Easy, enemies: vec![EnemyType::Medium, EnemyType::Small, EnemyType::Small] },
                Self { difficulty: Easy, enemies: vec![EnemyType::Medium; 2] },
            ],
            4 => vec![
                Self { difficulty: Easy, enemies: vec![EnemyType::Small; 6] },
                Self { difficulty: Easy, enemies: vec![EnemyType::Tank] },
                Self { difficulty: Medium, enemies: vec![EnemyType::Small, EnemyType::Small, EnemyType::Medium, EnemyType::Medium] },
            ],
            5 => vec![
                Self { difficulty: Easy, enemies: vec![EnemyType::Dps, EnemyType::Dps] },
                Self { difficulty: Easy, enemies: vec![EnemyType::Tank, EnemyType::Small, EnemyType::Small, EnemyType::Small] },
                Self { difficulty: Medium, enemies: vec![EnemyType::Tank, EnemyType::Medium, EnemyType::Small, EnemyType::Dps] },
            ],
            6 | 7 => vec![Self::easy_encounter(rng), Self::easy_encounter(rng), Self::medium_encounter(rng)],
            8 | 9 => vec![Self::easy_encounter(rng), Self::medium_encounter(rng), Self::medium_encounter(rng)],
            _ => panic!(),
        }
    }

    fn generate_normal(depth: u16, rng: &mut impl Rng) -> Vec<Self> {
        let battles = 3 + (depth / 10);
        let (mut easy, mut medium, mut hard) = (battles / 3, battles / 3, battles / 3);

        let remainder = battles % 3;
        [Easy, Medium, Hard].choose_multiple(rng, remainder as usize)
            .for_each(|d| match d {
                Easy => easy += 1,
                Medium => medium += 1,
                Hard => hard += 1,
                Boss => panic!(),
            });
        
        let mut encounters: Vec<Option<EncounterDifficulty>> = vec![None; battles as usize];
        encounters[0] = Some(Easy);
        easy -= 1;
        *encounters.last_mut().unwrap() = Some(Hard);
        hard -= 1;

        let mut open_indices: Vec<_> = (1..encounters.len()-2).collect();
        while hard > 0 {
            let idx = *open_indices.choose(rng).unwrap();
            open_indices.retain(|&i| i < idx -1 || i > idx +1);

            encounters[idx] = Some(Hard);
            hard -= 1;
        }

        let encounters: Vec<_> = encounters.into_iter().map(|e| e.unwrap_or_else(|| {
            if rng.random_bool(easy as f64 / medium as f64) {
                easy -= 1;
                Easy
            } else {
                medium -= 1;
                Medium
            }
        })).collect();                
        encounters.into_iter().map(|e| match e {
            Easy => Self::easy_encounter(rng),
            Medium => Self::medium_encounter(rng),
            Hard => Self::hard_encounter(rng),
            Boss => panic!(),
        }).collect()
    }
    
    fn easy_encounter(rng: &mut impl Rng) -> Self {
        static FIRST_CHOICE: [&[EnemyType]; 3] = [
            &[EnemyType::Small; 2],
            &[EnemyType::Small; 3],
            &[EnemyType::Medium],
        ];
        static SECOND_CHOICE: [&[EnemyType]; 2] = [
            &[EnemyType::Small; 3],
            &[EnemyType::Medium],
        ];

        let enemies = chain!(
            FIRST_CHOICE.choose(rng).unwrap().iter(),
            SECOND_CHOICE.choose(rng).unwrap().iter(),
        ).copied().collect();

        Self { difficulty: Easy, enemies }
    }

    fn medium_encounter(rng: &mut impl Rng) -> Self {
        static FIRST_CHOICE: [&[EnemyType]; 4] = [
            &[],
            &[EnemyType::Small; 2],
            &[EnemyType::Small; 3],
            &[EnemyType::Medium],
        ];
        static SECOND_CHOICE: [EnemyType; 3] = [EnemyType::Medium, EnemyType::Tank, EnemyType::Dps];

        let enemies = chain!(
            FIRST_CHOICE.choose(rng).unwrap().iter(),
            (0..3).filter_map(|_| SECOND_CHOICE.choose(rng)),
        ).copied().collect();

        Self { difficulty: Medium, enemies }
    }

    fn hard_encounter(rng: &mut impl Rng) -> Self {
        static CHOICE: [EnemyType; 3] = [EnemyType::Medium, EnemyType::Tank, EnemyType::Dps]; // TODO add special
        let picks = rng.random_range(5..=6);
        let enemies = (0..picks).filter_map(|_| CHOICE.choose(rng)).copied().collect();

        Self { difficulty: Hard, enemies }
    }
}
