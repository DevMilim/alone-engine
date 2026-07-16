use darling::{Error, FromDeriveInput, FromField, ast::Data};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, Meta, Path, Token, Type, parse::Parse, punctuated::Punctuated};

#[derive(Debug)]
struct GameField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    base: bool,
    component: bool,
    object: bool,
}

impl FromField for GameField {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        let mut base = false;
        let mut component = false;
        let mut object = false;

        for attr in &field.attrs {
            if attr.path().is_ident("base") {
                base = true;
            } else if attr.path().is_ident("component") {
                component = true;
            } else if attr.path().is_ident("object") {
                object = true
            }
        }
        Ok(GameField {
            ident: field.ident.clone(),
            ty: field.ty.clone(),
            base,
            component,
            object,
        })
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(game), supports(struct_named))]
struct GameReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<darling::util::Ignored, GameField>,
    #[darling(default, with = "parse_subscriptions")]
    subscribe: Vec<Subscription>,
    #[darling(default, with = "parse_subscriptions")]
    connect: Vec<Subscription>,
}

#[derive(Debug)]
struct Subscription {
    handler: Ident,
    event_type: Path,
}

impl Parse for Subscription {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let handler: Ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        let event_type: Path = input.parse()?;
        Ok(Subscription {
            handler,
            event_type,
        })
    }
}

fn parse_subscriptions(meta: &Meta) -> Result<Vec<Subscription>, Error> {
    let list = match meta {
        Meta::List(list) => list,
        _ => return Err(Error::custom("Use o formato: subscribe(metodo: Tipo)")),
    };

    let parser = Punctuated::<Subscription, Token![,]>::parse_terminated;
    let subs = list
        .parse_args_with(parser)
        .map_err(|e| Error::custom(e.to_string()).with_span(meta))?;

    Ok(subs.into_iter().collect())
}

#[proc_macro_derive(GameObject, attributes(base, component, object, game))]
pub fn scene_tree(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let receiver = match GameReceiver::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    // 1. Otimização dos blocos de eventos (gerados condicionalmente para evitar overhead)
    let subscribe_block = (!receiver.subscribe.is_empty()).then(|| {
        let arms = receiver.subscribe.iter().map(|sub| {
            let event_ty = &sub.event_type;
            let handler_ident = &sub.handler;
            quote! {
                if let Some(payload) = any_event.downcast_ref::<#event_ty>() {
                    self.#handler_ident(ctx, payload);
                }
            }
        });
        quote! {
            if let ::alone_engine::prelude::GlobalEvent::Broadcast(any_event) = event {
                #(#arms)*
            }
        }
    });

    let connect_block = (!receiver.connect.is_empty()).then(|| {
        let arms = receiver.connect.iter().map(|sub| {
            let event_ty = &sub.event_type;
            let handler_ident = &sub.handler;
            quote! {
                if let Some(payload) = any_event.downcast_ref::<#event_ty>() {
                    self.#handler_ident(ctx, payload);
                }
            }
        });
        quote! {
            if let ::alone_engine::prelude::GlobalEvent::Targeted(id, any_event) = event {
                if &self.base().id == id {
                    #(#arms)*
                    return;
                }
            }
        }
    });

    let struct_name = &receiver.ident;
    let fields = receiver.data.take_struct().unwrap();

    let mut base_field = None;
    let mut component_fields = Vec::new();
    let mut object_fields = Vec::new();
    let mut bounds = Vec::new();

    for field in fields.fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        if field.base {
            if base_field.is_some() {
                return syn::Error::new_spanned(
                    ident,
                    "Apenas um campo pode ser marcado como base",
                )
                .to_compile_error()
                .into();
            }
            if type_is_base(ty) {
                base_field = Some(ident.clone());
            } else {
                return syn::Error::new_spanned(ty, "O campo base precisa ser do tipo Base")
                    .to_compile_error()
                    .into();
            }
        } else if field.component {
            component_fields.push(ident.clone());
            bounds.push(quote! { #ty: ::alone_engine::prelude::Component });
        } else if field.object {
            object_fields.push(ident.clone());
            bounds.push(
                quote! { #ty: ::alone_engine::prelude::GameObject + ::alone_engine::prelude::GameObjectDispatch },
            );
        }
    }

    let base_field = match base_field {
        Some(field) => field,
        None => {
            return syn::Error::new_spanned(struct_name, "Nenhum campo foi marcado com #[base]")
                .to_compile_error()
                .into();
        }
    };

    let apply_transform = quote! {
        let inherit = !self.#base_field.top_level;
        self.#base_field.transform.apply_parent(&parent_base.transform, inherit);
    };

    let (impl_generics, ty_generics, where_clause) = receiver.generics.split_for_impl();
    let where_tokens = if let Some(wc) = where_clause {
        quote! { #wc, Self: ::alone_engine::prelude::GameObject, #(#bounds),* }
    } else {
        quote! { where Self: ::alone_engine::prelude::GameObject, #(#bounds),* }
    };

    quote! {
        impl #impl_generics ::alone_engine::prelude::GameObjectBase for #struct_name #ty_generics {
            fn base(&self) -> &::alone_engine::prelude::Base {
                &self.#base_field
            }

            fn base_mut(&mut self) -> &mut ::alone_engine::prelude::Base {
                &mut self.#base_field
            }
        }

        impl #impl_generics ::alone_engine::prelude::GameObjectDispatch for #struct_name #ty_generics #where_tokens {
            fn is_pending_removal(&self) -> bool {
                self.base().pending_removal
            }

            fn dispatch_start(&mut self, ctx: &mut impl ::alone_engine::prelude::EngineApi, parent_base: &::alone_engine::prelude::Base) {
                if self.is_started() {
                    return;
                }
                ctx.register_alive(self.base().id);
                self.start(ctx);
                #(self.#component_fields.start(ctx, &mut self.#base_field);)*
                #(self.#object_fields.dispatch_start(ctx, &self.#base_field);)*
                self.mark_as_started();
            }

            fn dispatch_message(&mut self, ctx: &mut impl ::alone_engine::prelude::EngineApi) {
                if ctx.mail_box_is_empty() {
                    return;
                }

                let mailbox = ctx.mailbox();
                if let Some(msgs) = mailbox.remove(&self.base().id) {
                    for msg in msgs {
                        if let Some(message) = msg.downcast_ref::<<Self as ::alone_engine::prelude::GameObject>::Message>() {
                            self.on_message(ctx, message);
                        } else {
                            #[cfg(debug_assertions)]
                            println!("Tipo de evento incompatível recebido");
                        }
                    }
                }

                if ctx.mail_box_is_empty() {
                    return;
                }
                #(self.#object_fields.dispatch_message(ctx);)*
            }

            fn dispatch_event(&mut self, ctx: &mut impl ::alone_engine::prelude::EngineApi, event: &::alone_engine::prelude::GlobalEvent) {
                #subscribe_block
                #connect_block
                #(self.#object_fields.dispatch_event(ctx, event);)*
            }

            fn dispatch_update(&mut self, ctx: &mut impl ::alone_engine::prelude::EngineApi, parent_base: &::alone_engine::prelude::Base, delta: f32) {
                #apply_transform
                if !self.is_started() {
                    self.dispatch_start(ctx, parent_base);
                }
                self.update(ctx, delta);
                #(self.#component_fields.update(ctx, &mut self.#base_field, delta);)*
                #(self.#object_fields.dispatch_update(ctx, &self.#base_field, delta);)*
            }

            fn dispatch_late_update(&mut self, ctx: &mut impl ::alone_engine::prelude::EngineApi, parent_base: &::alone_engine::prelude::Base, delta: f32) {
                #apply_transform
                if !self.is_started() {
                    self.dispatch_start(ctx, parent_base);
                }
                self.late_update(ctx, delta);
                #(self.#component_fields.late_update(ctx, &mut self.#base_field, delta);)*
                #(self.#object_fields.dispatch_late_update(ctx, &self.#base_field, delta);)*
            }

            fn dispatch_fixed_update(&mut self, ctx: &mut impl ::alone_engine::prelude::EngineApi, parent_base: &::alone_engine::prelude::Base, delta: f32) {
                #apply_transform
                if !self.is_started() {
                    self.dispatch_start(ctx, parent_base);
                }
                self.fixed_update(ctx, delta);
                #(self.#component_fields.fixed_update(ctx, &mut self.#base_field, delta);)*
                #(self.#object_fields.dispatch_fixed_update(ctx, &self.#base_field, delta);)*
            }

            fn dispatch_draw(&mut self, renderer: &mut impl ::alone_engine::prelude::RenderApi, parent_base: &::alone_engine::prelude::Base, blending: f32) {
                #apply_transform
                self.draw(renderer, blending);
                #(self.#component_fields.draw(renderer, &self.#base_field, blending);)*
                #(self.#object_fields.dispatch_draw(renderer, &self.#base_field, blending);)*
            }

            fn dispatch_destroy(&mut self, ctx: &mut impl ::alone_engine::prelude::EngineApi) {
                ctx.unregister_alive(self.base().id);
                ctx.abort_tasks_of(self.base().id);
                self.destroy(ctx);
                #(self.#component_fields.destroy(ctx, &self.#base_field);)*
                #(self.#object_fields.dispatch_destroy(ctx);)*
            }
        }
    }
    .into()
}

fn type_is_base(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(seg) = type_path.path.segments.last()
    {
        return seg.ident == "Base";
    }

    false
}

fn derive_object_dispatch_enum(
    input: &syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, TokenStream> {
    let name = &input.ident;

    let data = match &input.data {
        syn::Data::Enum(d) => d,
        _ => {
            return Err(
                syn::Error::new_spanned(name, "Só pode ser utilizado em enums")
                    .into_compile_error()
                    .into(),
            );
        }
    };

    let mut variant_idents = Vec::new();
    let mut variant_types = Vec::new();

    for variant in &data.variants {
        variant_idents.push(&variant.ident);
        match &variant.fields {
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                variant_types.push(&fields.unnamed.first().unwrap().ty);
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    variant,
                    "Cada variante deve possuir exatamente um campo não nomeado. Ex: Potion(Potion)"
                ).into_compile_error().into());
            }
        }
    }
    let mut seen_types = std::collections::HashSet::new();
    let mut from_impls = Vec::new();

    for (ident, ty) in variant_idents.iter().zip(variant_types.iter()) {
        let ty_string = quote! {#ty}.to_string();
        if seen_types.insert(ty_string) {
            from_impls.push(quote! {
                impl From<#ty> for #name {
                    fn from(obj: #ty) -> Self {
                        Self::#ident(obj)
                    }
                }
            });
        }
    }

    Ok(quote! {
        impl ::alone_engine::prelude::GameObjectDispatch for #name {
            fn is_pending_removal(&self) -> bool {
                match self {
                    #(Self::#variant_idents(inner) => inner.is_pending_removal(),)*
                }
            }

            fn dispatch_start(&mut self, ctx: &mut impl ::alone_engine::prelude::EngineApi, parent_base: &::alone_engine::prelude::Base) {
                match self {
                    #(Self::#variant_idents(inner) => inner.dispatch_start(ctx, parent_base),)*
                }
            }

            fn dispatch_message(&mut self, ctx: &mut impl ::alone_engine::prelude::EngineApi) {
                match self {
                    #(Self::#variant_idents(inner) => inner.dispatch_message(ctx),)*
                }
            }

            fn dispatch_event(&mut self, ctx: &mut impl ::alone_engine::prelude::EngineApi, event: &::alone_engine::prelude::GlobalEvent) {
                match self {
                    #(Self::#variant_idents(inner) => inner.dispatch_event(ctx, event),)*
                }
            }

            fn dispatch_update(&mut self, ctx: &mut impl ::alone_engine::prelude::EngineApi, parent_base: &::alone_engine::prelude::Base, delta: f32) {
                match self {
                    #(Self::#variant_idents(inner) => inner.dispatch_update(ctx, parent_base, delta),)*
                }
            }

            fn dispatch_late_update(&mut self, ctx: &mut impl ::alone_engine::prelude::EngineApi, parent_base: &::alone_engine::prelude::Base, delta: f32) {
                match self {
                    #(Self::#variant_idents(inner) => inner.dispatch_late_update(ctx, parent_base, delta),)*
                }
            }

            fn dispatch_fixed_update(&mut self, ctx: &mut impl ::alone_engine::prelude::EngineApi, parent_base: &::alone_engine::prelude::Base, delta: f32) {
                match self {
                    #(Self::#variant_idents(inner) => inner.dispatch_fixed_update(ctx, parent_base, delta),)*
                }
            }

            fn dispatch_draw(&mut self, renderer: &mut impl ::alone_engine::prelude::RenderApi, parent_base: &::alone_engine::prelude::Base, blending: f32) {
                match self {
                    #(Self::#variant_idents(inner) => inner.dispatch_draw(renderer, parent_base, blending),)*
                }
            }

            fn dispatch_destroy(&mut self, ctx: &mut impl ::alone_engine::prelude::EngineApi) {
                match self {
                    #(Self::#variant_idents(inner) => inner.dispatch_destroy(ctx),)*
                }
            }
        }
        impl ::alone_engine::prelude::GameObjectBase for #name {
            fn base(&self) -> &::alone_engine::prelude::Base {
                match self {
                    #(Self::#variant_idents(inner) => inner.base(),)*
                }
            }
            fn base_mut(&mut self) -> &mut ::alone_engine::prelude::Base {
                match self {
                    #(Self::#variant_idents(inner) => inner.base_mut(),)*
                }
            }
        }

        #(#from_impls)*
    })
}

#[proc_macro_derive(ObjectEnum)]
pub fn object_enum_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match derive_object_dispatch_enum(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err,
    }
}

#[proc_macro_derive(Scene)]
pub fn scene_dispatch_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;

    let dispatch_impl = match derive_object_dispatch_enum(&input) {
        Ok(tokens) => tokens,
        Err(err) => return err,
    };

    let scene_impl = quote! {
        impl ::alone_engine::prelude::Scene for #name {
            fn get_dispatch(&mut self) -> &mut impl ::alone_engine::prelude::GameObjectDispatch {
                self
            }
        }
    };

    quote! {
        #dispatch_impl
        #scene_impl
    }
    .into()
}
