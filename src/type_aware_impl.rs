use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::Token;
use syn::punctuated::Punctuated;
use syn::DeriveInput;
use syn::ItemImpl;
use syn::Path;
use syn::Type;
use syn::TypeParamBound;

/**
    Parses the input as a [`ItemImpl`] and adds the necessary generics for the `target_type`.
*/
pub fn type_aware_impl(input: TokenStream, target_type: &DeriveInput) -> TokenStream {
    match syn::parse::<ItemImpl>(input.into()) {
        Ok(mut impl_statement) => {
            impl_statement
                .generics
                .params
                .extend(target_type.generics.params.clone());

            add_type_generics(&mut impl_statement.self_ty, target_type);

            if let Some(ref where_clause) = target_type.generics.where_clause {
                if let Some(ref mut impl_where_clause) = impl_statement.generics.where_clause {
                    impl_where_clause
                        .predicates
                        .extend(where_clause.predicates.clone())
                }
            }

            impl_statement.to_token_stream().into()
        }
        Err(err) => err.into_compile_error(),
    }
}

fn add_type_generics(impl_type: &mut Box<Type>, target_type: &DeriveInput) {
    let recurse = |impl_type: &mut Box<Type>| add_type_generics(impl_type, target_type);

    let boxed = |not_boxed_type: &mut Type| {
        let mut boxed_type = Box::new(not_boxed_type.clone());
        recurse(&mut boxed_type);
        *not_boxed_type = Box::<Type>::into_inner(boxed_type);
    };

    let path = |path: &mut Path| {
        if path.is_ident(&target_type.ident) {
            let ident = &target_type.ident;
            let (_, type_generics, _) = target_type.generics.split_for_impl();
            *path = syn::parse(quote! {#ident #type_generics}.into()).unwrap();
        }
        // some stuff to be done
    };

    let bounds = |bounds: &mut Punctuated<TypeParamBound, Token!(+)>| {
        bounds
            .iter_mut()
            .map(|bound| {
                if let TypeParamBound::Trait(trait_bound) = bound {
                    path(&mut trait_bound.path)
                }
            })
            .for_each(drop)
    };

    match **impl_type {
        syn::Type::Array(ref mut type_array) => recurse(&mut type_array.elem),
        syn::Type::BareFn(ref mut bare_fn) => {
            bare_fn
                .inputs
                .iter_mut()
                .map(|argument| boxed(&mut argument.ty))
                .for_each(drop);

            match bare_fn.output {
                syn::ReturnType::Type(_, ref mut return_type) => recurse(return_type),
                syn::ReturnType::Default => (),
            }
        }
        syn::Type::Group(ref mut group) => recurse(&mut group.elem),
        syn::Type::ImplTrait(ref mut impl_trait) => bounds(&mut impl_trait.bounds),
        syn::Type::Paren(ref mut paren) => recurse(&mut paren.elem),
        syn::Type::Path(ref mut type_path) => {
            if let Some(ref mut q_self) = type_path.qself {
                recurse(&mut q_self.ty)
            }
            path(&mut type_path.path)
        }
        syn::Type::Ptr(ref mut ptr) => recurse(&mut ptr.elem),
        syn::Type::Reference(ref mut reference) => recurse(&mut reference.elem),
        syn::Type::Slice(ref mut slice) => recurse(&mut slice.elem),
        syn::Type::TraitObject(ref mut trait_object) => bounds(&mut trait_object.bounds),
        syn::Type::Tuple(ref mut tuple) => tuple.elems.iter_mut().map(boxed).for_each(drop),
        _ => (),
    };
}

