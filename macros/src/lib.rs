use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, ItemFn, ReturnType, Token,
};

struct DatabaseTypes {
    types: Punctuated<Ident, Token![,]>,
}

impl Parse for DatabaseTypes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(DatabaseTypes {
            types: Punctuated::parse_terminated(input)?,
        })
    }
}

#[proc_macro_attribute]
pub fn transactional(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let db_types = parse_macro_input!(attr as DatabaseTypes);

    let fn_name = &input.sig.ident;
    let fn_args = &input.sig.inputs;
    let fn_body = &input.block;
    let fn_vis = &input.vis;
    let fn_generics = &input.sig.generics;
    let (impl_generics, _type_generics, where_clause) = fn_generics.split_for_impl();

    let return_type = match &input.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };

    let db_setup = db_types.types.iter().enumerate().map(|(i, db_type)| {
        let db_var = format_ident!("db_{}", i);
        let txn_var = format_ident!("txn_{}", i);
        quote! {
            let #db_var = cx.get::<#db_type>().expect(&format!("the DB struct `{}` not found", stringify!(#db_type)));
            let #txn_var = #db_var.create_transaction().await.expect(&format!("Failed to create transaction for {}", stringify!(#db_type)));
        }
    });

    let cx_setup = db_types.types.iter().enumerate().map(|(i, _)| {
        let txn_var = format_ident!("txn_{}", i);
        quote! { .with_value(#txn_var) }
    });

    let db_commit = db_types.types.iter().map(|db_type| {
        quote! {
            cx = #db_type::commit_transaction_in_context(cx).await.expect(&format!("Failed to commit transaction for {}", stringify!(#db_type)));
        }
    });

    let db_rollback = db_types.types.iter().map(|db_type| {
        quote! {
            cx = #db_type::rollback_transaction_in_context(cx).await.expect(&format!("Failed to rollback transaction for {}", stringify!(#db_type)));
        }
    });

    let expanded = quote! {
        #fn_vis async fn #fn_name #impl_generics(#fn_args) #where_clause -> #return_type {
            let mut cx = Context::current();
            #(#db_setup)*
            cx = cx #(#cx_setup)*;

            let result = async move {
                #fn_body
            }.with_context(cx.clone()).await;

            match result {
                Ok(value) => {
                    // Commit the transactions
                    #(#db_commit)*
                    Ok(value)
                }
                Err(e) => {
                    // Rollback the transactions
                    #(#db_rollback)*
                    Err(e)
                }
            }
        }
    };

    TokenStream::from(expanded)
}
