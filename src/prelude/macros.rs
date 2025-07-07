use macro_rules_attribute::attribute_alias;

attribute_alias! {
   #[apply(UnitEnum)] = #[derive(Clone, Copy, PartialEq, Eq, strum::EnumIter, strum::VariantArray, strum::EnumCount, strum::EnumIs, strum::FromRepr, derive_more::Debug, derive_more::Unwrap)];
   #[apply(Enum)] = #[derive(Clone, derive_more::Debug, strum::EnumCount, strum::EnumIs)];
   #[apply(Default)] = #[derive(Clone, derive_more::Debug, smart_default::SmartDefault)];
}