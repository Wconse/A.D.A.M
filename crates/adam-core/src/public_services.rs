use crate::BasisPoints;

/// Persistent regional capacity delivered by recurring public spending.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RegionalPublicServices {
    healthcare: BasisPoints,
    infrastructure: BasisPoints,
    administration: BasisPoints,
}

impl Default for RegionalPublicServices {
    fn default() -> Self {
        let baseline = BasisPoints::new(5_000).expect("service baseline is bounded");
        Self {
            healthcare: baseline,
            infrastructure: baseline,
            administration: baseline,
        }
    }
}

impl RegionalPublicServices {
    #[must_use]
    pub const fn healthcare(self) -> BasisPoints {
        self.healthcare
    }

    #[must_use]
    pub const fn infrastructure(self) -> BasisPoints {
        self.infrastructure
    }

    #[must_use]
    pub const fn administration(self) -> BasisPoints {
        self.administration
    }

    #[must_use]
    pub(crate) fn improved_by_program(
        self,
        priority: crate::PublicServicePriority,
        improvement: BasisPoints,
    ) -> Self {
        let increase = |value: BasisPoints| {
            BasisPoints::new(value.get().saturating_add(improvement.get()).min(10_000))
                .expect("program service improvement is bounded")
        };
        match priority {
            crate::PublicServicePriority::Healthcare => Self {
                healthcare: increase(self.healthcare),
                ..self
            },
            crate::PublicServicePriority::Infrastructure => Self {
                infrastructure: increase(self.infrastructure),
                ..self
            },
            crate::PublicServicePriority::Administration => Self {
                administration: increase(self.administration),
                ..self
            },
        }
    }

    pub(crate) fn adjusted_toward_funding(self, funding_target: BasisPoints) -> Self {
        let administration = move_toward(self.administration, funding_target, 250);
        let delivery_ceiling = BasisPoints::new(
            administration
                .get()
                .saturating_add(1_000)
                .min(BasisPoints::MAX),
        )
        .expect("service ceiling is bounded");
        let delivered_target = BasisPoints::new(funding_target.get().min(delivery_ceiling.get()))
            .expect("service target is bounded");
        Self {
            healthcare: move_toward(self.healthcare, delivered_target, 500),
            infrastructure: move_toward(self.infrastructure, delivered_target, 500),
            administration,
        }
    }
}

fn move_toward(current: BasisPoints, target: BasisPoints, step: u16) -> BasisPoints {
    let value = if current.get() < target.get() {
        current.get().saturating_add(step).min(target.get())
    } else {
        current.get().saturating_sub(step).max(target.get())
    };
    BasisPoints::new(value).expect("bounded service movement")
}

impl crate::World {
    #[must_use]
    pub fn regional_public_services(
        &self,
    ) -> &std::collections::BTreeMap<crate::RegionId, RegionalPublicServices> {
        &self.regional_public_services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn administrative_capacity_limits_service_recovery() {
        let services = RegionalPublicServices::default().adjusted_toward_funding(BasisPoints::FULL);
        assert_eq!(services.administration().get(), 5_250);
        assert_eq!(services.healthcare().get(), 5_500);
        assert_eq!(services.infrastructure().get(), 5_500);
    }
}
