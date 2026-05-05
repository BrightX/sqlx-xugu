use std::fmt;
use std::fmt::{Display, Formatter};
use std::num::NonZeroU32;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct StatementId(IdInner);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct IdInner(NonZeroU32);

pub(crate) struct DisplayId {
    prefix: &'static str,
    id: NonZeroU32,
}

impl StatementId {
    pub const NAMED_START: Self = Self(IdInner::NAMED_START);

    const NAME_PREFIX: &'static str = "st_sqlx_";

    pub fn next(&self) -> Self {
        Self(self.0.next())
    }

    /// Get a type to format this statement ID with [`Display`].
    ///
    /// Returns `None` if this is the unnamed statement.
    #[inline(always)]
    pub fn display(&self) -> DisplayId {
        self.0.display(Self::NAME_PREFIX)
    }
}

impl Display for StatementId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl Display for DisplayId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.prefix, self.id)
    }
}

impl IdInner {
    const NAMED_START: Self = Self(NonZeroU32::MIN);

    #[inline(always)]
    fn next(&self) -> Self {
        Self(self.0.checked_add(1).unwrap_or(NonZeroU32::MIN))
    }

    #[inline(always)]
    fn display(&self, prefix: &'static str) -> DisplayId {
        DisplayId { prefix, id: self.0 }
    }
}
