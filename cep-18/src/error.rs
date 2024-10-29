#[repr(u16)]
pub enum Error {
    InsufficientBalance = 1,
    CliffPeriodNotEnded = 2,
    UnauthorizedSigner = 3,
    ProposalExpired = 4,
    AlreadySigned = 5,
    NoTokensToRelease = 6,
    InsufficientSigners = 7,
    InvalidCategory = 8,
    InvalidAmount = 9,
}
