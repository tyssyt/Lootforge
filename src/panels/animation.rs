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

#[apply(UnitEnum)]
#[derive(Default)]
pub enum Animation {
    FighterAttack,
    FighterWalk,
    BigWormIdle,
    BigWornAttack,
    #[default]
    SlimeProj,
}
impl Deref for Animation {
    type Target = AnimationData;
    fn deref(&self) -> &'static Self::Target {
        match *self {
            FighterAttack => &AnimationData { sheet: FighterAttackSheet, len: 6, frames: &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (0, 5)]},
            FighterWalk   => &AnimationData { sheet: FighterWalkSheet, len: 8, frames: &[(0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6), (0, 7), (0, 8)]},
            BigWormIdle   => &AnimationData { sheet: BigWorn, len: 6, frames: &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 2), (0, 1)] },
            BigWornAttack => &AnimationData { sheet: BigWorn, len: 2, frames: &[(0, 4), (0, 5)] },
            SlimeProj     => &AnimationData { sheet: SlimeProjectile, len: 4, frames: &[(0, 1), (0, 2), (0, 3), (0, 4)] },
        }
    }
}

pub struct AnimationData {
    sheet: Spritesheet,
    pub len: usize,
    frames: &'static [(u16, u16)],
}
impl AnimationData {
    pub fn frame(&self, n: usize) -> Image<'_> {
        self.sheet.get_sprite(self.frames[n])
    }
}

#[apply(UnitEnum)]
enum Spritesheet {
    FighterAttackSheet,
    FighterWalkSheet,
    BigWorn,
    SlimeProjectile,
}
// TODO consider fucking magic macro instead, this is like barely okay but already annoying
impl Spritesheet {
    fn get_sprite(&self, pos: (u16, u16)) -> Image<'_> {
        self.get_sprite_2(pos.0, pos.1)
    }

    fn get_sprite_2(&self, row: u16, col: u16) -> Image<'_> {
        Image::new(self.image())
            .maintain_aspect_ratio(false)
            .fit_to_exact_size(self.sprite_size())
            .uv(Rect::from_min_max(
                pos2(col as f32 / self.cols() as f32, row as f32 / self.rows() as f32),
                pos2((col+1) as f32 / self.cols() as f32, (row+1) as f32 / self.rows() as f32),
            ))
    }

    fn image(&self) -> ImageSource<'_> {
        match *self {
            FighterAttackSheet => include_image!("../../assets/explorers/fighter_attack.png"),
            FighterWalkSheet   => include_image!("../../assets/explorers/fighter_walk.png"),
            BigWorn            => include_image!("../../assets/enemies/big_worm.png"),
            SlimeProjectile    => include_image!("../../assets/enemies/slime-projectile.png"),
        }
    }
    fn sprite_size(&self) -> Vec2 {        
        match *self {
            FighterAttackSheet => vec2(128., 64.),
            FighterWalkSheet   => vec2(64., 64.),
            BigWorn            => vec2(64., 64.),
            SlimeProjectile    => vec2(16., 16.),
        }
    }
    // TODO combine rows and cols and then do math on that above
    fn rows(&self) -> u16 {        
        match *self {
            FighterAttackSheet => 1,
            FighterWalkSheet   => 1,
            BigWorn            => 1,
            SlimeProjectile    => 1,
        }
    }
    fn cols(&self) -> u16 {        
        match *self {
            FighterAttackSheet => 6,
            FighterWalkSheet   => 9,
            BigWorn            => 6,
            SlimeProjectile    => 4,
        }
    }
}