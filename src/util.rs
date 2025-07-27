use rand::{distr::{uniform::{SampleBorrow, SampleUniform}, weighted::Weight}, seq::{IndexedRandom, IteratorRandom, SliceChooseIter}, Rng};

#[derive(PartialEq, PartialOrd)]
pub struct F32Ord(pub f32);
impl Eq for F32Ord {}
impl Ord for F32Ord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

pub trait SplitOption<A, B> {
    fn split(self) -> (Option<A>, Option<B>);
}
impl<A, B> SplitOption<A, B> for Option<(A, B)> {
    fn split(self) -> (Option<A>, Option<B>) {
        match self {
            Some((a, b)) => (Some(a), Some(b)),
            None => (None, None),
        }
    }
}

pub trait IndexedRandomNonEmpty: IndexedRandom {
    fn pick<R: Rng + ?Sized>(&self, rng: &mut R) -> &Self::Output {
        self.choose(rng).unwrap()
    }
    
    fn pick_weighted<R, F, B, X>(&self, rng: &mut R, weight: F) -> &Self::Output
    where
        R: Rng + ?Sized,
        F: Fn(&Self::Output) -> B,
        B: SampleBorrow<X>,
        X: SampleUniform + Weight + PartialOrd<X>,   
    {
        self.choose_weighted(rng, weight).unwrap()
    }

    fn pick_multiple<R>(&self, rng: &mut R, amount: usize) -> SliceChooseIter<'_, Self, Self::Output>
    where
        Self::Output: Sized,
        R: Rng + ?Sized,
    {
        self.choose_multiple(rng, amount)
    }
}
impl <IR: IndexedRandom + ?Sized> IndexedRandomNonEmpty for IR {}

pub trait IteratorRandomNonEmpty: IteratorRandom {
    fn pick<R: Rng + ?Sized>(self, rng: &mut R) -> Self::Item {
        self.choose_stable(rng).unwrap()
    }
}
impl <IR: IteratorRandom + ?Sized> IteratorRandomNonEmpty for IR {}
