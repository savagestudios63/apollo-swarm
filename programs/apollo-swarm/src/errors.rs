use anchor_lang::prelude::*;

#[error_code]
pub enum ApolloError {
    #[msg("String exceeds maximum allowed length")]
    StringTooLong,

    #[msg("Agent is not active")]
    AgentInactive,

    #[msg("Signer is not the agent authority")]
    UnauthorizedAgent,

    #[msg("Signer is not the swarm authority")]
    UnauthorizedSwarm,

    #[msg("Agent is already a member of the swarm")]
    AlreadyMember,

    #[msg("Agent is not a member of the swarm")]
    NotMember,

    #[msg("Total share basis points would exceed 10_000")]
    ShareOverflow,

    #[msg("Invalid task state transition")]
    InvalidTaskState,

    #[msg("Agent role does not match required task role")]
    RoleMismatch,

    #[msg("Task executor mismatch")]
    ExecutorMismatch,

    #[msg("Approval threshold not yet met")]
    ThresholdNotMet,

    #[msg("Approver has already approved this task")]
    DuplicateApproval,

    #[msg("Insufficient funds in treasury")]
    InsufficientTreasury,

    #[msg("Coordination model does not support this action")]
    UnsupportedCoordination,

    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Swarm member count is at capacity for this operation")]
    InvalidMemberCount,
}
