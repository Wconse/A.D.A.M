use core::fmt;

macro_rules! typed_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u32);

        impl $name {
            #[must_use]
            pub const fn new(value: u32) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn get(self) -> u32 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

typed_id!(CountryId);
typed_id!(GoodId);
typed_id!(NeedProfileId);
typed_id!(CohortId);
typed_id!(RegionId);
typed_id!(ActorId);
typed_id!(PowerNodeId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_ids_with_same_number_are_distinct_types() {
        let country = CountryId::new(7);
        let actor = ActorId::new(7);
        assert_eq!(country.get(), actor.get());
        assert_eq!(country.to_string(), "7");
    }

    #[test]
    fn ids_have_stable_compact_layout() {
        assert_eq!(size_of::<CountryId>(), size_of::<u32>());
        assert_eq!(size_of::<GoodId>(), size_of::<u32>());
        assert_eq!(size_of::<NeedProfileId>(), size_of::<u32>());
        assert_eq!(size_of::<CohortId>(), size_of::<u32>());
        assert_eq!(size_of::<RegionId>(), size_of::<u32>());
        assert_eq!(size_of::<ActorId>(), size_of::<u32>());
        assert_eq!(size_of::<PowerNodeId>(), size_of::<u32>());
    }
}
