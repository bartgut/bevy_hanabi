//! A particle system with a 2D camera.
//!
//! The particle effect instance override its `z_layer_2d` field, which can be
//! tweaked at runtime via the egui inspector to move the 2D rendering layer of
//! particle above or below the reference square.

use bevy::{
    log::LogPlugin,
    prelude::*,
    render::{
        camera::ScalingMode, render_resource::WgpuFeatures, settings::WgpuSettings, RenderPlugin,
    },
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
#[cfg(feature = "examples_world_inspector")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use noise::{NoiseFn, Perlin};

use bevy_hanabi::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut wgpu_settings = WgpuSettings::default();
    wgpu_settings
        .features
        .set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true);

    let mut app = App::default();
    app.insert_resource(ClearColor(Color::DARK_GRAY))
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    level: bevy::log::Level::WARN,
                    filter: "bevy_hanabi=warn,2d=trace".to_string(),
                    update_subscriber: None,
                })
                .set(RenderPlugin {
                    render_creation: wgpu_settings.into(),
                    synchronous_pipeline_compilation: false,
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "🎆 Hanabi — 2d".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(HanabiPlugin);

    #[cfg(feature = "examples_world_inspector")]
    app.add_plugins(WorldInspectorPlugin::default());

    app.add_systems(Startup, setup)
        .add_systems(Update, (bevy::window::close_on_esc))
        .run();

    Ok(())
}

fn setup(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Spawn a 2D camera
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 1.0;
    camera.projection.scaling_mode = ScalingMode::FixedVertical(1.);
    commands.spawn(camera);

    // Create a color gradient for the particles
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec3::splat(1.).extend(0.0));
    gradient.add_key(0.1, Vec3::splat(1.).extend(0.5));
    gradient.add_key(0.9, Vec3::splat(1.).extend(0.5));
    gradient.add_key(1.0, Vec3::splat(1.).extend(0.0));

    let mut splash_gradient = Gradient::new();
    splash_gradient.add_key(0.0, Vec3::splat(1.).extend(0.0));
    splash_gradient.add_key(0.1, Vec3::splat(1.).extend(0.5));
    splash_gradient.add_key(0.9, Vec3::splat(1.).extend(0.5));
    splash_gradient.add_key(1.0, Vec3::splat(1.).extend(0.0));

    let writer = ExprWriter::new();

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let lifetime = writer.lit(0.8).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let init_pos = SetPositionBoxModifier {
        center: writer.lit(Vec3::new(0.0, 0.5, 0.0)).expr(),
        width: writer.lit(1.8).expr(),
        height: writer.lit(0.1).expr(),
    };

    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, writer.lit(Vec3::new(0.0, -0.5, 0.0)).expr());

    let splash_vel = SetAttributeModifier::new(Attribute::VELOCITY, writer.lit(Vec3::ZERO).expr());
    let splash_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(0.2).expr());

    let mut module = writer.finish();

    let raindrop_size = SetSizeModifier {
        size: CpuValue::Uniform((Vec2::new(0.0025, 0.02), Vec2::new(0.0025, 0.05))),
        ..default()
    };

    let splash_size = SetSizeModifier {
        size: CpuValue::Single((Vec2::new(0.0025, 0.0025))),
        ..default()
    };

    let accel = module.lit(Vec3::new(0., -1., 0.));
    let update_accel = AccelModifier::new(accel);

    let clone_modifier = CloneModifier::new(0.8, 1);
    let splash_color = SetColorModifier {
        color: CpuValue::Single(Vec4::new(0.0, 1.0, 1.0, 1.0)),
    };

    // Create a new effect asset spawning 30 particles per second from a circle
    // and slowly fading from blue-ish to transparent over their lifetime.
    // By default the asset spawns the particles at Z=0.
    let spawner = Spawner::rate(50.0.into());
    let effect = effects.add(
        EffectAsset::new(vec![4096,4096], spawner, module)
            .with_name("2d")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .update_groups(clone_modifier, ParticleGroupSet::single(0))
            .update_groups(update_accel, ParticleGroupSet::single(0))
            .update_groups(splash_vel, ParticleGroupSet::single(1))
            .update_groups(splash_lifetime, ParticleGroupSet::single(1))
            .render_groups(ColorOverLifetimeModifier { gradient }, ParticleGroupSet::single(0))
            .render_groups(ColorOverLifetimeModifier { gradient: splash_gradient }, ParticleGroupSet::single(1))
            .render_groups(raindrop_size, ParticleGroupSet::single(0))
            .render_groups(splash_size, ParticleGroupSet::single(1))
    );

    // Spawn an instance of the particle effect, and override its Z layer to
    // be above the reference white square previously spawned.
    commands
        .spawn(ParticleEffectBundle {
            // Assign the Z layer so it appears in the egui inspector and can be modified at runtime
            effect: ParticleEffect::new(effect).with_z_layer_2d(Some(0.1)),
            ..default()
        })
        .insert(Name::new("effect:2d"));
}
