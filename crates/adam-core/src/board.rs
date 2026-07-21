use crate::{
    ActorId, BasisPoints, CorporateAction, CorporateRole, DomainEvent, FirmId, ResolutionId, World,
    WorldError,
};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BoardVote {
    For,
    Against,
    Abstain,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ResolutionStatus {
    Open,
    Approved,
    Rejected,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BoardResolution {
    id: ResolutionId,
    firm: FirmId,
    action: CorporateAction,
    proposer: ActorId,
    votes: BTreeMap<ActorId, BoardVote>,
    status: ResolutionStatus,
}
impl BoardResolution {
    #[must_use]
    pub const fn new(
        id: ResolutionId,
        firm: FirmId,
        action: CorporateAction,
        proposer: ActorId,
    ) -> Self {
        Self {
            id,
            firm,
            action,
            proposer,
            votes: BTreeMap::new(),
            status: ResolutionStatus::Open,
        }
    }
    #[must_use]
    pub const fn id(&self) -> ResolutionId {
        self.id
    }
    #[must_use]
    pub const fn firm(&self) -> FirmId {
        self.firm
    }
    #[must_use]
    pub const fn action(&self) -> CorporateAction {
        self.action
    }
    #[must_use]
    pub const fn proposer(&self) -> ActorId {
        self.proposer
    }
    #[must_use]
    pub fn votes(&self) -> &BTreeMap<ActorId, BoardVote> {
        &self.votes
    }
    #[must_use]
    pub const fn status(&self) -> ResolutionStatus {
        self.status
    }
    /// Records or replaces a director's vote while the resolution is open.
    /// # Errors
    /// Returns an error for a non-director or closed resolution.
    pub fn cast_vote(
        &mut self,
        directors: &BTreeSet<ActorId>,
        actor: ActorId,
        vote: BoardVote,
    ) -> Result<(), &'static str> {
        if self.status != ResolutionStatus::Open {
            return Err("resolution is closed");
        }
        if !directors.contains(&actor) {
            return Err("actor is not a director");
        }
        self.votes.insert(actor, vote);
        Ok(())
    }
    pub fn close(
        &mut self,
        directors: &BTreeSet<ActorId>,
        quorum: BasisPoints,
        approval: BasisPoints,
    ) -> ResolutionStatus {
        let total = directors.len() as u64;
        let cast = self.votes.len() as u64;
        if total == 0 || cast * 10_000 < total * u64::from(quorum.get()) {
            self.status = ResolutionStatus::Rejected;
            return self.status;
        }
        let for_votes = self
            .votes
            .values()
            .filter(|v| **v == BoardVote::For)
            .count() as u64;
        let decisive = self
            .votes
            .values()
            .filter(|v| **v != BoardVote::Abstain)
            .count() as u64;
        self.status = if decisive > 0 && for_votes * 10_000 >= decisive * u64::from(approval.get())
        {
            ResolutionStatus::Approved
        } else {
            ResolutionStatus::Rejected
        };
        self.status
    }
}
impl World {
    /// Proposes a board resolution through executive or director authority.
    /// # Errors
    /// Returns an error for duplicate IDs, unknown firms, or unauthorized proposers.
    pub fn propose_board_resolution(
        &mut self,
        resolution: BoardResolution,
    ) -> Result<(), WorldError> {
        if self.board_resolutions.contains_key(&resolution.id()) {
            return Err(WorldError::DuplicateBoardResolution(resolution.id()));
        }
        if !self.firms().contains_key(&resolution.firm()) {
            return Err(WorldError::UnknownFirm(resolution.firm()));
        }
        let authorized = self.firm_appointments().contains_key(&(
            resolution.firm(),
            resolution.proposer(),
            CorporateRole::ChiefExecutive,
        )) || self.firm_appointments().contains_key(&(
            resolution.firm(),
            resolution.proposer(),
            CorporateRole::BoardDirector,
        ));
        if !authorized {
            return Err(WorldError::UnauthorizedBoardAction(resolution.proposer()));
        }
        self.events.append(
            self.date,
            DomainEvent::BoardResolutionProposed {
                resolution: resolution.id(),
                firm: resolution.firm(),
                proposer: resolution.proposer(),
            },
        );
        self.board_resolutions.insert(resolution.id(), resolution);
        Ok(())
    }
    /// Casts a vote using current board membership.
    /// # Errors
    /// Returns an error for unknown resolution, non-director, or closed vote.
    pub fn cast_board_vote(
        &mut self,
        id: ResolutionId,
        actor: ActorId,
        vote: BoardVote,
    ) -> Result<(), WorldError> {
        let firm = self
            .board_resolutions
            .get(&id)
            .ok_or(WorldError::UnknownBoardResolution(id))?
            .firm();
        let directors = self.directors(firm);
        self.board_resolutions
            .get_mut(&id)
            .ok_or(WorldError::UnknownBoardResolution(id))?
            .cast_vote(&directors, actor, vote)
            .map_err(WorldError::InvalidBoardVote)?;
        self.events.append(
            self.date,
            DomainEvent::BoardVoteCast {
                resolution: id,
                actor,
                vote,
            },
        );
        Ok(())
    }
    /// Closes a resolution under current board membership and thresholds.
    /// # Errors
    /// Returns an error for an unknown resolution.
    pub fn close_board_resolution(
        &mut self,
        id: ResolutionId,
        quorum: BasisPoints,
        approval: BasisPoints,
    ) -> Result<ResolutionStatus, WorldError> {
        let firm = self
            .board_resolutions
            .get(&id)
            .ok_or(WorldError::UnknownBoardResolution(id))?
            .firm();
        let directors = self.directors(firm);
        let status = self
            .board_resolutions
            .get_mut(&id)
            .ok_or(WorldError::UnknownBoardResolution(id))?
            .close(&directors, quorum, approval);
        self.events.append(
            self.date,
            DomainEvent::BoardResolutionClosed {
                resolution: id,
                status,
            },
        );
        Ok(status)
    }
    #[must_use]
    pub fn board_resolutions(&self) -> &BTreeMap<ResolutionId, BoardResolution> {
        &self.board_resolutions
    }
    fn directors(&self, firm: FirmId) -> BTreeSet<ActorId> {
        self.firm_appointments()
            .keys()
            .filter(|(f, _, role)| *f == firm && *role == CorporateRole::BoardDirector)
            .map(|(_, actor, _)| *actor)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quorum_and_majority_are_both_required() {
        let directors = BTreeSet::from([ActorId::new(1), ActorId::new(2), ActorId::new(3)]);
        let mut r = BoardResolution::new(
            ResolutionId::new(1),
            FirmId::new(1),
            CorporateAction::ApproveMajorInvestment,
            ActorId::new(1),
        );
        r.cast_vote(&directors, ActorId::new(1), BoardVote::For)
            .expect("vote");
        assert_eq!(
            r.close(
                &directors,
                BasisPoints::new(6667).expect("q"),
                BasisPoints::new(5001).expect("a")
            ),
            ResolutionStatus::Rejected
        );
    }
    #[test]
    fn qualified_vote_approves() {
        let directors = BTreeSet::from([ActorId::new(1), ActorId::new(2), ActorId::new(3)]);
        let mut r = BoardResolution::new(
            ResolutionId::new(1),
            FirmId::new(1),
            CorporateAction::DeclareDividend,
            ActorId::new(1),
        );
        for id in [1, 2, 3] {
            r.cast_vote(
                &directors,
                ActorId::new(id),
                if id < 3 {
                    BoardVote::For
                } else {
                    BoardVote::Against
                },
            )
            .expect("vote");
        }
        assert_eq!(
            r.close(
                &directors,
                BasisPoints::new(6667).expect("q"),
                BasisPoints::new(5001).expect("a")
            ),
            ResolutionStatus::Approved
        );
    }
}
