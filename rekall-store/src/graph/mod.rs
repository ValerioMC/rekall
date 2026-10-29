//! A project's deployment graphs: steps routed by outcome, each reaching one connection, kept to
//! be laid on a task whenever its work has to be released the same way.

pub mod entity;
pub mod repository;
