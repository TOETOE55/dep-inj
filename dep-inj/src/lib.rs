use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::default::Default;
use syn::{parse_macro_input, parse_quote};

///
///
/// ```
/// #[derive(DepInj)]
/// #[target(Foo)]
/// struct FooState<T> {}
/// ```
///
/// will expand to
///
/// ```
///
///
/// #[repr(transparent)]
/// struct Foo<T, Deps: ?Sized> {
///     _marker: core::marker::PhantomData<FooState<T>>,
///     deps: Deps
/// }
///
/// // 为了禁止别人手动实现`Drop`
/// impl<T, Deps: ?Sized> Drop for Foo<T, Deps> {
///     #[inline]
///     fn drop(&mut self) { }
/// }
///
/// impl<T, Deps: AsRef<FooState<T>> + ?Sized> std::ops::Deref for Foo<T, Deps> {
///     type Target = FooState<T>;
///     #[inline]
///     fn deref(&self) -> &Self::Target {
///         self.deps.as_ref()
///     }
/// }
///
/// impl<T, Deps: AsMut<FooState<T>> + AsRef<FooState<T>> + ?Sized> std::ops::DerefMut for Foo<T, Deps> {
///     #[inline]
///     fn deref_mut(&mut self) -> &mut Self::Target {
///         self.deps.as_mut()
///     }
/// }
///
/// impl<T, Deps: Into<FooState<T>>> From<Foo<T, Deps>> for FooState<T> {
///     #[inline]
///     fn from(value: Foo<T, Deps>) -> Self {
///         value.into_inner().into()
///     }
/// }
///
/// impl<T, Deps: ?Sized> Foo<T, Deps> {
///     #[inline]
///     pub fn inj_ref(deps: &Deps) -> &Self {
///         unsafe { &*(deps as *const Deps as *const Self) }
///     }
///     #[inline]
///     pub fn deps_ref(&self) -> &Deps {
///         unsafe { &*(self as *const Self as *const Deps) }
///     }
///     #[inline]
///     pub fn inj_ref_mut(deps: &mut Deps) -> &mut Self {
///         unsafe { &mut*(deps as *mut Deps as *mut Self) }
///     }
///     #[inline]
///     pub fn deps_ref_mut(&mut self) -> &mut Deps {
///         unsafe { &mut*(self as *mut Self as *mut Deps) }
///     }
///     #[inline]
///     pub fn inj_box(deps: Box<Deps>) -> Box<Self> {
///         unsafe { Box::from_raw(Box::into_raw(deps) as *mut Self) }
///     }
///     #[inline]
///     pub fn deps_box(self: Box<Self>) -> Box<Deps> {
///         unsafe { Box::from_raw(Box::into_raw(self) as *mut Deps) }
///     }
///     #[inline]
///     pub fn inj_rc(deps: std::rc::Rc<Deps>) -> std::rc::Rc<Self> {
///         unsafe { std::rc::Rc::from_raw(std::rc::Rc::into_raw(deps) as *const Self)}
///     }
///     #[inline]
///     pub fn deps_rc(self: std::rc::Rc<Self>) -> std::rc::Rc<Deps> {
///         unsafe { std::rc::Rc::from_raw(std::rc::Rc::into_raw(self) as *const Deps) }
///     }
///     #[inline]
///     pub fn inj_arc(deps: std::sync::Arc<Deps>) -> std::sync::Arc<Self> {
///         unsafe { std::sync::Arc::from_raw(std::sync::Arc::into_raw(deps) as *const Self)}
///     }
///     #[inline]
///     pub fn deps_arc(self: std::sync::Arc<Self>) -> std::sync::Arc<Deps> {
///         unsafe { std::sync::Arc::from_raw(std::sync::Arc::into_raw(self) as *const Deps) }
///     }
/// }
///
/// impl<T, Deps> Foo<T, Deps> {
///     #[inline]
///     pub fn new(deps: Deps) -> Self {
///         Self {
///             _marker: core::marker::PhantomData,
///             deps
///         }
///     }
///
///     #[inline]
///     pub fn into_inner(self) -> Deps {
///         self.deps
///     }
/// }
/// ```
#[proc_macro_derive(DepInj, attributes(target, inject))]
pub fn derive_dep_inj(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as syn::DeriveInput);

    match derive_dep_inj_impl(derive_input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn derive_dep_inj_impl(derive_input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let target_def = target_def(&derive_input)?;

    Ok(quote! {
        #target_def
    })
}

fn target_def(derive_input: &syn::DeriveInput) -> syn::Result<TokenStream> {
    // `FooState<T>`
    let derive_type = derive_type(derive_input);
    // `struct Foo<T, Deps: ?Sized> { .. }`
    let target_struct = target_struct(derive_input, &derive_type)?;
    // `Foo<T, Deps>`
    let target_type = target_type(&target_struct);
    // `impl<T, Deps: ?Sized> Drop for Foo<T, Deps>`
    let target_drop = target_drop(&target_struct, &target_type);
    // `impl Deref for Foo<T, Deps>`
    let target_deref = target_deref(&target_struct, &target_type, &derive_type);
    // `impl DerefMut for Foo<T, Deps>`
    let target_deref_mut = target_deref_mut(&target_struct, &target_type, &derive_type);
    // `impl From<Foo<T, Deps>> for FooState<T>`
    let target_from = target_from(&target_struct, &target_type, &derive_type);
    let target_ref_casting = target_impl_ref_casting(&target_struct, &target_type);
    let target_impl_new = target_impl_new(&target_struct, &target_type);

    Ok(quote! {
        #target_struct
        #target_drop
        #target_deref
        #target_deref_mut
        #target_from
        #target_ref_casting
        #target_impl_new
    })
}

fn target_struct(
    derive_input: &syn::DeriveInput,
    derive_type: &syn::Type,
) -> syn::Result<syn::ItemStruct> {
    let target_ident = target_ident(derive_input)?;

    let mut target_generics = derive_input.generics.clone();
    target_generics
        .params
        .push(parse_quote! { __Deps__: ?Sized });

    let target_fields = syn::Fields::Named(parse_quote! {{
        _marker: core::marker::PhantomData<#derive_type>,
        deps: __Deps__
    }});

    Ok(syn::ItemStruct {
        attrs: vec![parse_quote!(#[repr(transparent)])],
        vis: derive_input.vis.clone(),
        struct_token: Default::default(),
        ident: target_ident,
        generics: target_generics,
        fields: target_fields,
        semi_token: None,
    })
}

fn target_ident(derive_input: &syn::DeriveInput) -> syn::Result<syn::Ident> {
    let target = derive_input
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("target"))
        .collect::<Vec<_>>();

    match &*target {
        [target] => target.parse_args::<syn::Ident>(),
        _ => Err(syn::Error::new(
            Span::call_site(),
            "`DepInj` requires one and only one `#[target()]` attribute",
        )),
    }
}

fn target_type(target_struct: &syn::ItemStruct) -> syn::Type {
    let ident = target_struct.ident.clone();
    let mut generic = target_struct.generics.clone();
    generic.where_clause = None;
    for param in generic.params.iter_mut() {
        match param {
            syn::GenericParam::Type(ty) => {
                ty.attrs = vec![];
                ty.colon_token = None;
                ty.bounds = Default::default();
                ty.eq_token = None;
                ty.default = None;
            }
            syn::GenericParam::Lifetime(lifetime) => {
                lifetime.attrs = vec![];
                lifetime.colon_token = None;
                lifetime.bounds = Default::default();
            }
            syn::GenericParam::Const(r#const) => {
                r#const.attrs = vec![];
                r#const.const_token = Default::default();
                r#const.eq_token = None;
                r#const.default = None;
            }
        }
    }

    // Foo<T, Deps>
    parse_quote! {
        #ident #generic
    }
}

fn target_drop(target_struct: &syn::ItemStruct, target_type: &syn::Type) -> syn::ItemImpl {
    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics: target_struct.generics.clone(),
        trait_: Some((None, parse_quote!(Drop), Default::default())),
        self_ty: Box::new(target_type.clone()),
        brace_token: Default::default(),
        items: vec![parse_quote! {
            #[inline]
            fn drop(&mut self) {}
        }],
    }
}

fn target_deref(
    target_struct: &syn::ItemStruct,
    target_type: &syn::Type,
    derive_type: &syn::Type,
) -> syn::ItemImpl {
    let mut generics = target_struct.generics.clone();
    let where_clause = generics.where_clause.get_or_insert(syn::WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });
    where_clause
        .predicates
        .push(parse_quote!(__Deps__: AsRef<#derive_type>));

    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics,
        trait_: Some((None, parse_quote!(core::ops::Deref), Default::default())),
        self_ty: Box::new(target_type.clone()),
        brace_token: Default::default(),
        items: vec![
            parse_quote! {
                type Target = #derive_type;
            },
            parse_quote! {
                #[inline]
                fn deref(&self) -> &Self::Target {
                    self.deps.as_ref()
                }
            },
        ],
    }
}

fn target_deref_mut(
    target_struct: &syn::ItemStruct,
    target_type: &syn::Type,
    derive_type: &syn::Type,
) -> syn::ItemImpl {
    let mut generics = target_struct.generics.clone();
    let where_clause = generics.where_clause.get_or_insert(syn::WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });
    where_clause
        .predicates
        .push(parse_quote!(__Deps__: AsRef<#derive_type> + AsMut<#derive_type>));

    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics,
        trait_: Some((None, parse_quote!(core::ops::DerefMut), Default::default())),
        self_ty: Box::new(target_type.clone()),
        brace_token: Default::default(),
        items: vec![parse_quote! {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.deps.as_mut()
            }
        }],
    }
}

fn target_from(
    target_struct: &syn::ItemStruct,
    target_type: &syn::Type,
    derive_type: &syn::Type,
) -> syn::ItemImpl {
    let mut generics = target_struct.generics.clone();
    // __Deps__: ?Sized -> __Deps__
    let dep = generics.params.last_mut().unwrap();
    *dep = syn::GenericParam::Type(parse_quote!(__Deps__));

    let where_clause = generics.where_clause.get_or_insert(syn::WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });
    where_clause
        .predicates
        .push(parse_quote!(__Deps__: Into<#derive_type>));

    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics,
        trait_: Some((None, parse_quote!(From<#target_type>), Default::default())),
        self_ty: Box::new(derive_type.clone()),
        brace_token: Default::default(),
        items: vec![parse_quote! {
            fn from(value: #target_type) -> Self {
                value.into_inner().into()
            }
        }],
    }
}

fn target_impl_ref_casting(
    target_struct: &syn::ItemStruct,
    target_type: &syn::Type,
) -> syn::ItemImpl {
    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics: target_struct.generics.clone(),
        trait_: None,
        self_ty: Box::new(target_type.clone()),
        brace_token: Default::default(),
        items: vec![
            parse_quote! {
                #[inline]
                pub fn inj_ref(deps: &__Deps__) -> &Self {
                     unsafe { &*(deps as *const __Deps__ as *const Self) }
                }
            },
            parse_quote! {
                #[inline]
                pub fn deps_ref(&self) -> &__Deps__ {
                    unsafe { &*(self as *const Self as *const __Deps__) }
                }
            },
            parse_quote! {
                #[inline]
                pub fn inj_ref_mut(deps: &mut __Deps__) -> &mut Self {
                    unsafe { &mut*(deps as *mut __Deps__ as *mut Self) }
                }
            },
            parse_quote! {
                #[inline]
                pub fn deps_ref_mut(&mut self) -> &mut __Deps__ {
                    unsafe { &mut*(self as *mut Self as *mut __Deps__) }
                }
            },
            parse_quote! {
                #[inline]
                pub fn inj_box(deps: Box<__Deps__>) -> Box<Self> {
                    unsafe { Box::from_raw(Box::into_raw(deps) as *mut Self) }
                }
            },
            parse_quote! {
                #[inline]
                pub fn deps_box(self: Box<Self>) -> Box<__Deps__> {
                    unsafe { Box::from_raw(Box::into_raw(self) as *mut __Deps__) }
                }
            },
            parse_quote! {
                #[inline]
                pub fn inj_rc(deps: std::rc::Rc<__Deps__>) -> std::rc::Rc<Self> {
                    unsafe { std::rc::Rc::from_raw(std::rc::Rc::into_raw(deps) as *const Self)}
                }
            },
            parse_quote! {
                #[inline]
                pub fn deps_rc(self: std::rc::Rc<Self>) -> std::rc::Rc<__Deps__> {
                    unsafe { std::rc::Rc::from_raw(std::rc::Rc::into_raw(self) as *const __Deps__) }
                }
            },
            parse_quote! {
                #[inline]
                pub fn inj_arc(deps: std::sync::Arc<__Deps__>) -> std::sync::Arc<Self> {
                    unsafe { std::sync::Arc::from_raw(std::sync::Arc::into_raw(deps) as *const Self)}
                }
            },
            parse_quote! {
                #[inline]
                pub fn deps_arc(self: std::sync::Arc<Self>) -> std::sync::Arc<__Deps__> {
                    unsafe { std::sync::Arc::from_raw(std::sync::Arc::into_raw(self) as *const __Deps__) }
                }
            },
        ],
    }
}

fn target_impl_new(target_struct: &syn::ItemStruct, target_type: &syn::Type) -> syn::ItemImpl {
    let mut generics = target_struct.generics.clone();
    // __Deps__: ?Sized -> __Deps__
    let dep = generics.params.last_mut().unwrap();
    *dep = syn::GenericParam::Type(parse_quote!(__Deps__));

    syn::ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics,
        trait_: None,
        self_ty: Box::new(target_type.clone()),
        brace_token: Default::default(),
        items: vec![
            parse_quote! {
                #[inline]
                pub fn new(deps: __Deps__) -> Self {
                    Self {
                        _marker: core::marker::PhantomData,
                       deps
                    }
                }
            },
            parse_quote! {
                #[inline]
                pub fn into_inner(self) -> __Deps__ {
                    unsafe {
                        let deps = core::ptr::read(&self.deps);
                        core::mem::forget(self);
                        deps
                    }
                }
            },
        ],
    }
}

fn derive_type(derive_input: &syn::DeriveInput) -> syn::Type {
    let ident = derive_input.ident.clone();
    let mut generic = derive_input.generics.clone();
    generic.where_clause = None;
    for param in generic.params.iter_mut() {
        match param {
            syn::GenericParam::Type(ty) => {
                ty.attrs = vec![];
                ty.colon_token = None;
                ty.bounds = Default::default();
                ty.eq_token = None;
                ty.default = None;
            }
            syn::GenericParam::Lifetime(lifetime) => {
                lifetime.attrs = vec![];
                lifetime.colon_token = None;
                lifetime.bounds = Default::default();
            }
            syn::GenericParam::Const(r#const) => {
                r#const.attrs = vec![];
                r#const.const_token = Default::default();
                r#const.eq_token = None;
                r#const.default = None;
            }
        }
    }

    // FooState<T>
    parse_quote! {
        #ident #generic
    }
}
//
// struct InjectTrait {
//     trait_token: Token![trait],
//     ident: syn::Ident,
//     generics: syn::Generics,
// }
//
// struct InjectType {
//     pub type_token: Token![type],
//     pub ident: syn::Ident,
//     pub generics: syn::Generics,
// }
//
// enum Injection {
//     Trait(InjectTrait),
//     Type(InjectType),
// }
//
// impl Parse for InjectTrait {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         Ok(Self {
//             trait_token: input.parse()?,
//             ident: input.parse()?,
//             generics: input.parse()?,
//         })
//     }
// }

// impl Parse for InjectType {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         Ok(Self {
//             type_token: input.parse()?,
//             ident: input.parse()?,
//             generics: input.parse()?
//         })
//     }
// }
//
// impl Parse for Injection {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         let lookahead = input.fork().lookahead1();
//         if lookahead.peek(Token![trait]) {
//             Ok(Self::Trait(input.parse()?))
//         } else {
//             Ok(Self::Type(input.parse()?))
//         }
//     }
// }

// fn injections(derive_input: &syn::DeriveInput) -> syn::Result<Vec<Injection>> {
//     derive_input
//         .attrs
//         .iter()
//         .filter(|attr| attr.path.is_ident("inj"))
//         .map(|attr: &syn::Attribute| attr.parse_args::<Injection>())
//         .collect()
// }
