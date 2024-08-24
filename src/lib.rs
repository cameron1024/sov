//! Generate a struct representing a `Vec` of the underlying type, except that it is represented in
//! memory as a single struct containing `Vec`s for each field:
//! ```rust
//! # use sov::StructOfVecs;
//! #[derive(StructOfVecs)]
//! struct Foo {
//!     x: u64,
//!     y: String,
//! }
//! ```
//! will generate a struct which looks roughly like this:
//! ```rust
//! struct VecFoo {
//!     x: Vec<u64>
//!     y: Vec<String>,
//! }
//! ```
//! This type, while unrelated to `Vec` from the compiler's point of view, provides an API similar
//! to `Vec`:
//! ```rust
//! # use sov::StructOfVecs;
//! #[derive(StructOfVecs)]
//! struct Foo {
//!     x: u64,
//!     y: String,
//! }
//!
//! fn main() {
//!     let mut vec = VecFoo::new();
//!
//!     for i in 0..1000 {
//!         vec.push(Foo {
//!             x: i,
//!             y: format!("hello {i}"),
//!         });
//!     }
//!
//!     println!("{}", vec.len());
//!
//!     // equivalent to &vec[0]
//!     let first = vec.get(0);
//!     assert_eq!(first.x, 0);
//! }
//! ```
//! ## Why would you want to do this?
//!
//! The short answer is "performance". This representation has better performance under certain
//! circumstances, and is also often able to consume less memory than the equivalent representation
//! using `Vec`.
//!
//! If you care about performance, you should benchmark. If benchmarking is too much effort, you
//! probably don't care enough about performance to justify using this over `Vec`, which has
//! several advantages over this crate (e.g. wider ecosystem support, more thoroughly reviewed code).
//!
//! ### How can it be faster?
//!
//! Consider the following code:
//! ```rust
//! # fn generate_random_vec() -> Vec<Foo> { vec![] }
//! struct Foo {
//!   x: u64,
//!   y: u8,
//! }
//!
//! fn main() {
//!     let vec: Vec<Foo> = generate_random_vec();
//!     let sum_of_xs: u64 = vec.iter().map(|foo| foo.x).sum();
//!     println!("{sum_of_xs}");
//! }
//! ```
//!
//! Since `Vec` lays out each element contiguously in memory, in order to read all the `x`s, we
//! need to read 8 bytes, then skip over 8 bytes (1 byte for `y`, and then 7 padding bytes), then
//! repeat.
//!
//! This is fine, but it's not great for our CPU cache. Accessing main memory is a relatively slow
//! operation by a modern CPU's standards. A good rule of thumb is that a L1 cache lookup takes ~1
//! nanosecond, and a main memory read takes ~100 nanoseconds. At the same time, a single addition
//! often takes significantly less than 1 nanosecond, even before considering things like SIMD. If
//! we had to go to main memory every time we read a value, we would spend potentially 99.9% of our
//! time in this code *just reading memory*.
//!
//! CPU caches are much faster, but also much smaller than main memory, so avoiding wasting cache
//! space is important for performance. Caches also typically load data in 64-byte chunks, known as
//! "cache lines". In the above addition example, when we load the first `x`, we're also loading
//! 8 bytes of "junk" (`y` + padding) into our cache, and so on. Effectively, we're losing half of
//! our cache.
//!
//! However, compare this to the following:
//! ```
//! # fn generate_random_vec() -> VecFoo { VecFoo { x: vec![], y: vec![] } }
//! struct VecFoo {
//!     x: Vec<u64>,
//!     y: Vec<u8>,
//! }
//!
//! fn main() {
//!     let vec: VecFoo = generate_random_vec();
//!     let sum_of_xs: u64 = vec.x.iter().copied().sum();
//! }
//! ```
//! Now, we're iterating over a `Vec<u64>`, which contains only the data we're interested in. The
//! data for `y` is stored in a totally separate heap allocation.
//!
//! As a result, we get to use the full power of our CPU cache, which gives better performance.
//!
//! A separate (but related) benefit is that this kind of operation can often be more effectively
//! vectorized, whether that be automatically by the compiler, or manually by a programmer
//! explicitly using SIMD. [Godbolt][godbolt] shows that this example auto-vectorizes when using
//! the struct-of-vecs approach, but not when summing naively.
//!
//! When benchmarking this example, I get the following results on my machine when summing 100,000
//! elements:
//!  - using `VecFoo` to sum the `x`s, I see a ~3x speedup compared to `Vec<Foo>`
//!  - using `VecFoo` to sum the `y`s, I see a ~50x speedup compared to `Vec<Foo>`
//!
//! These results are quite dramatic, but they are specifically chosen because they are exactly the
//! kind of problem this crate aims to solve. This is not to say that you will always see such
//! extreme performance improvements. For example, the following code will likely see no
//! performance improvement compared to the naive equivalent (or even worse!):
//! ```rust
//! # fn generate_random_vec() -> VecBar { VecBar {x: vec![], y: vec![], } }
//! # use sov::StructOfVecs;
//! #[derive(StructOfVecs)]
//! struct Bar {
//!     x: u64,
//!     y: u64,
//! }
//!
//! fn main() {
//!     let vec: VecBar = generate_random_vec();
//!     let mut sum = 0;
//!
//!     sum += vec.xs().iter().copied().sum::<u64>();
//!     sum += vec.ys().iter().copied().sum::<u64>();
//!
//!     println!("{sum}");
//!
//! }
//! ```
//! As always, the only way to know is to benchmark.
//!
//! ### How can it use less memory?
//!
//! As alluded to above, different types have different "alignment", which says which memory
//! addresses are valid for it to occupy. For example, `u64` has 8-byte alignment, which means it
//! is only allowed to exist at memory addresses which are a multiple of 8.
//!
//! When you make a struct, the compiler automatically inserts padding so that every field can be
//! aligned properly. However, when using this crate, there's no need for padding bytes (in the
//! `VecFoo` representation, that is), since every field is stored separately and stored
//! contiguously, as guaranteed by `Vec`.
//!
//! [godbolt]: https://godbolt.org/z/YqMz8G6K9
use codegen::codegen;
use proc_macro2::TokenStream;
use syn::{parse_macro_input, DeriveInput};

macro_rules! bail {
    ($span:expr => $($t:tt)*) => {
        return Err(quote::quote_spanned! { $span.span() => compile_error!($($t)*)})
    };
    ($($t:tt)*) => {
        return Err(quote::quote! { compile_error!($($t)*)})
    };
}

mod codegen;
mod parse;
mod util;

struct Foo {
    x: u64,
    y: String,
}

pub(crate) type Result<T> = core::result::Result<T, TokenStream>;

/// Generate a struct of `Vec`s. See the crate-level docs for more detail
#[proc_macro_derive(StructOfVecs, attributes(sov))]
pub fn struct_of_vecs(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    match parse::Input::from_derive_input(input) {
        Ok(input) => codegen(input).into(),
        Err(tokens) => tokens.into(),
    }
}
