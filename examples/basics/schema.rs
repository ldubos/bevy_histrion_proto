mod prototypes;

#[cfg(not(feature = "schemars"))]
fn main() {
    panic!("run this example with --features schemars");
}

#[cfg(feature = "schemars")]
fn main() {
    use bevy::prelude::*;
    use bevy_histrion_proto::prelude::*;

    use prototypes::*;

    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(bevy_histrion_proto::HistrionProtoPlugin)
        .register_prototype::<MyPrototype1>()
        .register_prototype::<MyPrototype2>()
        .register_prototype::<ProtoWithAssets>();

    let schema = serde_json::to_string_pretty(&app.get_prototypes_schema()).unwrap();

    println!("{}", &schema);

    std::fs::write("./.vscode/prototypes.schema.json", &schema).unwrap();
}
