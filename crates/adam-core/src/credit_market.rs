use crate::{
    ActorId, BasisPoints, CorporateAction, CorporateRole, FirmCreditorPriority, FirmId, Money,
    QuantityMilli, World, WorldCommand, WorldError,
};

const AUTONOMOUS_CREDIT_TERM_MONTHS: u16 = 12;
const BASE_INTEREST_BPS: u16 = 600;
const DISTRESS_INTEREST_BPS_PER_MONTH: u16 = 250;
const MAX_PORTFOLIO_SHARE_BPS: i128 = 4_000;
const LENDER_LIQUIDITY_RESERVE_BPS: i128 = 5_000;
const MINIMUM_GAP_COVERAGE_BPS: i128 = 2_500;
const SUCCESSFUL_LOAN_CAPACITY_BONUS_BPS: i128 = 100;
const MAX_SUCCESSFUL_LOAN_CAPACITY_BONUS_BPS: i128 = 500;
const MAX_LOSS_CAPACITY_PENALTY_BPS: i128 = 2_500;
const MAX_LOSS_RATE_PREMIUM_BPS: i128 = 3_000;
const SUCCESSFUL_LOAN_RATE_DISCOUNT_BPS: u16 = 50;
const MAX_SUCCESSFUL_LOAN_RATE_DISCOUNT_BPS: u16 = 250;
const BORROWER_DISTRESS_RISK_BPS_PER_MONTH: i128 = 400;
const MAX_COLLATERAL_CUSHION_BPS: i128 = 500;
const MAX_BORROWER_PAYMENT_SHORTFALL_PREMIUM_BPS: i128 = 2_000;
const BORROWER_DELINQUENCY_PREMIUM_BPS: i128 = 100;
const MAX_BORROWER_DELINQUENCY_PREMIUM_BPS: i128 = 1_500;
const BORROWER_DEFAULT_PREMIUM_BPS: i128 = 1_000;
const MAX_BORROWER_DEFAULT_PREMIUM_BPS: i128 = 3_000;
const BORROWER_SUCCESS_DISCOUNT_BPS: i128 = 50;
const MAX_BORROWER_SUCCESS_DISCOUNT_BPS: i128 = 250;

#[derive(Clone, Copy, Debug)]
struct AutonomousCreditRequest {
    actor: ActorId,
    firm: FirmId,
    country: crate::CountryId,
    funding_gap: Money,
}

#[derive(Clone, Copy, Debug)]
struct LenderAllocationCandidate {
    request: AutonomousCreditRequest,
    rate: BasisPoints,
    maximum_offer: crate::FirmCreditOffer,
    risk_adjusted_return_bps: i128,
}

/// Cumulative realized lending outcomes for one actor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LenderCreditHistory {
    principal_repaid: Money,
    interest_income: Money,
    realized_losses: Money,
    successful_loans: u64,
    defaulted_loans: u64,
}

impl LenderCreditHistory {
    #[must_use]
    pub const fn principal_repaid(self) -> Money {
        self.principal_repaid
    }
    #[must_use]
    pub const fn interest_income(self) -> Money {
        self.interest_income
    }
    #[must_use]
    pub const fn realized_losses(self) -> Money {
        self.realized_losses
    }
    #[must_use]
    pub const fn successful_loans(self) -> u64 {
        self.successful_loans
    }
    #[must_use]
    pub const fn defaulted_loans(self) -> u64 {
        self.defaulted_loans
    }
}

/// Cumulative realized debt-service conduct for one borrowing firm.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BorrowerCreditHistory {
    scheduled_due: Money,
    scheduled_paid: Money,
    on_time_payments: u64,
    delinquent_payments: u64,
    successful_loans: u64,
    defaulted_loans: u64,
}

impl BorrowerCreditHistory {
    #[must_use]
    pub const fn scheduled_due(self) -> Money {
        self.scheduled_due
    }
    #[must_use]
    pub const fn scheduled_paid(self) -> Money {
        self.scheduled_paid
    }
    #[must_use]
    pub const fn on_time_payments(self) -> u64 {
        self.on_time_payments
    }
    #[must_use]
    pub const fn delinquent_payments(self) -> u64 {
        self.delinquent_payments
    }
    #[must_use]
    pub const fn successful_loans(self) -> u64 {
        self.successful_loans
    }
    #[must_use]
    pub const fn defaulted_loans(self) -> u64 {
        self.defaulted_loans
    }
}

/// One autonomous financing decision made from observed borrower and lender state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AutonomousFirmCreditDecision {
    pub actor: ActorId,
    pub firm: FirmId,
    pub creditor: ActorId,
    pub principal: Money,
    pub annual_interest: BasisPoints,
    pub term_months: u16,
    pub funding_gap: Money,
}

impl World {
    /// Lets viable cash-constrained firms seek competing domestic credit offers.
    ///
    /// Borrowers request only enough to cover one observed operating month. Lenders retain half
    /// their liquid cash, cap total firm-credit exposure at 40% of liquid wealth plus claims, and
    /// price distress plus portfolio concentration. The borrower accepts the cheapest offer that
    /// covers at least one quarter of the funding gap through the ordinary command boundary.
    ///
    /// # Errors
    /// Returns an error on duplicate monthly execution, inconsistent references, arithmetic
    /// overflow, or a failed authoritative offer/acceptance transition. The operation is atomic.
    #[allow(clippy::too_many_lines)]
    pub fn execute_observed_firm_credit_market(
        &mut self,
    ) -> Result<Vec<AutonomousFirmCreditDecision>, WorldError> {
        if self.last_firm_credit_market_date == Some(self.date) {
            return Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "observed firm credit market",
                date: self.date,
            });
        }
        let mut next = self.clone();
        next.firm_credit_offers
            .retain(|_, offer| offer.expires_on() >= next.date);

        // Applications are collected before any lender commits capital. This prevents canonical
        // firm order from deciding which borrower consumes a scarce portfolio first.
        let mut requests = Vec::new();
        for firm in next.firms.keys().copied().collect::<Vec<_>>() {
            if next.is_firm_insolvent(firm)
                || next.firm_operating_history.get(&firm).map_or(0, Vec::len) < 3
                || next
                    .firm_creditor_claims
                    .keys()
                    .any(|(candidate, _, _)| *candidate == firm)
            {
                continue;
            }
            let Some(actor) = next.financing_actor(firm) else {
                continue;
            };
            let Some(funding_gap) = next.observed_firm_funding_gap(firm)? else {
                continue;
            };
            requests.push(AutonomousCreditRequest {
                actor,
                firm,
                country: next.firm_country(firm)?,
                funding_gap,
            });
        }
        let reviewed = u64::try_from(requests.len())
            .map_err(|_| WorldError::ArithmeticOverflow("reviewed credit applications"))?;
        let mut offers_underwritten = 0_u64;

        // Each lender sees the whole contemporaneous application batch. A provisional
        // underwriting pass establishes the evidence-backed maximum; finite portfolio headroom
        // is then committed to the highest risk-adjusted return first. Distress is charged more
        // heavily than its contractual rate premium, while over-collateralization earns only a
        // bounded cushion, so a high nominal rate cannot automatically make the weakest borrower
        // the best use of scarce capital.
        let lenders: Vec<_> = next.actors().keys().copied().collect();
        for creditor in lenders {
            let mut remaining = next.autonomous_lender_capacity(creditor)?;
            if remaining.minor_units() <= 0 {
                continue;
            }
            let creditor_country = next.actor_country(creditor)?;
            let mut candidates = Vec::new();
            for request in requests.iter().copied().filter(|request| {
                request.country == creditor_country
                    && request.actor != creditor
                    && !next
                        .ownership_stakes
                        .contains_key(&(request.firm, creditor))
            }) {
                let rate = next.autonomous_credit_rate(creditor, request.firm)?;
                let desired = Self::autonomous_credit_request_with_service_reserve(
                    request.funding_gap,
                    rate,
                )?;
                let mut probe = next.clone();
                let maximum_offer = match probe.underwrite_firm_credit_offer(
                    creditor,
                    request.firm,
                    FirmCreditorPriority::Secured,
                    desired,
                    rate,
                    AUTONOMOUS_CREDIT_TERM_MONTHS,
                ) {
                    Ok(offer) => offer,
                    Err(WorldError::InvalidFirmCredit(_)) => continue,
                    Err(error) => return Err(error),
                };
                let principal = i128::from(maximum_offer.principal().minor_units()).max(1);
                let collateral_ratio_bps =
                    i128::from(maximum_offer.collateral_value().minor_units().max(0))
                        .saturating_mul(10_000)
                        .saturating_div(principal);
                let collateral_cushion = collateral_ratio_bps
                    .saturating_sub(10_000)
                    .saturating_div(20)
                    .clamp(0, MAX_COLLATERAL_CUSHION_BPS);
                let distress_charge = i128::from(
                    next.firm_distress_months
                        .get(&request.firm)
                        .copied()
                        .unwrap_or(0),
                )
                .saturating_mul(BORROWER_DISTRESS_RISK_BPS_PER_MONTH);
                candidates.push(LenderAllocationCandidate {
                    request,
                    rate,
                    maximum_offer,
                    risk_adjusted_return_bps: i128::from(rate.get())
                        .saturating_add(collateral_cushion)
                        .saturating_sub(distress_charge),
                });
            }
            candidates.sort_by_key(|candidate| {
                (
                    std::cmp::Reverse(candidate.risk_adjusted_return_bps),
                    std::cmp::Reverse(candidate.maximum_offer.principal()),
                    candidate.request.firm,
                )
            });
            for candidate in candidates {
                if remaining.minor_units() <= 0 {
                    break;
                }
                let allocated = Money::from_minor_units(
                    candidate
                        .maximum_offer
                        .principal()
                        .minor_units()
                        .min(remaining.minor_units()),
                );
                match next.underwrite_firm_credit_offer(
                    creditor,
                    candidate.request.firm,
                    FirmCreditorPriority::Secured,
                    allocated,
                    candidate.rate,
                    AUTONOMOUS_CREDIT_TERM_MONTHS,
                ) {
                    Ok(offer) => {
                        offers_underwritten = offers_underwritten.saturating_add(1);
                        remaining = Money::from_minor_units(
                            remaining
                                .minor_units()
                                .checked_sub(offer.principal().minor_units())
                                .ok_or(WorldError::ArithmeticOverflow(
                                    "lender allocation headroom",
                                ))?,
                        );
                    }
                    Err(WorldError::InvalidFirmCredit(_)) => {}
                    Err(error) => return Err(error),
                }
            }
        }

        let mut decisions = Vec::new();
        for request in requests {
            let minimum_acceptable = i128::from(request.funding_gap.minor_units())
                .saturating_mul(MINIMUM_GAP_COVERAGE_BPS)
                / 10_000;
            let selected = next
                .firm_credit_offers
                .iter()
                .filter(|((firm, _, _), _)| *firm == request.firm)
                .map(|(_, offer)| *offer)
                .filter(|offer| i128::from(offer.principal().minor_units()) >= minimum_acceptable)
                .min_by_key(|offer| {
                    (
                        offer.annual_interest(),
                        std::cmp::Reverse(offer.principal()),
                        offer.creditor(),
                    )
                });
            let Some(selected) = selected else {
                next.firm_credit_offers
                    .retain(|(firm, _, _), _| *firm != request.firm);
                continue;
            };
            WorldCommand::AcceptFirmCreditOffer {
                actor: request.actor,
                creditor: selected.creditor(),
                firm: request.firm,
                priority: selected.priority(),
            }
            .apply(&mut next)?;
            next.firm_credit_offers
                .retain(|(firm, _, _), _| *firm != request.firm);
            decisions.push(AutonomousFirmCreditDecision {
                actor: request.actor,
                firm: request.firm,
                creditor: selected.creditor(),
                principal: selected.principal(),
                annual_interest: selected.annual_interest(),
                term_months: selected.term_months(),
                funding_gap: request.funding_gap,
            });
        }
        next.last_firm_credit_market_date = Some(next.date);
        next.events.append(
            next.date,
            crate::DomainEvent::ObservedFirmCreditMarketCompleted {
                borrowers_reviewed: reviewed,
                offers_underwritten,
                offers_accepted: u64::try_from(decisions.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("accepted credit offers"))?,
            },
        );
        *self = next;
        Ok(decisions)
    }

    fn autonomous_credit_request_with_service_reserve(
        funding_gap: Money,
        rate: BasisPoints,
    ) -> Result<Money, WorldError> {
        let gap = i128::from(funding_gap.minor_units());
        let first_principal = (gap + i128::from(AUTONOMOUS_CREDIT_TERM_MONTHS) - 1)
            / i128::from(AUTONOMOUS_CREDIT_TERM_MONTHS);
        let first_interest = (gap * i128::from(rate.get()) + 119_999) / 120_000;
        let requested = gap
            .checked_add(first_principal)
            .and_then(|value| value.checked_add(first_interest))
            .ok_or(WorldError::ArithmeticOverflow("credit service reserve"))?;
        Ok(Money::from_minor_units(i64::try_from(requested).map_err(
            |_| WorldError::ArithmeticOverflow("credit request"),
        )?))
    }

    fn observed_firm_funding_gap(&self, firm: FirmId) -> Result<Option<Money>, WorldError> {
        let definition = self.firms.get(&firm).ok_or(WorldError::UnknownFirm(firm))?;
        let baseline = self
            .observed_operating_baseline(firm)?
            .ok_or(WorldError::InvalidFirmCredit("missing operating evidence"))?;
        let recipe = self
            .production_recipes
            .get(&definition.recipe())
            .ok_or(WorldError::UnknownRecipe(definition.recipe()))?;
        let target = self
            .firm_production_targets
            .get(&firm)
            .copied()
            .unwrap_or(definition.capacity_batches());
        let (payroll, wage_arrears) = self
            .employment_agreements
            .values()
            .filter(|row| row.firm() == firm)
            .try_fold((0_i128, 0_i128), |(payroll, arrears), row| {
                let current = if row.active() {
                    i128::from(row.wage().minor_units()) * i128::from(row.workers())
                } else {
                    0
                };
                Ok::<(i128, i128), WorldError>((
                    payroll
                        .checked_add(current)
                        .ok_or(WorldError::ArithmeticOverflow("credit funding payroll"))?,
                    arrears
                        .checked_add(i128::from(row.arrears().minor_units()))
                        .ok_or(WorldError::ArithmeticOverflow(
                            "credit funding wage arrears",
                        ))?,
                ))
            })?;
        let mut inputs = 0_i128;
        for input in recipe.inputs() {
            let price = baseline.input_prices.get(&input.good()).ok_or(
                WorldError::MissingRegionalPrice {
                    region: definition.region(),
                    good: input.good(),
                },
            )?;
            inputs = inputs
                .checked_add(
                    i128::from(price.minor_units())
                        * i128::from(input.quantity_per_batch().get())
                        * i128::from(target)
                        / i128::from(QuantityMilli::SCALE),
                )
                .ok_or(WorldError::ArithmeticOverflow("credit funding inputs"))?;
        }
        let obligation = payroll
            .checked_add(wage_arrears)
            .and_then(|value| value.checked_add(inputs))
            .ok_or(WorldError::ArithmeticOverflow("credit funding obligation"))?;
        let observed_surplus = i128::from(baseline.monthly_sales.minor_units())
            .saturating_sub(payroll)
            .saturating_sub(inputs);
        let gap = obligation.saturating_sub(i128::from(definition.cash().minor_units()));
        if observed_surplus <= 0 || gap <= 0 {
            return Ok(None);
        }
        Ok(Some(Money::from_minor_units(i64::try_from(gap).map_err(
            |_| WorldError::ArithmeticOverflow("firm funding gap"),
        )?)))
    }

    pub(crate) fn autonomous_lender_capacity(
        &self,
        creditor: ActorId,
    ) -> Result<Money, WorldError> {
        let cash = i128::from(
            self.actor_cash
                .get(&creditor)
                .copied()
                .unwrap_or_default()
                .minor_units(),
        )
        .max(0);
        let exposure = self
            .firm_creditor_claims
            .values()
            .filter(|claim| claim.creditor() == creditor)
            .try_fold(0_i128, |sum, claim| {
                sum.checked_add(i128::from(claim.principal().minor_units()))
                    .and_then(|value| {
                        value.checked_add(i128::from(claim.accrued_interest().minor_units()))
                    })
                    .ok_or(WorldError::ArithmeticOverflow("lender credit exposure"))
            })?;
        let wealth = cash
            .checked_add(exposure)
            .ok_or(WorldError::ArithmeticOverflow("lender financial wealth"))?;
        let history = self
            .lender_credit_history
            .get(&creditor)
            .copied()
            .unwrap_or_default();
        let success_bonus = i128::from(history.successful_loans)
            .saturating_mul(SUCCESSFUL_LOAN_CAPACITY_BONUS_BPS)
            .min(MAX_SUCCESSFUL_LOAN_CAPACITY_BONUS_BPS);
        let loss_penalty = self
            .lender_loss_ratio_bps(creditor)
            .saturating_div(2)
            .min(MAX_LOSS_CAPACITY_PENALTY_BPS);
        let portfolio_share = MAX_PORTFOLIO_SHARE_BPS
            .saturating_add(success_bonus)
            .saturating_sub(loss_penalty)
            .clamp(1_500, 4_500);
        let portfolio_headroom = wealth
            .saturating_mul(portfolio_share)
            .saturating_div(10_000)
            .saturating_sub(exposure)
            .max(0);
        let liquid_headroom = cash
            .saturating_mul(10_000 - LENDER_LIQUIDITY_RESERVE_BPS)
            .saturating_div(10_000);
        Ok(Money::from_minor_units(
            i64::try_from(portfolio_headroom.min(liquid_headroom))
                .map_err(|_| WorldError::ArithmeticOverflow("lender capacity"))?,
        ))
    }

    pub(crate) fn autonomous_credit_rate(
        &self,
        creditor: ActorId,
        firm: FirmId,
    ) -> Result<BasisPoints, WorldError> {
        let exposure = self
            .firm_creditor_claims
            .values()
            .filter(|claim| claim.creditor() == creditor)
            .try_fold(0_i128, |sum, claim| {
                sum.checked_add(i128::from(claim.principal().minor_units()))
                    .ok_or(WorldError::ArithmeticOverflow("lender concentration"))
            })?;
        let cash = i128::from(
            self.actor_cash
                .get(&creditor)
                .copied()
                .unwrap_or_default()
                .minor_units(),
        )
        .max(0);
        let wealth = cash.saturating_add(exposure).max(1);
        let concentration_premium = u16::try_from(
            exposure
                .saturating_mul(2_000)
                .saturating_div(wealth)
                .clamp(0, 2_000),
        )
        .map_err(|_| WorldError::ArithmeticOverflow("credit concentration premium"))?;
        let distress_premium =
            u16::from(self.firm_distress_months.get(&firm).copied().unwrap_or(0))
                .saturating_mul(DISTRESS_INTEREST_BPS_PER_MONTH);
        let history = self
            .lender_credit_history
            .get(&creditor)
            .copied()
            .unwrap_or_default();
        let loss_premium = u16::try_from(
            self.lender_loss_ratio_bps(creditor)
                .saturating_mul(MAX_LOSS_RATE_PREMIUM_BPS)
                .saturating_div(10_000)
                .clamp(0, MAX_LOSS_RATE_PREMIUM_BPS),
        )
        .map_err(|_| WorldError::ArithmeticOverflow("lender loss premium"))?;
        let success_discount = u16::try_from(history.successful_loans)
            .unwrap_or(u16::MAX)
            .saturating_mul(SUCCESSFUL_LOAN_RATE_DISCOUNT_BPS)
            .min(MAX_SUCCESSFUL_LOAN_RATE_DISCOUNT_BPS);
        let borrower_adjustment = self.borrower_credit_rate_adjustment_bps(firm);
        let rate = i128::from(BASE_INTEREST_BPS)
            .saturating_add(i128::from(concentration_premium))
            .saturating_add(i128::from(distress_premium))
            .saturating_add(i128::from(loss_premium))
            .saturating_sub(i128::from(success_discount))
            .saturating_add(borrower_adjustment)
            .clamp(0, i128::from(BasisPoints::MAX));
        let rate = u16::try_from(rate)
            .map_err(|_| WorldError::ArithmeticOverflow("borrower adjusted credit rate"))?;
        BasisPoints::new(rate).map_err(|_| WorldError::InvalidFirmCredit("invalid credit rate"))
    }

    pub(crate) fn record_borrower_credit_outcome(
        &mut self,
        firm: FirmId,
        due: Money,
        paid: Money,
        service_attempt: bool,
        resolved: bool,
        defaulted: bool,
    ) -> Result<(), WorldError> {
        if !self.firms.contains_key(&firm) {
            return Err(WorldError::UnknownFirm(firm));
        }
        let history = self.borrower_credit_history.entry(firm).or_default();
        if service_attempt {
            history.scheduled_due = Money::from_minor_units(
                history
                    .scheduled_due
                    .minor_units()
                    .checked_add(due.minor_units())
                    .ok_or(WorldError::ArithmeticOverflow(
                        "borrower scheduled due history",
                    ))?,
            );
            history.scheduled_paid = Money::from_minor_units(
                history
                    .scheduled_paid
                    .minor_units()
                    .checked_add(paid.minor_units())
                    .ok_or(WorldError::ArithmeticOverflow(
                        "borrower scheduled paid history",
                    ))?,
            );
            if paid.minor_units() >= due.minor_units() {
                history.on_time_payments = history.on_time_payments.saturating_add(1);
            } else {
                history.delinquent_payments = history.delinquent_payments.saturating_add(1);
            }
        }
        if resolved {
            if defaulted {
                history.defaulted_loans = history.defaulted_loans.saturating_add(1);
            } else {
                history.successful_loans = history.successful_loans.saturating_add(1);
            }
        }
        self.events.append(
            self.date,
            crate::DomainEvent::BorrowerCreditHistoryUpdated {
                firm,
                due,
                paid,
                service_attempt,
                resolved,
                defaulted,
            },
        );
        Ok(())
    }

    fn borrower_credit_rate_adjustment_bps(&self, firm: FirmId) -> i128 {
        let history = self
            .borrower_credit_history
            .get(&firm)
            .copied()
            .unwrap_or_default();
        let due = i128::from(history.scheduled_due.minor_units()).max(0);
        let paid = i128::from(history.scheduled_paid.minor_units()).clamp(0, due);
        let shortfall_premium = if due == 0 {
            0
        } else {
            due.saturating_sub(paid)
                .saturating_mul(MAX_BORROWER_PAYMENT_SHORTFALL_PREMIUM_BPS)
                .saturating_div(due)
        };
        let delinquency_premium = i128::from(history.delinquent_payments)
            .saturating_mul(BORROWER_DELINQUENCY_PREMIUM_BPS)
            .min(MAX_BORROWER_DELINQUENCY_PREMIUM_BPS);
        let default_premium = i128::from(history.defaulted_loans)
            .saturating_mul(BORROWER_DEFAULT_PREMIUM_BPS)
            .min(MAX_BORROWER_DEFAULT_PREMIUM_BPS);
        let success_discount = i128::from(history.successful_loans)
            .saturating_mul(BORROWER_SUCCESS_DISCOUNT_BPS)
            .min(MAX_BORROWER_SUCCESS_DISCOUNT_BPS);
        shortfall_premium
            .saturating_add(delinquency_premium)
            .saturating_add(default_premium)
            .saturating_sub(success_discount)
    }

    #[must_use]
    pub fn borrower_credit_history(
        &self,
    ) -> &std::collections::BTreeMap<FirmId, BorrowerCreditHistory> {
        &self.borrower_credit_history
    }

    pub(crate) fn record_lender_credit_outcome(
        &mut self,
        creditor: ActorId,
        principal_repaid: Money,
        interest_income: Money,
        realized_loss: Money,
        resolved: bool,
        defaulted: bool,
    ) -> Result<(), WorldError> {
        let history = self.lender_credit_history.entry(creditor).or_default();
        history.principal_repaid = Money::from_minor_units(
            history
                .principal_repaid
                .minor_units()
                .checked_add(principal_repaid.minor_units())
                .ok_or(WorldError::ArithmeticOverflow("lender principal history"))?,
        );
        history.interest_income = Money::from_minor_units(
            history
                .interest_income
                .minor_units()
                .checked_add(interest_income.minor_units())
                .ok_or(WorldError::ArithmeticOverflow("lender interest history"))?,
        );
        history.realized_losses = Money::from_minor_units(
            history
                .realized_losses
                .minor_units()
                .checked_add(realized_loss.minor_units())
                .ok_or(WorldError::ArithmeticOverflow("lender loss history"))?,
        );
        if resolved {
            if defaulted {
                history.defaulted_loans = history.defaulted_loans.saturating_add(1);
            } else {
                history.successful_loans = history.successful_loans.saturating_add(1);
            }
        }
        self.events.append(
            self.date,
            crate::DomainEvent::LenderCreditHistoryUpdated {
                creditor,
                principal_repaid,
                interest_income,
                realized_loss,
                resolved,
                defaulted,
            },
        );
        Ok(())
    }

    #[must_use]
    pub fn lender_credit_history(
        &self,
    ) -> &std::collections::BTreeMap<ActorId, LenderCreditHistory> {
        &self.lender_credit_history
    }

    fn lender_loss_ratio_bps(&self, creditor: ActorId) -> i128 {
        let history = self
            .lender_credit_history
            .get(&creditor)
            .copied()
            .unwrap_or_default();
        let repaid = i128::from(history.principal_repaid.minor_units()).max(0);
        let losses = i128::from(history.realized_losses.minor_units()).max(0);
        let resolved = repaid.saturating_add(losses);
        if resolved == 0 {
            0
        } else {
            losses.saturating_mul(10_000).saturating_div(resolved)
        }
    }

    fn financing_actor(&self, firm: FirmId) -> Option<ActorId> {
        if let Some((_, actor, _)) = self.firm_appointments.keys().find(|(candidate, _, role)| {
            *candidate == firm && *role == CorporateRole::ChiefExecutive
        }) {
            return Some(*actor);
        }
        self.actors().keys().copied().find(|actor| {
            self.can_perform_corporate_action(*actor, firm, CorporateAction::SetOverallPolicy)
        })
    }

    fn firm_country(&self, firm: FirmId) -> Result<crate::CountryId, WorldError> {
        let region = self
            .firms
            .get(&firm)
            .ok_or(WorldError::UnknownFirm(firm))?
            .region();
        self.regions
            .get(&region)
            .map(crate::Region::country)
            .ok_or(WorldError::UnknownRegion(region))
    }

    fn actor_country(&self, actor: ActorId) -> Result<crate::CountryId, WorldError> {
        let region = self
            .actors()
            .get(&actor)
            .ok_or(WorldError::UnknownActor(actor))?
            .home_region();
        self.regions
            .get(&region)
            .map(crate::Region::country)
            .ok_or(WorldError::UnknownRegion(region))
    }
}
