//! Experimental helper macros for use with [`async_graphql`].
//!
//! # Example
//! ```no_run
//! use std::convert::Infallible;
//! use async_graphql::*;
//! use async_graphql_extras::graphql_object;
//! use warp::Filter;
//!
//! type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;
//!
//! struct Query;
//!
//!
//! /// Information about the user
//! #[graphql_object]
//! pub struct UserData {
//!     username: String,
//!     display_name: String,
//! }
//!
//! #[Object]
//! impl Query {
//!     /// Ping endpoint that returns the same object as the input
//!     async fn ping(&self, user_input: UserDataInput) -> UserData {
//!         UserData {
//!             username: user_input.username,
//!             display_name: user_input.display_name,
//!         }
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
//!     let filter = async_graphql_warp::graphql(schema).and_then(
//!         |(schema, request): (MySchema, async_graphql::Request)| async move {
//!             // Execute query
//!             let resp = schema.execute(request).await;
//!
//!             // Return result
//!             Ok::<_, Infallible>(async_graphql_warp::Response::from(resp))
//!         },
//!     );
//!     warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
//! }

extern crate proc_macro;

use darling::{FromField, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, AttributeArgs, ItemStruct, Type};

/// Options to the [`graphql_object`] macro
#[derive(Debug, FromMeta)]
struct GraphqlObjectMetaArgs {
    /// Customized doc string for the InputObject version of the struct
    #[darling(default)]
    input_object_doc: Option<String>,

    /// Skips deriving `SimpleObject` on the struct so that the user can do it manually
    #[darling(default)]
    skip_derive_simple_object: bool,
}

/// Options on fields for the [`graphql_object`] macro
#[derive(Debug, FromField)]
#[darling(attributes(graphql_object))]
struct GraphqlObjectFieldArgs {
    /// Indicates that this field is another graphql object and should have its type transformed to
    /// match while implementing `InputObject` etc.
    #[darling(default)]
    nested: bool,
}

/// Take a result and return token stream errors if it is an error
macro_rules! handle_darling_errors {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => {
                return TokenStream::from(e.write_errors()).into();
            }
        }
    };
}

/// An attribute macro that will derive both a [`SimpleObject`] and an [`InputObject`] for a struct
#[proc_macro_attribute]
pub fn graphql_object(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse attributes
    let attr_args = parse_macro_input!(args as AttributeArgs);
    // Get macro options from parsed attributes
    let options = handle_darling_errors!(GraphqlObjectMetaArgs::from_list(&attr_args));

    // Parse the reference struct
    let reference_struct = parse_macro_input!(input as ItemStruct);

    // Create output buffer
    let mut out = quote! {};

    // Generate the `SimpleObject` version of the struct
    let o = generate_output_object(&reference_struct, &options);
    out = quote! {
        #out
        #o
    };

    // Generate the `InputObject` version of the struct
    let o = generate_input_object(&reference_struct, &options);
    out = quote! {
        #out
        #o
    };

    out.into()
}

/// Generate the `SimpleObject` version of a struct
fn generate_output_object(
    reference_struct: &ItemStruct,
    options: &GraphqlObjectMetaArgs,
) -> TokenStream2 {
    // Start with a copy of the reference struct
    let mut object_struct = reference_struct.clone();

    // Remove any `io_object` meta tags from the fields
    for field in &mut object_struct.fields {
        utils::strip_annotations_with_path(format_ident!("graphql_object"), &mut field.attrs);
    }

    let extra_derive = if !options.skip_derive_simple_object {
        quote! {
            #[derive(::async_graphql::SimpleObject)]
        }
    } else {
        quote! {}
    };

    // output the struct unchanged, but with the extra simple object derive
    quote! {
        #extra_derive
        #object_struct
    }
}

/// Generate the `InputObject` of a generated struct
fn generate_input_object(
    reference_struct: &ItemStruct,
    options: &GraphqlObjectMetaArgs,
) -> TokenStream2 {
    // Initialize output
    let mut out = TokenStream2::new();

    // ouput a copy of the struct for the input type
    let mut input_obj_struct = reference_struct.clone();
    input_obj_struct.ident = format_ident!("{}{}", input_obj_struct.ident, "Input");

    // Update the input struct doc string if necessary
    if let Some(input_doc) = &options.input_object_doc {
        if let Some(doc) = input_obj_struct
            .attrs
            .iter_mut()
            .filter(|x| x.path.get_ident() == Some(&format_ident!("doc")))
            .next()
        {
            let input_doc = input_doc;

            doc.tokens = quote! { = #input_doc};
        }
    }

    // Loop through the fields and update them as necessary for the input type
    for field in &mut input_obj_struct.fields {
        let args = handle_darling_errors!(GraphqlObjectFieldArgs::from_field(&field));

        // If this is a nested object that should be transformed to it's input object equivalent
        if args.nested {
            // Transform the type of the arg from `MyType` to `MyTypeInput`
            if let Type::Path(path) = &mut field.ty {
                let ident = &mut path.path.segments.last_mut().unwrap().ident;
                *ident = format_ident!("{}{}", ident, "Input");
            } else {
                panic!("Cannot add #[object] annotation to type: {:?}", field.ty);
            }
        }

        // Remove any `io_object` annotation left over on the field
        let mut new_attrs = Vec::new();
        for attr in &field.attrs {
            if attr.path.get_ident() != Some(&format_ident!("graphql_object")) {
                new_attrs.push(attr.clone());
            }
        }

        field.attrs = new_attrs;
    }

    // Output input object struct
    out = quote! {
        #out

        #[derive(::async_graphql::InputObject)]
        #input_obj_struct
    };

    // Implement `Into<OriginalStruct> for OriginalStructInput`
    let orig_ident = &reference_struct.ident;
    let input_obj_ident = input_obj_struct.ident;

    let mut field_assignments = Vec::new();

    for field in &reference_struct.fields {
        let name = field.ident.as_ref().expect("Can't work with tuple structs");

        field_assignments.push(quote! {
            #name: self.#name.into()
        });
    }

    out = quote! {
        #out

        impl Into<#orig_ident> for #input_obj_ident {
            fn into(self) -> #orig_ident {
                #orig_ident {
                    #(#field_assignments),*
                }
            }
        }
    };

    out.into()
}

mod utils {
    use syn::{Attribute, Ident};

    pub fn strip_annotations_with_path(path: Ident, attrs: &mut Vec<Attribute>) {
        let mut new_attrs = Vec::new();
        for attr in attrs.iter() {
            if attr.path.get_ident() != Some(&path) {
                new_attrs.push(attr.clone());
            }
        }

        *attrs = new_attrs;
    }
}
