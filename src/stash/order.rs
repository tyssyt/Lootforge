use std::cmp::Ordering;

use strum::EnumIter;

use crate::item::Item;

#[derive(Default, PartialEq, EnumIter, strum::Display, Clone, Copy)]
pub enum Order {
    #[default]
    Age,
    #[strum(to_string = "Rank (Desc)")]
    RankDesc,
    #[strum(to_string = "Rank (Asc.)")]
    RankAsc,
}
impl Order {
    pub fn cmp(&self, a: &Item, b: &Item) -> Ordering {
        match self {
            Order::Age => a.id.cmp(&b.id),
            Order::RankDesc => b.rank().cmp(&a.rank()),
            Order::RankAsc => a.rank().cmp(&b.rank()),
        }
    }
}