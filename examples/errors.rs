use bevy_rich_text3d::{ParseBuilder, Text3d};

pub fn main() {
    let (_, error) = Text3d::parse_with_errors("Hello, {World}!", ParseBuilder::new());
    println!("{}", error.unwrap());

    let (_, error) = Text3d::parse_with_errors("Hello, {W\no\nr\nl\nd}!", ParseBuilder::new());
    println!("{}", error.unwrap());

    let (_, error) = Text3d::parse_with_errors("{red: Hello, World!}}", ParseBuilder::new());
    println!("{}", error.unwrap());

    let (_, error) = Text3d::parse_with_errors("Hello, {greeen: World}!", ParseBuilder::new());
    println!("{}", error.unwrap());

    let (_, error) = Text3d::parse_with_errors(
        "Hello,\n beautiful {?unknown:World} of bevy\n!",
        ParseBuilder::new(),
    );
    println!("{}", error.unwrap());

    let (_, error) = Text3d::parse_with_errors(
        "Hello,\n beautiful {?!unknown:World} of bevy\n!",
        ParseBuilder::new(),
    );
    println!("{}", error.unwrap());
}
