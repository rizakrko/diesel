use meta::*;
use proc_macro2::{self, Ident, Span};
use syn;
use util::*;

pub fn derive(mut item: syn::DeriveInput) -> Result<proc_macro2::TokenStream, Diagnostic> {
    let flags =
        MetaItem::with_name(&item.attrs, "diesel").unwrap_or_else(|| MetaItem::empty("diesel"));
    let struct_ty = ty_for_foreign_derive(&item, &flags)?;

    item.generics.params.push(parse_quote!(__ST));
    item.generics.params.push(parse_quote!(__DB));
    {
        let where_clause = item.generics
            .where_clause
            .get_or_insert(parse_quote!(where));
        where_clause
            .predicates
            .push(parse_quote!(__DB: diesel::backend::Backend));
        where_clause
            .predicates
            .push(parse_quote!(Self: FromSql<__ST, __DB>));
    }
    let (impl_generics, _, where_clause) = item.generics.split_for_impl();

    let dummy_mod = Ident::new(
        &format!(
            "_impl_from_sql_row_for_{}",
            item.ident.to_string().to_lowercase()
        ),
        Span::call_site(),
    );
    Ok(wrap_in_dummy_mod(
        dummy_mod,
        quote! {
            use self::diesel::deserialize::{self, FromSql, FromSqlRow, Queryable};

            impl #impl_generics FromSqlRow<__ST, __DB> for #struct_ty
            #where_clause
            {
                fn build_from_row<R: diesel::row::Row<__DB>>(row: &mut R)
                    -> deserialize::Result<Self>
                {
                    FromSql::<__ST, __DB>::from_sql(row.take())
                }
            }

            impl #impl_generics Queryable<__ST, __DB> for #struct_ty
            #where_clause
            {
                type Row = Self;

                fn build(row: Self::Row) -> Self {
                    row
                }
            }
        },
    ))
}
