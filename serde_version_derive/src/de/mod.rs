use crate::ast::attr::PathOrSelf;
use crate::ast::Container;
use proc_macro2::{Span, TokenStream};
use proc_macro_util::prelude::*;

pub fn expand_derive_deserialize_versioned(
    input: &syn::DeriveInput,
) -> Result<TokenStream, Vec<syn::Error>> {
    let ctxt = Ctxt::new();
    let cont = Container::from_ast(&ctxt, input);
    ctxt.check()?;

    match cont.attrs.versions() {
        Some(versions) => {
            let de_impl_generics = {
                let mut generics = cont.generics.clone();
                generics.params = Some(syn::GenericParam::Lifetime(syn::LifetimeDef::new(
                    syn::Lifetime::new("'de", Span::call_site()),
                )))
                .into_iter()
                .chain(generics.params)
                .collect();
                generics
            };
            let ident = &cont.ident;
            let ty_generics = cont.generics;

            let mut where_clause =
                syn::parse2::<syn::WhereClause>(quote! { where __VM: _serde_version::VersionMap })
                    .unwrap();
            if let Some(cont_where_clause) = cont.generics.where_clause.as_ref() {
                where_clause
                    .predicates
                    .extend(cont_where_clause.predicates.iter().cloned())
            }

            let deser_name = quote! { std::any::type_name::<Self>() };

            let last_version = *versions
                .iter()
                .find(|(_, v)| {
                    if let PathOrSelf::SelfType = &v.path {
                        true
                    } else {
                        false
                    }
                })
                // The self type is always described in the version attribute
                // This is enforced when building the Container
                .unwrap()
                .0;

            let deserialize_arms = versions.iter()
                .filter_map(|(version_number, version)| {
                    if version_number != &last_version {
                        let path = match &version.path {
                            PathOrSelf::Path(path) => path,
                            _ => unreachable!("Because version_number != &last_version"),
                        };
                        Some(quote! {
                            Some(#version_number) => std::result::Result::map(
                                <::core::marker::PhantomData<#path> as _serde_version::DeserializeVersionedSeed<'_>>::deserialize_versioned(::core::marker::PhantomData, __deserializer, __version_map),
                                std::convert::Into::into
                            ),
                        })
                    }
                    else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let next_element_arms = versions.iter()
                .filter_map(|(version_number, version)| {
                    if version_number != &last_version {
                        let path = match &version.path {
                            PathOrSelf::Path(path) => path,
                            _ => unreachable!("Because version_number != &last_version"),
                        };
                        Some(quote! {
                            Some(#version_number) => std::result::Result::map(
                                <::core::marker::PhantomData<#path> as _serde_version::DeserializeVersionedSeed<'_>>::next_element(::core::marker::PhantomData, __seq_access, __version_map),
                                |v| std::option::Option::map(v, std::convert::Into::into)
                            ),
                        })
                    }
                    else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let next_value_arms= versions.iter()
                .filter_map(|(version_number, version)| {
                    if version_number != &last_version {
                        let path = match &version.path {
                            PathOrSelf::Path(path) => path,
                            _ => unreachable!("Because version_number != &last_version"),
                        };
                        Some(quote! {
                            Some(#version_number) => std::result::Result::map(
                                <::core::marker::PhantomData<#path> as _serde_version::DeserializeVersionedSeed<'_>>::next_value(::core::marker::PhantomData, __map_access, __version_map),
                                std::convert::Into::into
                            ),
                        })
                    }
                    else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let next_key_arms = versions
                .iter()
                .map(|(version_number, version)| {
                    if version_number != &last_version {
                        let path = match &version.path {
                            PathOrSelf::Path(path) => path,
                            _ => unreachable!("Because version_number != &last_version"),
                        };
                        Some(quote! {
                            Some(#version_number) => std::result::Result::map(
                                <::core::marker::PhantomData<#path> as _serde_version::DeserializeVersionedSeed<'_>>::next_key(
                                    ::core::marker::PhantomData, 
                                    __map_access,
                                    __version_map
                                ),
                                |v| std::option::Option::map(v, std::convert::Into::into)
                            ),
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let variant_arms = versions
                .iter()
                .filter_map(|(version_number, version)| {
                    if version_number != &last_version {
                        let path = match &version.path {
                            PathOrSelf::Path(path) => path,
                            _ => unreachable!("Because version_number != &last_version"),
                        };
                        Some(quote! {
                            Some(#version_number) => std::result::Result::map(
                                <::core::marker::PhantomData<#path> as _serde_version::DeserializeVersionedSeed<'_>>::variant(
                                    ::core::marker::PhantomData,
                                    __enum_access,
                                    __version_map
                                ),
                                |(v, variant)| (std::convert::Into::into(v), variant)
                            ),
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            let code = quote! {
                impl #de_impl_generics _serde_version::DeserializeVersioned<'de> for #ident #ty_generics {
                    fn deserialize_versioned<__D, __VM: _serde_version::VersionMap>(
                        __deserializer: __D,
                        __version_map: __VM,
                    ) -> std::result::Result<Self, _serde_version::Error<__D::Error>>
                    where
                        __D: _serde::Deserializer<'de>, {
                        match __version_map.get(#deser_name) {
                            #(#deserialize_arms)*
                            None | Some(#last_version) => <Self as _serde::Deserialize<'de>>::deserialize(__deserializer)
                                .map_err(_serde_version::Error::DeserializeError),
                            Some(v) => Err(_serde_version::Error::InvalidVersionError(
                                _serde_version::InvalidVersionError {
                                    version: v,
                                    type_id: #deser_name.to_owned()
                                }
                            )),
                        }
                    }

                    #[inline]
                    fn next_element<__S, __VM: _serde_version::VersionMap>(
                        __seq_access: &mut __S,
                        __version_map: __VM,
                    ) -> std::result::Result<Option<Self>, _serde_version::Error<__S::Error>>
                    where
                        __S: _serde::de::SeqAccess<'de>
                    {
                        match __version_map.get(#deser_name) {
                            #(#next_element_arms)*
                            None | Some(#last_version) => <__S as _serde::de::SeqAccess<'de>>::next_element_seed(
                                __seq_access,
                                std::marker::PhantomData
                            ).map_err(_serde_version::Error::DeserializeError),
                            Some(v) => Err(_serde_version::Error::InvalidVersionError(
                                _serde_version::InvalidVersionError {
                                    version: v,
                                    type_id: #deser_name.to_owned()
                                }
                            ))
                        }
                    }

                    #[inline]
                    fn next_value<__M, __VM: _serde_version::VersionMap>(
                        __map_access: &mut __M,
                        __version_map: __VM,
                    ) -> std::result::Result<Self, _serde_version::Error<__M::Error>>
                    where
                        __M: _serde::de::MapAccess<'de>,
                    {
                        match __version_map.get(#deser_name) {
                            #(#next_value_arms)*
                            None | Some(#last_version) => <__M as _serde::de::MapAccess<'de>>::next_value_seed(
                                __map_access,
                                std::marker::PhantomData
                            ).map_err(_serde_version::Error::DeserializeError),
                            Some(v) => Err(_serde_version::Error::InvalidVersionError(
                                _serde_version::InvalidVersionError {
                                    version: v,
                                    type_id: #deser_name.to_owned()
                                }
                            )),
                        }
                    }

                    #[inline]
                    fn next_key<__M, __VM: _serde_version::VersionMap>(
                        __map_access: &mut __M,
                        __version_map: __VM,
                    ) -> std::result::Result<Option<Self>, _serde_version::Error<__M::Error>>
                    where
                        __M: _serde::de::MapAccess<'de>,
                    {
                        match __version_map.get(#deser_name) {
                            #(#next_key_arms)*
                            None | Some(#last_version) => <__M as _serde::de::MapAccess<'de>>::next_key_seed(
                                __map_access,
                                std::marker::PhantomData
                            ).map_err(_serde_version::Error::DeserializeError),
                            Some(v) => Err(_serde_version::Error::InvalidVersionError(
                                _serde_version::InvalidVersionError {
                                    version: v,
                                    type_id: #deser_name.to_owned()
                                }
                            )),
                        }
                    }

                    #[inline]
                    fn variant<__E, __VM: _serde_version::VersionMap>(
                        __enum_access: __E,
                        __version_map: __VM,
                    ) -> std::result::Result<(Self, __E::Variant), _serde_version::Error<__E::Error>>
                    where
                        __E: _serde::de::EnumAccess<'de>,
                    {
                        match __version_map.get(#deser_name) {
                            #(#variant_arms)*
                            None | Some(#last_version) => <__E as _serde::de::EnumAccess<'de>>::variant_seed(
                                __enum_access,
                                std::marker::PhantomData
                            ).map_err(_serde_version::Error::DeserializeError),
                            Some(v) => Err(_serde_version::Error::InvalidVersionError(
                                _serde_version::InvalidVersionError {
                                    version: v,
                                    type_id: #deser_name.to_owned()
                                }
                            )),
                        }
                    }

                    #[inline]
                    fn last_version() -> usize {
                        #last_version
                    }
                }
            };
            Ok(crate::util::wrap_in_const(
                None,
                None,
                "DESERIALIZE_VERSIONED",
                ident,
                code,
            ))
        }
        None => Ok(TokenStream::new()),
    }
}
