// provide an aggregated view for all available modules
mod health_check;
mod subscriptions;
mod subscriptions_confirm;
mod newsletters;
mod home;
mod login;
// re-export useful functions
pub use health_check::*;
pub use subscriptions::*;
pub use subscriptions_confirm::*;
pub use newsletters::*;
pub use home::*;
pub use login::*;