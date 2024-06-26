use std::borrow::Cow;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, Attribute, Data, Fields, FieldsNamed, FieldsUnnamed, Generics};

mod enum_schemas;
mod enum_variant;
mod feature;
mod flattened_map_schema;
mod struct_schemas;
mod xml;

pub(crate) use self::{
    enum_schemas::*,
    feature::{FromAttributes, NamedFieldStructFeatures, UnnamedFieldStructFeatures},
    flattened_map_schema::*,
    struct_schemas::*,
    xml::XmlAttr,
};

use super::{
    feature::{pop_feature_as_inner, Feature, FeaturesExt, IntoInner},
    ComponentSchema, FieldRename, VariantRename,
};
use crate::feature::{Bound, Inline, SkipBound, Symbol};
use crate::serde_util::SerdeValue;
use crate::{bound, DiagLevel, DiagResult, Diagnostic, TryToTokens};

pub(crate) struct ToSchema<'a> {
    ident: &'a Ident,
    attributes: &'a [Attribute],
    generics: &'a Generics,
    data: &'a Data,
    // vis: &'a Visibility,
}

impl<'a> ToSchema<'a> {
    pub(crate) fn new(
        data: &'a Data,
        attributes: &'a [Attribute],
        ident: &'a Ident,
        generics: &'a Generics,
        // vis: &'a Visibility,
    ) -> Self {
        Self {
            data,
            ident,
            attributes,
            generics,
            // vis,
        }
    }
}

impl TryToTokens for ToSchema<'_> {
    fn try_to_tokens(&self, tokens: &mut TokenStream) -> DiagResult<()> {
        let oapi = crate::oapi_crate();
        let ident = self.ident;
        let mut variant = SchemaVariant::new(self.data, self.attributes, ident, self.generics)?;

        let (_, ty_generics, _) = self.generics.split_for_impl();

        let inline = variant.inline().as_ref().map(|i| i.0).unwrap_or(false);
        let symbol = if inline {
            None
        } else if let Some(symbol) = variant.symbol() {
            if self.generics.type_params().next().is_none() {
                Some(quote! { #symbol.to_string().replace(" :: ", ".") })
            } else {
                Some(quote! {
                   {
                       let full_name = std::any::type_name::<#ident #ty_generics>();
                       if let Some((_, args)) = full_name.split_once('<') {
                           format!("{}<{}", #symbol, args)
                       } else {
                           full_name.into()
                       }
                   }
                })
            }
        } else {
            Some(quote! { std::any::type_name::<#ident #ty_generics>().replace("::", ".") })
        };

        let skip_bound = variant.pop_skip_bound();
        let bound = if skip_bound == Some(SkipBound(true)) {
            None
        } else {
            variant.pop_bound().map(|b| b.0)
        };

        let mut generics = bound::without_defaults(self.generics);
        if skip_bound != Some(SkipBound(true)) {
            generics = match bound {
                Some(predicates) => bound::with_where_predicates(&generics, &predicates),
                None => bound::with_bound(self.data, &generics, parse_quote!(#oapi::oapi::ToSchema + 'static)),
            };
        }

        let (impl_generics, _, where_clause) = generics.split_for_impl();

        let variant = variant.try_to_token_stream()?;
        let body = match symbol {
            None => {
                quote! {
                    #variant.into()
                }
            }
            Some(symbol) => {
                quote! {
                    let schema = #variant;
                    components.schemas.insert(#symbol, schema.into());
                    #oapi::oapi::RefOr::Ref(#oapi::oapi::Ref::new(format!("#/components/schemas/{}", #symbol)))
                }
            }
        };
        tokens.extend(quote!{
            impl #impl_generics #oapi::oapi::ToSchema for #ident #ty_generics #where_clause {
                fn to_schema(components: &mut #oapi::oapi::Components) -> #oapi::oapi::RefOr<#oapi::oapi::schema::Schema> {
                    #body
                }
            }
        });
        Ok(())
    }
}

#[derive(Debug)]
enum SchemaVariant<'a> {
    Named(NamedStructSchema<'a>),
    Unnamed(UnnamedStructSchema<'a>),
    Enum(EnumSchema<'a>),
    Unit(UnitStructVariant),
}

impl<'a> SchemaVariant<'a> {
    pub(crate) fn new(
        data: &'a Data,
        attributes: &'a [Attribute],
        ident: &'a Ident,
        generics: &'a Generics,
    ) -> DiagResult<SchemaVariant<'a>> {
        match data {
            Data::Struct(content) => match &content.fields {
                Fields::Unnamed(fields) => {
                    let FieldsUnnamed { unnamed, .. } = fields;
                    let mut unnamed_features = attributes.parse_features::<UnnamedFieldStructFeatures>()?.into_inner();

                    let symbol = pop_feature_as_inner!(unnamed_features => Feature::Symbol(_v));
                    let inline = pop_feature_as_inner!(unnamed_features => Feature::Inline(_v));
                    Ok(Self::Unnamed(UnnamedStructSchema {
                        struct_name: Cow::Owned(ident.to_string()),
                        attributes,
                        features: unnamed_features,
                        fields: unnamed,
                        symbol,
                        inline,
                    }))
                }
                Fields::Named(fields) => {
                    let FieldsNamed { named, .. } = fields;
                    let mut named_features = attributes.parse_features::<NamedFieldStructFeatures>()?.into_inner();
                    let symbol = pop_feature_as_inner!(named_features => Feature::Symbol(_v));
                    let inline = pop_feature_as_inner!(named_features => Feature::Inline(_v));

                    Ok(Self::Named(NamedStructSchema {
                        struct_name: Cow::Owned(ident.to_string()),
                        attributes,
                        rename_all: named_features.pop_rename_all_feature(),
                        features: named_features,
                        fields: named,
                        generics: Some(generics),
                        symbol,
                        inline,
                    }))
                }
                Fields::Unit => Ok(Self::Unit(UnitStructVariant)),
            },
            Data::Enum(content) => Ok(Self::Enum(EnumSchema::new(
                Cow::Owned(ident.to_string()),
                &content.variants,
                attributes,
            )?)),
            _ => Err(Diagnostic::spanned(
                ident.span(),
                DiagLevel::Error,
                "unexpected data type, expected syn::Data::Struct or syn::Data::Enum",
            )),
        }
    }

    fn symbol(&self) -> &Option<Symbol> {
        match self {
            Self::Enum(schema) => &schema.symbol,
            Self::Named(schema) => &schema.symbol,
            Self::Unnamed(schema) => &schema.symbol,
            _ => &None,
        }
    }
    fn inline(&self) -> &Option<Inline> {
        match self {
            Self::Enum(schema) => &schema.inline,
            Self::Named(schema) => &schema.inline,
            Self::Unnamed(schema) => &schema.inline,
            _ => &None,
        }
    }
    fn pop_skip_bound(&mut self) -> Option<SkipBound> {
        match self {
            Self::Enum(schema) => schema.pop_skip_bound(),
            Self::Named(schema) => schema.pop_skip_bound(),
            Self::Unnamed(schema) => schema.pop_skip_bound(),
            _ => None,
        }
    }
    fn pop_bound(&mut self) -> Option<Bound> {
        match self {
            Self::Enum(schema) => schema.pop_bound(),
            Self::Named(schema) => schema.pop_bound(),
            Self::Unnamed(schema) => schema.pop_bound(),
            _ => None,
        }
    }
}

impl TryToTokens for SchemaVariant<'_> {
    fn try_to_tokens(&self, tokens: &mut TokenStream) -> DiagResult<()> {
        match self {
            Self::Enum(schema) => schema.try_to_tokens(tokens),
            Self::Named(schema) => schema.try_to_tokens(tokens),
            Self::Unnamed(schema) => schema.try_to_tokens(tokens),
            Self::Unit(unit) => {
                unit.to_tokens(tokens);
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
struct UnitStructVariant;

impl ToTokens for UnitStructVariant {
    fn to_tokens(&self, stream: &mut proc_macro2::TokenStream) {
        let oapi = crate::oapi_crate();
        stream.extend(quote! {
            #oapi::oapi::schema::empty()
        });
    }
}

#[derive(Debug)]
enum Property {
    Schema(ComponentSchema),
    SchemaWith(Feature),
    FlattenedMap(FlattenedMapSchema),
}

impl TryToTokens for Property {
    fn try_to_tokens(&self, tokens: &mut TokenStream) -> DiagResult<()> {
        match self {
            Self::Schema(schema) => {
                schema.to_tokens(tokens);
                Ok(())
            }
            Self::FlattenedMap(schema) => {
                schema.to_tokens(tokens);
                Ok(())
            }
            Self::SchemaWith(with_schema) => with_schema.try_to_tokens(tokens),
        }
    }
}

trait SchemaFeatureExt {
    fn split_for_symbol(self) -> (Vec<Feature>, Vec<Feature>);
}

impl SchemaFeatureExt for Vec<Feature> {
    fn split_for_symbol(self) -> (Vec<Feature>, Vec<Feature>) {
        self.into_iter()
            .partition(|feature| matches!(feature, Feature::Symbol(_)))
    }
}

#[inline]
fn is_not_skipped(rule: Option<&SerdeValue>) -> bool {
    rule.map(|value| !value.skip).unwrap_or(true)
}

#[inline]
fn is_flatten(rule: Option<&SerdeValue>) -> bool {
    rule.map(|value| value.flatten).unwrap_or(false)
}
