use sov::StructOfVecs;

fn main() {
    let mut foos = VecFoo::new();

    foos.push(Foo {
        x: 123,
        y: String::from("asdf"),
    });
    let FooRef { x, y } = foos.get(0);

    println!("{x} {y}");
}

#[derive(StructOfVecs)]
struct Foo {
    x: i32,
    y: String,
}

