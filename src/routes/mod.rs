// provide an aggregated view for all available modules
mod health_check;
mod subscriptions;
mod subscriptions_confirm;
mod newsletters;
// re-export useful functions
pub use health_check::*;
pub use subscriptions::*;
pub use subscriptions_confirm::*;
pub use newsletters::*;