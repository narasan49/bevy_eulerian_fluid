use bevy::app::Plugin;

use crate::{common_pass::prefix_sum::PrefixSumPass, plugin::FluidComputePassPlugin};

pub mod prefix_sum;

pub(crate) struct CommonPassPlugin;

impl Plugin for CommonPassPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(FluidComputePassPlugin::<PrefixSumPass>::default());
    }
}
