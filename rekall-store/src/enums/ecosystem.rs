use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(Ecosystem {
    Actions = "ACTIONS",
    Composer = "COMPOSER",
    Erlang = "ERLANG",
    Go = "GO",
    Maven = "MAVEN",
    Npm = "NPM",
    Nuget = "NUGET",
    Pip = "PIP",
    Pub = "PUB",
    Rubygems = "RUBYGEMS",
    Rust = "RUST",
    Swift = "SWIFT"
});

impl Ecosystem {
    /// The ecosystem as the GitHub Advisory Database names it, which is also how a session writes it.
    pub fn advisory_name(self) -> &'static str {
        match self {
            Self::Actions => "actions",
            Self::Composer => "composer",
            Self::Erlang => "erlang",
            Self::Go => "go",
            Self::Maven => "maven",
            Self::Npm => "npm",
            Self::Nuget => "nuget",
            Self::Pip => "pip",
            Self::Pub => "pub",
            Self::Rubygems => "rubygems",
            Self::Rust => "rust",
            Self::Swift => "swift",
        }
    }

    pub fn of_advisory_name(name: &str) -> Option<Self> {
        <Self as sea_orm::Iterable>::iter().find(|ecosystem| ecosystem.advisory_name() == name)
    }

    /// The advisory names, in declaration order, for a refusal that lists what is accepted.
    pub fn advisory_names() -> Vec<&'static str> {
        <Self as sea_orm::Iterable>::iter().map(Self::advisory_name).collect()
    }
}
