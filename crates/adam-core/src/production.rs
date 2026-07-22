use std::collections::{BTreeMap, BTreeSet};

use crate::{FirmId, GoodId, Money, QuantityMilli, RecipeId, RegionId, World, WorldError};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProductionInput {
    good: GoodId,
    quantity_per_batch: QuantityMilli,
}
impl ProductionInput {
    #[must_use]
    pub const fn new(good: GoodId, quantity_per_batch: QuantityMilli) -> Self {
        Self {
            good,
            quantity_per_batch,
        }
    }
    #[must_use]
    pub const fn good(self) -> GoodId {
        self.good
    }
    #[must_use]
    pub const fn quantity_per_batch(self) -> QuantityMilli {
        self.quantity_per_batch
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProductionRecipe {
    id: RecipeId,
    name: String,
    output_good: GoodId,
    output_per_batch: QuantityMilli,
    labor_milli_worker_months: u64,
    inputs: Vec<ProductionInput>,
}
impl ProductionRecipe {
    /// Creates a fixed physical production recipe.
    ///
    /// # Errors
    /// Returns [`WorldError::InvalidProduction`] for empty output, labor, duplicate inputs, or names.
    pub fn new(
        id: RecipeId,
        name: impl Into<String>,
        output_good: GoodId,
        output_per_batch: QuantityMilli,
        labor_milli_worker_months: u64,
        mut inputs: Vec<ProductionInput>,
    ) -> Result<Self, WorldError> {
        let name = name.into();
        if name.trim().is_empty()
            || output_per_batch.get() == 0
            || labor_milli_worker_months == 0
            || inputs.iter().any(|i| i.quantity_per_batch().get() == 0)
        {
            return Err(WorldError::InvalidProduction(
                "recipe fields must be positive and named",
            ));
        }
        let unique: BTreeSet<_> = inputs.iter().map(|i| i.good()).collect();
        if unique.len() != inputs.len() || unique.contains(&output_good) {
            return Err(WorldError::InvalidProduction(
                "inputs must be unique and cannot equal output",
            ));
        }
        inputs.sort_by_key(|i| i.good());
        Ok(Self {
            id,
            name,
            output_good,
            output_per_batch,
            labor_milli_worker_months,
            inputs,
        })
    }
    #[must_use]
    pub const fn id(&self) -> RecipeId {
        self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn output_good(&self) -> GoodId {
        self.output_good
    }
    #[must_use]
    pub const fn output_per_batch(&self) -> QuantityMilli {
        self.output_per_batch
    }
    #[must_use]
    pub const fn labor_milli_worker_months(&self) -> u64 {
        self.labor_milli_worker_months
    }
    #[must_use]
    pub fn inputs(&self) -> &[ProductionInput] {
        &self.inputs
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Firm {
    id: FirmId,
    name: String,
    region: RegionId,
    recipe: RecipeId,
    workers: u64,
    capacity_batches: u64,
    cash: Money,
    inventories: BTreeMap<GoodId, QuantityMilli>,
}
impl Firm {
    /// Creates a production unit with assigned labor, capacity, cash, and inventories.
    ///
    /// # Errors
    /// Returns [`WorldError::InvalidProduction`] for an empty name or negative cash.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: FirmId,
        name: impl Into<String>,
        region: RegionId,
        recipe: RecipeId,
        workers: u64,
        capacity_batches: u64,
        cash: Money,
        inventories: BTreeMap<GoodId, QuantityMilli>,
    ) -> Result<Self, WorldError> {
        let name = name.into();
        if name.trim().is_empty() || cash.minor_units() < 0 {
            return Err(WorldError::InvalidProduction(
                "firm name must exist and cash cannot be negative",
            ));
        }
        Ok(Self {
            id,
            name,
            region,
            recipe,
            workers,
            capacity_batches,
            cash,
            inventories,
        })
    }
    #[must_use]
    pub const fn id(&self) -> FirmId {
        self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn region(&self) -> RegionId {
        self.region
    }
    #[must_use]
    pub const fn recipe(&self) -> RecipeId {
        self.recipe
    }
    #[must_use]
    pub const fn workers(&self) -> u64 {
        self.workers
    }
    #[must_use]
    pub const fn capacity_batches(&self) -> u64 {
        self.capacity_batches
    }
    #[must_use]
    pub const fn cash(&self) -> Money {
        self.cash
    }
    #[must_use]
    pub fn inventories(&self) -> &BTreeMap<GoodId, QuantityMilli> {
        &self.inventories
    }
    pub(crate) const fn set_cash(&mut self, value: Money) {
        self.cash = value;
    }
    pub(crate) fn debit_inventory(
        &mut self,
        good: GoodId,
        quantity: QuantityMilli,
    ) -> Result<(), WorldError> {
        let current = self
            .inventories
            .get(&good)
            .copied()
            .unwrap_or_default()
            .get();
        if current < quantity.get() {
            return Err(WorldError::InsufficientFirmInventory {
                firm: self.id,
                good,
            });
        }
        let next = current - quantity.get();
        if next == 0 {
            self.inventories.remove(&good);
        } else {
            self.inventories.insert(good, QuantityMilli::new(next));
        }
        Ok(())
    }
    pub(crate) fn credit_inventory(
        &mut self,
        good: GoodId,
        quantity: QuantityMilli,
    ) -> Result<(), WorldError> {
        let current = self
            .inventories
            .get(&good)
            .copied()
            .unwrap_or_default()
            .get();
        let next = current
            .checked_add(quantity.get())
            .ok_or(WorldError::ArithmeticOverflow("firm inventory credit"))?;
        self.inventories.insert(good, QuantityMilli::new(next));
        Ok(())
    }
    pub(crate) fn add_capacity(&mut self, value: u64) -> Result<(), WorldError> {
        self.capacity_batches = self
            .capacity_batches
            .checked_add(value)
            .ok_or(WorldError::ArithmeticOverflow("firm capacity"))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProductionPlan {
    firm: FirmId,
    batches: u64,
    output_good: GoodId,
    output: QuantityMilli,
    limiting_factor: &'static str,
}
impl ProductionPlan {
    #[must_use]
    pub const fn firm(&self) -> FirmId {
        self.firm
    }
    #[must_use]
    pub const fn batches(&self) -> u64 {
        self.batches
    }
    #[must_use]
    pub const fn output_good(&self) -> GoodId {
        self.output_good
    }
    #[must_use]
    pub const fn output(&self) -> QuantityMilli {
        self.output
    }
    #[must_use]
    pub const fn limiting_factor(&self) -> &'static str {
        self.limiting_factor
    }
}

impl World {
    /// Registers and validates a production recipe.
    /// # Errors
    /// Returns [`WorldError`] for duplicate IDs or unknown goods.
    pub fn register_production_recipe(
        &mut self,
        recipe: ProductionRecipe,
    ) -> Result<(), WorldError> {
        if self.production_recipes.contains_key(&recipe.id()) {
            return Err(WorldError::DuplicateRecipe(recipe.id()));
        }
        if !self.goods.contains_key(&recipe.output_good()) {
            return Err(WorldError::UnknownGood(recipe.output_good()));
        }
        for input in recipe.inputs() {
            if !self.goods.contains_key(&input.good()) {
                return Err(WorldError::UnknownGood(input.good()));
            }
        }
        self.production_recipes.insert(recipe.id(), recipe);
        Ok(())
    }
    /// Registers and validates a firm.
    /// # Errors
    /// Returns [`WorldError`] for duplicate IDs or unknown region, recipe, or inventory goods.
    pub fn register_firm(&mut self, firm: Firm) -> Result<(), WorldError> {
        if self.firms.contains_key(&firm.id()) {
            return Err(WorldError::DuplicateFirm(firm.id()));
        }
        if !self.regions.contains_key(&firm.region()) {
            return Err(WorldError::UnknownRegion(firm.region()));
        }
        if !self.production_recipes.contains_key(&firm.recipe()) {
            return Err(WorldError::UnknownRecipe(firm.recipe()));
        }
        for good in firm.inventories().keys() {
            if !self.goods.contains_key(good) {
                return Err(WorldError::UnknownGood(*good));
            }
        }
        self.firms.insert(firm.id(), firm);
        Ok(())
    }
    #[must_use]
    pub fn production_recipes(&self) -> &BTreeMap<RecipeId, ProductionRecipe> {
        &self.production_recipes
    }
    #[must_use]
    pub fn firms(&self) -> &BTreeMap<FirmId, Firm> {
        &self.firms
    }

    /// Plans one physical production month without mutating inventories.
    /// # Errors
    /// Returns [`WorldError`] on fixed-point overflow.
    pub fn plan_monthly_production(&self) -> Result<Vec<ProductionPlan>, WorldError> {
        self.firms
            .values()
            .map(|firm| {
                let recipe = &self.production_recipes[&firm.recipe()];
                let labor_batches = firm
                    .workers()
                    .checked_mul(1_000)
                    .ok_or(WorldError::ArithmeticOverflow("firm labor capacity"))?
                    / recipe.labor_milli_worker_months();
                let mut batches = firm.capacity_batches().min(labor_batches);
                let mut limiting = if labor_batches < firm.capacity_batches() {
                    "labor"
                } else {
                    "capital capacity"
                };
                for input in recipe.inputs() {
                    let available = firm
                        .inventories()
                        .get(&input.good())
                        .copied()
                        .unwrap_or_default()
                        .get();
                    let input_batches = available / input.quantity_per_batch().get();
                    if input_batches < batches {
                        batches = input_batches;
                        limiting = "intermediate input";
                    }
                }
                let output = recipe
                    .output_per_batch()
                    .get()
                    .checked_mul(batches)
                    .ok_or(WorldError::ArithmeticOverflow("planned production output"))?;
                Ok(ProductionPlan {
                    firm: firm.id(),
                    batches,
                    output_good: recipe.output_good(),
                    output: QuantityMilli::new(output),
                    limiting_factor: limiting,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Country, CountryId, Good, Population, Region, SimDate, WorldSeed};
    fn world(energy_inventory: u64, workers: u64) -> World {
        let mut w = World::new(WorldSeed::new(1), SimDate::new(2025, 1).expect("date"));
        w.register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("country");
        w.register_good(Good::new(GoodId::new(1), "Food").expect("good"))
            .expect("good");
        w.register_good(Good::new(GoodId::new(2), "Energy").expect("good"))
            .expect("good");
        w.register_region(
            Region::new(
                RegionId::new(1),
                CountryId::new(1),
                "R",
                Population::new(1),
                Money::from_minor_units(1),
            )
            .expect("region"),
        )
        .expect("region");
        w.register_production_recipe(
            ProductionRecipe::new(
                RecipeId::new(1),
                "Food recipe",
                GoodId::new(1),
                QuantityMilli::new(10_000),
                1_000,
                vec![ProductionInput::new(
                    GoodId::new(2),
                    QuantityMilli::new(2_000),
                )],
            )
            .expect("recipe"),
        )
        .expect("recipe");
        let inventory = BTreeMap::from([(GoodId::new(2), QuantityMilli::new(energy_inventory))]);
        w.register_firm(
            Firm::new(
                FirmId::new(1),
                "Farm",
                RegionId::new(1),
                RecipeId::new(1),
                workers,
                100,
                Money::from_minor_units(1_000),
                inventory,
            )
            .expect("firm"),
        )
        .expect("firm");
        w
    }
    #[test]
    fn intermediate_input_can_bind_output() {
        let plan = world(6_000, 100).plan_monthly_production().expect("plan");
        assert_eq!(plan[0].batches(), 3);
        assert_eq!(plan[0].limiting_factor(), "intermediate input");
    }
    #[test]
    fn labor_can_bind_output() {
        let plan = world(1_000_000, 2).plan_monthly_production().expect("plan");
        assert_eq!(plan[0].batches(), 2);
        assert_eq!(plan[0].limiting_factor(), "labor");
    }
}
