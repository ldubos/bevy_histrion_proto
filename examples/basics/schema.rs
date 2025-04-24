mod prototypes;

fn main() {
    use bevy::prelude::*;
    use bevy_histrion_proto::prelude::*;

    use prototypes::*;

    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(bevy_histrion_proto::PrototypesPlugin)
        .add_plugins(PrototypesPlugin);

    let schema = app.get_prototypes_schemas();
    println!("{}", &schema);
    std::fs::write("./.vscode/prototypes.schema.json", &schema).unwrap();
}
