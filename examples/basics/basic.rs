use bevy::{log::LogPlugin, prelude::*};
use bevy_histrion_proto::prelude::*;

mod prototypes;
use prototypes::*;

#[derive(Debug, Clone, Reflect, Default, Resource, Deref)]
struct HaveDlc(bool);

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(LogPlugin {
        level: bevy::log::Level::TRACE,
        ..default()
    }))
    .add_plugins(bevy_histrion_proto::PrototypesPlugin)
    .add_plugins(PrototypesPlugin)
    .insert_resource(HaveDlc(true))
    .add_systems(Startup, load_prototypes)
    .add_systems(Update, on_new_sword);

    app.run();
}

fn load_prototypes(mut prototype_server: PrototypeServer, have_dlc: Res<HaveDlc>) {
    prototype_server.load_prototypes_folder("prototypes");

    if have_dlc.0 {
        prototype_server.load_prototypes("./mega_dlc.proto.json");
    }
}

fn on_new_sword(swords: Reg<Sword>, icons: Res<Assets<Icon>>) {
    if let Some(sword) = swords.get("wooden_stick") {
        info!(
            r#"New sword:
            name: {}
            damage: {}
            level: {}
            effects: {:?}
            icon: {}
        "#,
            sword.name(),
            sword.damage,
            sword.level,
            sword.effects,
            icons.get(&sword.icon).unwrap()
        );
    }
}

// fn on_new_effect(
//     mut events: EventReader<RegistryEvent<Effect>>,
//     effects: Reg<Effect>,
//     icons: Res<Assets<Icon>>,
// ) {
//     for event in events.read() {
//         if let RegistryEvent::Added(id) = event {
//             let effect = effects.get(id).unwrap();
//             info!(
//                 r#"New effect:
//         id: {}
//         damage_multiplier: x{}
//         slow_factor: {}%
//         slow_duration: {}s
//         icon: {}"#,
//                 effect.id,
//                 effect.damage_multiplier.unwrap_or(1.0),
//                 effect.slow_factor.unwrap_or(0.0) * 100.0,
//                 effect.slow_duration.unwrap_or(0.0),
//                 icons.get(&effect.icon).unwrap()
//             );
//         }
//     }
// }
