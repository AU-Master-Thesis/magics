#[derive(derive_more::Display)]
#[display(fmt = "hello")]
struct Foo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
struct VariableIndex(usize);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let foo = Foo;

    println!("{}", foo);

    let vi: VariableIndex = 42usize.into();

    println!("{:?}", vi);

    Ok(())
}
