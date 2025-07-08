use crate::prelude::*;

use Animation::*;
use Spritesheet::*;


//type Animation = (usize, Rc<AnimationData>); // not sure yet on the Rc part

// for monsters, they will almost always have the same cycle (aka attack ever 10 frames)
// there can be special abilities that trigger attacks outside that cycle though


/*
    EVERY Skill gets a .5 delay between activtion and resolution
    This also applies to reactionary stuff, because I figure out at the moment of activation if a reaction happens

    --OR--

    the screen renders at a 5(6? 10?) frame delay, so that the logic is nice and simple if the screen is not open






    Considerations Melee Skills:
        - 1 Frame where he "blinks"
        - Windup / Skill / Winddown
        - Blink Back

        - what about aoe melee attacks??? I guess one could go 'in the middle'
        - if multiple melee skills overlap, we spawn duplicate sprites cause that is rad

    
    Considerations Projectile Skills:
        - Windup / Skill
        - Spawn Proj which files, meanwhile Winddown

    Considerations instant ranged Skills:
        - simplest, have regular Windup / Skill / Winddown
        - on Attack, spawn effect

    => weird stuff can happen, like I can blink to an enemy who is blinking to someone else if 2 melee skills overlap


    Idea, I check what the next "regular" skill is that would trigger and play that skills animation
    (if it has enough "before frames")

    => for enemies, I can make that animation long enough that it loops itself with the skill cd
    => basic enemies need no idle animation
    => only explorers need idle animations (and have much shorter skill animations)

    if an irregular skill appears (so something reactionary or something externally triggered)
        I 

*/

#[derive(Debug, Clone)]
pub struct AnimationPlayer {
    data: &'static AnimationData,
    frame: usize,
}
impl AnimationPlayer {
    pub fn new(animation: Animation) -> Self {
        Self { data: animation.deref_static(), frame: 0 }
    }
    pub fn frame(&mut self) -> Image<'_> {
        self.data.frame(self.frame)
    }
    pub fn next_frame(&mut self) -> bool {
        if self.frame >= self.data.len -1 {
            false
        } else {
            self.frame += 1;
            true
        }
    }
    pub fn sprite_size(&self) -> Vec2 {
        self.data.sprite_size()
    }
}

#[apply(UnitEnum)]
#[derive(Default)]
pub enum Animation {
    FighterAttack,
    FighterIdle,
    FighterWalk,

    BigWormIdle,
    BigWornAttack,
    BatIdle,
    BatAttack,
    MinotaurIdle,
    MinotaurAttack,
    OrcIdle,
    OrcAttack,
    SkeletonIdle,
    SkeletonAttack,
    SpiderIdle,
    SpiderAttack,
    BigSpiderIdle,
    BigSpiderAttack,

    #[default]
    SlimeProj,
}
impl Deref for Animation {
    type Target = AnimationData;
    fn deref(&self) -> &'static Self::Target {
        self.deref_static()
    }
}
impl Animation {
    pub fn deref_static(self) -> &'static AnimationData {
        match self {
            FighterAttack => &AnimationData { sheet: FighterAttackSheet, len: 6, frames: &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (0, 5)]},
            FighterIdle   => &AnimationData { sheet: FighterWalkSheet, len: 7, frames: &[(0, 3), (0, 3), (0, 3), (0, 6), (0, 5), (0, 5), (0, 6)]},
            FighterWalk   => &AnimationData { sheet: FighterWalkSheet, len: 8, frames: &[(0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6), (0, 7), (0, 8)]},

            
            BatIdle         => &AnimationData { sheet: Bat, len: 4, frames: &[(0, 0), (0, 1), (0, 2), (0, 1)] },
            BatAttack       => &AnimationData { sheet: Bat, len: 5, frames: &[(0, 2), (0, 3), (0, 4), (0, 4), (0, 5)] },
            BigWormIdle     => &AnimationData { sheet: BigWorn, len: 6, frames: &[(0, 0), (0, 0), (0, 1), (0, 2), (0, 2), (0, 1)] },
            BigWornAttack   => &AnimationData { sheet: BigWorn, len: 2, frames: &[(0, 4), (0, 5)] },
            MinotaurIdle    => &AnimationData { sheet: Minotaur, len: 5, frames: &[(0, 0), (0, 0), (0, 0), (0, 1), (0, 1)] },
            MinotaurAttack  => &AnimationData { sheet: Minotaur, len: 5, frames: &[(0, 2), (0, 3), (0, 4), (0, 5), (0, 6)] },
            OrcIdle         => &AnimationData { sheet: Orc, len: 6, frames: &[(0, 4), (0, 4), (0, 4), (0, 4), (0, 3), (0, 3)] },
            OrcAttack       => &AnimationData { sheet: Orc, len: 6, frames: &[(0, 5), (0, 0), (0, 1), (0, 2), (0, 3), (0, 4)] },
            SkeletonIdle    => &AnimationData { sheet: Skeleton, len: 7, frames: &[(0, 0), (0, 0), (0, 0), (0, 1), (0, 2), (0, 2), (0, 1)] },
            SkeletonAttack  => &AnimationData { sheet: Skeleton, len: 7, frames: &[(0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6), (0, 7)] },
            SpiderIdle      => &AnimationData { sheet: Spider, len: 5, frames: &[(0, 2), (0, 4), (0, 5), (0, 3), (0, 3)] },
            SpiderAttack    => &AnimationData { sheet: Spider, len: 8, frames: &[(0, 2), (0, 3), (0, 4), (0, 3), (0, 5), (0, 2), (0, 1), (0, 0)] },
            BigSpiderIdle   => &AnimationData { sheet: BigSpider, len: 5, frames: &[(0, 0), (0, 0), (0, 0), (0, 4), (0, 6)] },
            BigSpiderAttack => &AnimationData { sheet: BigSpider, len: 4, frames: &[(0, 0), (0, 1), (0, 2), (0, 3)] },
            
            SlimeProj => &AnimationData { sheet: SlimeProjectile, len: 4, frames: &[(0, 1), (0, 2), (0, 3), (0, 4)] },
        }
    }
}

#[derive(derive_more::Debug, Clone)]
pub struct AnimationData {
    sheet: Spritesheet,
    pub len: usize,

    #[debug(skip)]
    frames: &'static [(u16, u16)],
}
impl AnimationData {
    pub fn frame(&self, n: usize) -> Image<'static> {
        self.sheet.get_sprite(self.frames[n])
    }
    pub fn sprite_size(&self) -> Vec2 {
        self.sheet.sprite_size()
    }
}

#[apply(UnitEnum)]
pub enum Spritesheet {
    FighterAttackSheet,
    FighterWalkSheet,
    Bat,
    BigWorn,
    Minotaur,
    Orc,
    Skeleton,
    Spider,
    BigSpider,
    SlimeProjectile,
}
impl Spritesheet {
    pub fn get_sprite(&self, pos: (u16, u16)) -> Image<'static> {
        self.get_sprite_2(pos.0, pos.1)
    }

    pub fn get_sprite_2(&self, row: u16, col: u16) -> Image<'static> {
        Image::new(self.image())
            .maintain_aspect_ratio(false)
            .fit_to_exact_size(self.sprite_size())
            .uv(Rect::from_min_max(
                pos2(col as f32 / self.cols() as f32, row as f32 / self.rows() as f32),
                pos2((col+1) as f32 / self.cols() as f32, (row+1) as f32 / self.rows() as f32),
            ))
    }

    fn image(&self) -> ImageSource<'static> {
        match *self {
            FighterAttackSheet => include_image!("../../assets/explorers/fighter_attack.png"),
            FighterWalkSheet   => include_image!("../../assets/explorers/fighter_walk.png"),
            Bat                => include_image!("../../assets/enemies/bat.png"),
            BigWorn            => include_image!("../../assets/enemies/big_worm.png"),
            Minotaur           => include_image!("../../assets/enemies/minotaur.png"),
            Orc                => include_image!("../../assets/enemies/orc.png"),
            Skeleton           => include_image!("../../assets/enemies/skeleton.png"),
            Spider             => include_image!("../../assets/enemies/spider.png"),
            BigSpider          => include_image!("../../assets/enemies/big_spider.png"),
            SlimeProjectile    => include_image!("../../assets/enemies/slime-projectile.png"),
        }
    }
    fn sprite_size(&self) -> Vec2 {        
        match *self {
            FighterAttackSheet => vec2(128., 64.),
            FighterWalkSheet   => vec2(64., 64.),
            Bat                => vec2(32., 32.),
            BigWorn            => vec2(64., 64.),
            Minotaur           => vec2(74., 64.),
            Orc                => vec2(128., 68.),
            Skeleton           => vec2(64., 64.),
            Spider             => vec2(32., 32.),
            BigSpider          => vec2(64., 64.),
            SlimeProjectile    => vec2(16., 16.),
        }
    }
    // TODO combine rows and cols and then do math on that above
    fn rows(&self) -> u16 {        
        match *self {
            FighterAttackSheet => 1,
            FighterWalkSheet   => 1,
            Bat                => 1,
            BigWorn            => 1,
            Minotaur           => 1,
            Orc                => 1,
            Skeleton           => 1,
            Spider             => 1,
            BigSpider          => 1,
            SlimeProjectile    => 1,
        }
    }
    fn cols(&self) -> u16 {        
        match *self {
            FighterAttackSheet => 6,
            FighterWalkSheet   => 9,
            Bat                => 6,
            BigWorn            => 6,
            Minotaur           => 7,
            Orc                => 6,
            Skeleton           => 8,
            Spider             => 6,
            BigSpider          => 10,
            SlimeProjectile    => 4,
        }
    }
}