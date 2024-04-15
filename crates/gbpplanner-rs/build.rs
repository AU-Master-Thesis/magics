#![allow(missing_docs)]

fn main() {
    let target = std::env::var("TARGET").expect("set by `trunk`");
    if target.contains("windows") {
        // on windows we will set our game icon as icon for the executable
        embed_resource::compile("build/windows/icon.rc", embed_resource::NONE);
    }
}
