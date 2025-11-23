# bevy_eulerian_fluid

This project is a fluid simulation plugin for [Bevy](https://bevyengine.org/).

![img](./docs/bevy-fluid-various-shape-rigid-bodies.gif)

Try it on [here](https://narasan49.github.io/bevy_eulerian_fluid/)!

## Basic Usage
1. Add `FluidPlugin` and `PhysicsPlugins` to the app with the same length unit.
2. Spawn `FluidSettings`, then `FluidSimulationBundle` will be inserted automatically to the entity. By querying components `FluidTextures`, the simulation results can be retreived.  

Here is a short example. See [examples](./examples/) for the detailed implementation!  

```rust
use avian2d::PhysicsPlugins;
use bevy_eulerian_fluid::{
    settings::{FluidSettings, FluidTextures},
    FluidPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FluidPlugin::new(10.0))
        .add_plugins(PhysicsPlugins::default().with_length_unit(10.0))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, on_initialized)
        .run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(FluidSettings {
        rho: 997f32, // water
        gravity: Vec2::Y,
        size: UVec2::new(512),
        initial_fluid_level: 0.9,
    });
}

fn on_initialized(
    mut commands: Commands,
    query: Query<(Entity, &FluidTextures), Added<FluidTextures>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut velocity_materials: ResMut<Assets<VelocityMaterial>>,
) {
    for (entity, fluid_textures) in &query {
        // Implement your own code to visualize the results.
    }
}
```

### Interact to the fluid
The simulation entity has `LocalForces` component, which holds arrays of forces (in m/s^2) and position (in pixels). forces can be applied to the simulation domain by setting `LocalForces`.

See also an [interaction example](./examples/interaction.rs) for the detailed implementation.

## Features
- [x] Incompressible 2D fluid simulation
- [ ] Viscosity
- [x] Fluid surface
- [ ] Fluid source/drain
- [x] Solid body interaction
  - [x] One-way solid body to fluid interaction
  - [x] Two-way coupling with solid body and fluid
  - [x] Various shape support: Circle, Rectangle, Capsule, Triangle

## Examples
There are some examples to demonstrate how to visualize and interact to the simulation results:  
- **Fluid-Solid two-way interaction**

  ```ps1
  cargo run --example various_shapes
  ```
  ![img](./docs/bevy-fluid-various-shape-rigid-bodies.gif)

- **Imposing forces with mouse and touch input**
  (Also available [here](https://narasan49.github.io/bevy_eulerian_fluid/))
  ```ps1
  cargo run --example interaction
  ```
  ![img](./docs/bevy-fluid-interaction.gif)

## Versions
| Bevy | Bevy Eularian Fluid |
| --- | --- |
| 0.17 | 0.2, 0.3 |
| 0.15 | 0.1 |

## Acknowledgments
The simulation is inspired by and based on the algorithms described in these books:

- [Fluid Simulation for Computer Graphics](https://www.amazon.co.jp/dp/1482232839) by Robert Bridson
- [GPU Gems Chapter 38](https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-38-fast-fluid-dynamics-simulation-gpu) by Mark J. Harris

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
