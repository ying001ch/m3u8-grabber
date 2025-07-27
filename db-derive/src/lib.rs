// db_derive/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(DbInsertable)]
pub fn db_insertable_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("DbInsertable only supports structs with named fields"),
        },
        _ => panic!("DbInsertable can only be derived for structs"),
    };

    let field_names: Vec<_> = fields
        .iter()
        .filter(|field| field.ident.as_ref().unwrap() != "id")
        .map(|field| field.ident.as_ref().unwrap())
        .collect();

    let field_count = field_names.len();
    let placeholders = vec!["?"; field_count].join(", ");

    let expanded = quote! {
        impl DbInsertable for #name {
            fn insert_statement() -> String {
                format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    stringify!(#name),
                    vec![#(stringify!(#field_names)),*].join(", "),
                    #placeholders
                )
            }

            fn as_arguments(&self) -> ::sqlx_sqlite::SqliteArguments<'_> {
                use sqlx::Arguments;

                let mut args = ::sqlx_sqlite::SqliteArguments::default();
                #(
                    args.add(&self.#field_names);
                )*
                args
            }
        }
    };

    TokenStream::from(expanded)
}