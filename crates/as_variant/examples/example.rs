use as_variant::AsVariant;

#[derive(Debug)]
struct Foo {
    a: i32,
    b: bool,
}

#[derive(Debug)]
struct Bar {
    c: String,
    d: [usize; 2],
}

#[derive(Debug, AsVariant)]
enum Example {
    // NoData,
    Data(Foo),
    Bar(Bar),
    Tuple((Foo, Bar)),
    // Tuple(Foo, Bar),
    NoData,
    UnnamedVariantData { first: usize, second: usize },
}

impl Example {
    // fn as_data(&self) -> Option<&Foo> {
    //     if let Self::Data(v) = self {
    //         Some(v)
    //     } else {
    //         None
    //     }
    // }

    // fn as_tuple(&self) -> Option<&(Foo, Bar)> {
    //     if let Self::Tuple(v) = self {
    //         Some(v)
    //     } else {
    //         None
    //     }
    // }

    // fn as_tuple_mut(&mut self) -> Option<&mut (Foo, Bar)> {
    //     if let Self::Tuple(v) = self {
    //         Some(v)
    //     } else {
    //         None
    //     }
    // }
}

fn main() {
    let mut example = Example::Data(Foo { a: 1, b: true });

    {
        let Some(foo) = example.as_data() else {
            panic!("not Data variant");
        };
    }

    if let Some(foo) = example.as_data_mut() {
        foo.a = 42;
    }

    // eprintln!("{}", foo.a);

    dbg!(&example);

    // panic!("asdsad");
}
