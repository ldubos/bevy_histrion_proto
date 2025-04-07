use bevy::prelude::*;
use bevy_histrion_proto::prelude::*;

mod prototypes;
use prototypes::*;

#[derive(Debug, Clone, Reflect, Default, Resource, Deref)]
struct HaveDlc(bool);

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(bevy_histrion_proto::HistrionProtoPlugin)
        .add_plugins(PrototypesPlugin)
        .insert_resource(HaveDlc(true))
        .add_systems(Startup, load_prototypes)
        .add_systems(Update, (on_new_sword, on_new_effect));

    app.run();
}

fn load_prototypes(asset_server: Res<AssetServer>, have_dlc: Res<HaveDlc>) {
    let _ = asset_server.load_folder("prototypes");

    if **have_dlc {
        let _ = asset_server.load_untyped("mega_dlc.proto.json");
    }
}

fn on_new_sword(
    mut events: EventReader<RegistryEvent<Sword>>,
    swords: Reg<Sword>,
    icons: Res<Assets<Icon>>,
) {
    for event in events.read() {
        match event {
            RegistryEvent::Added(id) => {
                let sword = swords.get(id).unwrap();
                info!(
                    r#"New sword:
    id: {}
    damage: {}
    level: {}
    effects: {:?}
    icon: {}
"#,
                    sword.id,
                    sword.damage,
                    sword.level,
                    sword.effects,
                    icons.get(&sword.icon).unwrap()
                );
            }
            _ => {}
        }
    }
}

fn on_new_effect(
    mut events: EventReader<RegistryEvent<Effect>>,
    effects: Reg<Effect>,
    icons: Res<Assets<Icon>>,
) {
    for event in events.read() {
        match event {
            RegistryEvent::Added(id) => {
                let effect = effects.get(id).unwrap();
                info!(
                    r#"New effect:
    id: {}
    damage_multiplier: x{}
    slow_factor: {}%
    slow_duration: {}s
    icon: {}"#,
                    effect.id,
                    effect.damage_multiplier.unwrap_or(1.0),
                    effect.slow_factor.unwrap_or(0.0) * 100.0,
                    effect.slow_duration.unwrap_or(0.0),
                    icons.get(&effect.icon).unwrap()
                );
            }
            _ => {}
        }
    }
}
