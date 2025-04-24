# Bevy Histrion Proto

**Bevy Histrion Proto** is an opinionated tool designed to facilitate data-driven development with Bevy and assets. It allows you to define game elements using "prototype" files, making it easier to manage and iterate on your game's content.

**Key Features**:
- Define game elements such as objects, characters, and effects using Rust structs.
- Generate JSON schema files to enable autocompletion in VSCode.
- Easily reference assets dependencies within your prototype definitions.

## Getting Started

### Defining Prototypes

Prototypes are defined using Rust structs with specific attributes. Here's an example of how to define a `Sword` and an `Effect`:

```rust
#[derive(Debug, Clone, JsonSchema, Prototype)]
#[proto(discriminant = "sword")]
pub struct Sword {
    pub damage: f32,
    pub level: u32,
    pub effects: Vec<PrototypeId<Effect>>,
    pub icon: Handle<Image>,
}

#[derive(Debug, Clone, JsonSchema, Prototype)]
#[proto(discriminant = "effect")]
pub struct Effect {
    pub damage_multiplier: Option<f32>,
    pub slow_factor: Option<f32>,
    pub slow_duration: Option<f32>,
    pub icon: Handle<Image>,
}
```

### Creating Proto Assets

Once you have defined your prototypes, you can create JSON files to define your game objects. Here's an example of a "proto" asset file:

```json
[
    {
        "type": "sword",
        "name": "mighty_sword",
        "damage": 3000.0,
        "level": 100,
        "effects": [
            "bleeding",
            "freezing"
        ],
        "icon": "mighty_sword_icon.png"
    },
    {
        "type": "effect",
        "name": "bleeding",
        "damage_multiplier": 3.0,
        "icon": "bleeding_effect.png",
    },
    {
        "type": "effect",
        "name": "freezing",
        "slow_factor": 0.5,
        "slow_duration": 3.0,
        "icon": "freezing_effect.png",
    }
]
```

It's also possible to define one prototype per file or to define thems in multiple files, it can be useful for "content packs".

### JSON Schema for Autocompletion

BHP can generate JSON schema files to help you with autocompletion in your IDE. You can find examples here:

- [.vscode/settings.json](./.vscode/settings.json)
- [.vscode/prototypes.schema.json](./.vscode/prototypes.schema.json)
- [bevy_histrion_proto/examples/basics/schema.rs](./examples/basics/schema.rs)

## Examples

Check out the examples in the `examples` directory to see how you can use it in your own projects.

## Features

| feature  | description                                              |
| -------- | -------------------------------------------------------- |
| derive   | ...                                                      |
| schemars | Enables JSON schema generation with the `schemars` crate |

## Bevy Compatibility

| bevy          | bevy_histrion_proto |
| ------------- | ------------------- |
| `0.16.0-rc.x` | `main`              |

## License

Dual-licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](/LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](/LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
