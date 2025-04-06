use bevy::prelude::*;
use bevy_histrion_proto::prelude::*;

mod prototypes;
use prototypes::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(bevy_histrion_proto::HistrionProtoPlugin)
        .init_asset::<MyAsset>()
        .register_asset_loader(MyAssetLoader)
        .register_prototype::<MyPrototype1>()
        .register_prototype::<MyPrototype2>()
        .register_prototype::<ProtoWithAssets>()
        .add_systems(Startup, |asset_server: Res<AssetServer>| {
            let _ = asset_server.load_folder("prototypes");
        })
        .add_systems(
            Update,
            |my_proto1_reg: Reg<MyPrototype1>,
             my_proto2_reg: Reg<MyPrototype2>,
             with_assets_reg: Reg<ProtoWithAssets>,
             mut exit_tx: EventWriter<AppExit>| {
                let hello = with_assets_reg.get_by_name("hello");
                let world = my_proto1_reg.get_by_name("world");
                let alone = my_proto2_reg.get_by_name("alone");

                if hello.is_some() && world.is_some() && alone.is_some() {
                    info!("{:#?}", hello.unwrap());
                    info!("{:#?}", world.unwrap());
                    info!("{:#?}", alone.unwrap());

                    exit_tx.write(AppExit::Success);
                }
            },
        );

    app.run();
}
