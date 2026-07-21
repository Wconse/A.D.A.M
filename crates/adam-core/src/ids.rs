use core::fmt;

macro_rules! typed_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u64);

        impl $name {
            #[must_use]
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn get(self) -> u64 {
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
typed_id!(ActorId);

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
}
