//! One file per section of the context a session reads; each adds its `render_*` methods to
//! `ContextRenderer`, which only composes them.

pub(super) mod text;
mod skills;
mod wrapup;
mod commits;
mod description;
mod connections;
mod components;
mod pages;
mod documents;
mod deviations;
mod thread;
mod flow;
mod steps;
