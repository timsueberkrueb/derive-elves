/*!
Writing inclusive derive macros is tedious,
this creates provides helper functions that make it easier.

# type aware impl
The `type_aware_impl` function makes it easy to write derive macros
that take the generics of the underlying type into consideration.
## Example
Considering this simple derive macro.
```ignore
#[proc_macro_derive(Append)]
pub fn push(input_stream: TokenStream) -> TokenStream {
    let input_type = parse_macro_input!(input_stream as DeriveInput);

    let ident = &input_type.ident;

    type_aware_impl(
        quote! {
            impl<T: Append<T>> Append<T> for #ident {
                fn append(&self, l: T) {
                    todo!()
                }
            }
        },
        &input_type,
    )
}
```
The the following anotated struct,
```ignore
#[derive(Append)]
struct Foo<S: ToString> {
    bar: S
}
```
would expand to this:
```ignore
struct Foo<S: ToString> {
    bar: S
}

impl<T: Append<T>, S: ToString> Append<T> for Foo<S> {
    fn append(&self, l: T) {
        todo!()
    }
}
```
The above also works for more complex patterns,
like the following:
```ignore
impl Trait for & #ident
```
```ignore
impl Trait for &mut #ident
```
```ignore
impl Trait for [#ident]
```
```ignore
impl Trait for (#ident, A, B, C)
```
*/

pub use type_aware_impl::*;

mod type_aware_impl;
