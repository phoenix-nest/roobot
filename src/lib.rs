use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};

mod icon_updater;
mod util;

pub(crate) trait Load {
    type State: for<'a> Deserialize<'a>;
    fn load(self, state: &Self::State) -> Result<impl Module>;
}

pub(crate) trait Module {
    type State: Serialize;

    async fn save(&self) -> Self::State;
    async fn shutdown(self) -> Self::State;
}
