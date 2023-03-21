// This should hold queue service logic:
// - Which job to return from Pop, based on priority, still/animation, max_workers per customer, etc.
// - GetScaleTarget logic based on same things.

// Store entities like Render, Job, Customer, etc. in the database with a repository contract.

pub mod entity;
pub mod load_balance;
pub mod repository;
