//! The two protocol eras, split at `2026-07-28`: up to `2025-11-25` a client opens with an
//! `initialize` handshake; from `2026-07-28` every request carries its own revision.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolVersion {
    V2026_07_28,
    V2025_11_25,
    V2025_06_18,
    V2025_03_26,
    V2024_11_05,
}

impl ProtocolVersion {
    const ALL: [ProtocolVersion; 5] = [
        Self::V2026_07_28,
        Self::V2025_11_25,
        Self::V2025_06_18,
        Self::V2025_03_26,
        Self::V2024_11_05,
    ];

    /// What a request with no revision in its header or `_meta` is read as.
    pub const ASSUMED_WHEN_HEADER_ABSENT: ProtocolVersion = Self::V2025_03_26;

    pub fn wire(&self) -> &'static str {
        match self {
            Self::V2026_07_28 => "2026-07-28",
            Self::V2025_11_25 => "2025-11-25",
            Self::V2025_06_18 => "2025-06-18",
            Self::V2025_03_26 => "2025-03-26",
            Self::V2024_11_05 => "2024-11-05",
        }
    }

    pub fn is_modern(&self) -> bool {
        *self == Self::V2026_07_28
    }

    /// `2024-11-05` is still served to a client that opens with it, but never offered: this
    /// server has never spoken that revision's HTTP+SSE transport.
    fn advertised(&self) -> bool {
        *self != Self::V2024_11_05
    }

    pub fn parse(wire: Option<&str>) -> Option<Self> {
        let wire = wire?;
        Self::ALL.into_iter().find(|version| version.wire() == wire)
    }

    pub fn advertised_versions() -> Vec<&'static str> {
        Self::ALL.into_iter().filter(|v| v.advertised()).map(|v| v.wire()).collect()
    }

    pub fn latest_legacy() -> Self {
        Self::V2025_11_25
    }
}
