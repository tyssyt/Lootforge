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