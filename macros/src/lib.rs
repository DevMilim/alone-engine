use darling::{FromDeriveInput, FromField, ast::Data};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, Path, Token, Type, parse::Parse, punctuated::Punctuated};

fn get_crate_name() -> proc_macro2::TokenStream {
    quote!(alone_engine)
}

#[derive(Debug)]
struct GameField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    base: bool,
    component: bool,
    interface: Option<syn::Path>,
    object: bool,
}

impl FromField for GameField {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        let mut base = false;
        let mut component = false;
        let mut component_trait = None;
        let mut object = false;
        let mut markers = 0u8;

        for attr in &field.attrs {
            if attr.path().is_ident("base") {
                base = true;
                markers += 1;
            } else if attr.path().is_ident("component") {
                component = true;
                markers += 1;

                if let syn::Meta::List(meta_list) = &attr.meta {
                    let args = meta_list
                        .parse_args::<ComponentArgs>()
                        .map_err(|e| darling::Error::custom(e.to_string()))?;
                    component_trait = args.interface;
                }
            } else if attr.path().is_ident("object") {
                object = true;
                markers += 1;
            }
        }
        if markers > 1 {
            return Err(darling::Error::custom(
                "um campo so pode ter um dos atributos: #[base], #[component] ou #[object]",
            )
            .with_span(field));
        }
        Ok(GameField {
            ident: field.ident.clone(),
            ty: field.ty.clone(),
            base,
            component,
            object,
            interface: component_trait,
        })
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(game), supports(struct_named))]
struct GameReceiver {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<darling::util::Ignored, GameField>,
}

#[derive(Debug)]
struct ComponentArgs {
    interface: Option<Path>,
}
impl Parse for ComponentArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut interface = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            let value: Path = input.parse()?;

            if ident == "interface" {
                interface = Some(value);
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("chave desconhecida `{ident}` em #[component(...)]; use `interface`"),
                ));
            }

            if input.is_empty() {
                break;
            }
            let _: Token![,] = input.parse()?;
        }

        Ok(Self { interface })
    }
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

fn parse_event_attributes(
    attrs: &[syn::Attribute],
) -> syn::Result<(Vec<Subscription>, Vec<Subscription>)> {
    let mut subscribe = Vec::new();
    let mut connect = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("subscribe") {
            let parsed =
                attr.parse_args_with(Punctuated::<Subscription, Token![,]>::parse_terminated)?;
            subscribe.extend(parsed);
        } else if attr.path().is_ident("connect") {
            let parsed =
                attr.parse_args_with(Punctuated::<Subscription, Token![,]>::parse_terminated)?;
            connect.extend(parsed);
        }
    }
    check_no_duplicates(&subscribe, "subscribe")?;
    check_no_duplicates(&connect, "connect")?;

    Ok((subscribe, connect))
}

fn check_no_duplicates(subs: &[Subscription], attr_name: &str) -> syn::Result<()> {
    let mut seen = std::collections::HashSet::new();
    for sub in subs {
        let ty = &sub.event_type;
        let key = (sub.handler.to_string(), quote!(#ty).to_string());
        if !seen.insert(key) {
            return Err(syn::Error::new_spanned(
                &sub.handler,
                format!(
                    "entrada duplicada em #[{attr_name}(...)] para `{}`",
                    sub.handler
                ),
            ));
        }
    }
    Ok(())
}

fn build_downcast_arms(subs: &[Subscription]) -> Vec<proc_macro2::TokenStream> {
    subs.iter()
        .map(|sub| {
            let event_ty = &sub.event_type;
            let handler_ident = &sub.handler;
            quote! {
                if let Some(payload) = any_event.downcast_ref::<#event_ty>() {
                    self.#handler_ident(ctx, payload);
                }
            }
        })
        .collect()
}

#[proc_macro_derive(GameObject, attributes(base, component, object, connect, subscribe))]
pub fn scene_tree(input: TokenStream) -> TokenStream {
    let crate_name = get_crate_name();
    let p = quote!(::#crate_name::prelude);
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let (subscribe, connect) = match parse_event_attributes(&input.attrs) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error().into(),
    };

    let receiver = match GameReceiver::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let subscribe_arms = build_downcast_arms(&subscribe);
    let connect_arms = build_downcast_arms(&connect);

    let mut seen_subscribe_types = std::collections::HashSet::new();
    let subscribe_type_ids: Vec<_> = subscribe
        .iter()
        .filter_map(|sub| {
            let ty = &sub.event_type;
            let ty_string = quote!(#ty).to_string();
            seen_subscribe_types
                .insert(ty_string)
                .then(|| quote! { ::std::any::TypeId::of::<#ty>() })
        })
        .collect();

    let broadcast_read_block = (!subscribe_type_ids.is_empty())
        .then(|| {
            quote! {
                for &type_id in &[#(#subscribe_type_ids),*] {
                    let buf = ctx.poll_broadcasts(self.base().id, type_id);
                    for any_event in &buf {
                        #(#subscribe_arms)*
                    }
                    ctx.recycle_broadcast_buffer(buf);
                }
            }
        })
        .unwrap_or_default();

    let event_dispatch_block = {
        quote! {
            if let Some(events) = ctx.take_mailbox(self.base().id){
                for event in events{
                    match event{
                        #p::GlobalEvent::Targeted(_id, any_event) => {
                            let _ = &any_event;
                            #(#connect_arms)*
                        }
                        #p::GlobalEvent::Broadcast(_) => {}
                        #p::GlobalEvent::Send(_id, any_event) =>{
                            if let Some(message) = any_event.downcast_ref::<<Self as #p::GameObject>::Message>() {
                                self.on_message(ctx, message);
                            }else{
                                println!("Evento incompativel recebido. Esperado: {}", std::any::type_name::<<Self as #p::GameObject>::Message>());
                            }
                        }
                    }
                }
            }
        }
    };

    let register_subscriptions = (!subscribe_type_ids.is_empty())
        .then(|| {
            quote! {
                ctx.register_subscriptions(self.base().id, &[#(#subscribe_type_ids),*]);
            }
        })
        .unwrap_or_default();

    let unregister_subscriptions = (!subscribe_type_ids.is_empty())
        .then(|| {
            quote! {
                ctx.unregister_subscriptions(self.base().id, &[#(#subscribe_type_ids),*]);
            }
        })
        .unwrap_or_default();

    let struct_name = &receiver.ident;
    let fields = receiver.data.take_struct().unwrap();

    let mut base_field = None;
    let mut component_fields = Vec::new();
    let mut object_fields = Vec::new();
    let mut bounds = Vec::new();
    let mut pending_component_impls = Vec::new();
    let mut seen_interfaces = std::collections::HashSet::new();
    let mut seen_component_bounds = std::collections::HashSet::new();
    let mut seen_object_bounds = std::collections::HashSet::new();
    for field in fields.fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        if field.base {
            if base_field.is_some() {
                return syn::Error::new_spanned(
                    ident,
                    "Apenas um campo pode ser marcado como base",
                )
                .into_compile_error()
                .into();
            }
            if type_is_base(ty) {
                base_field = Some(ident.clone());
            } else {
                return syn::Error::new_spanned(ty, "O campo base precisa ser do tipo Base")
                    .into_compile_error()
                    .into();
            }
        } else if field.component {
            component_fields.push(ident.clone());
            if seen_component_bounds.insert(quote!(#ty).to_string()) {
                bounds.push(quote! { #ty: #p::Component });
            }

            if let Some(trait_path) = &field.interface {
                let trait_name = quote! {#trait_path}.to_string();

                if !seen_interfaces.insert(trait_name) {
                    return syn::Error::new_spanned(
                        ident,
                        "A mesma interface de componente só pode ser utilizada uma vez",
                    )
                    .into_compile_error()
                    .into();
                }

                pending_component_impls.push((ident.clone(), ty.clone(), trait_path.clone()));
            }
        } else if field.object {
            object_fields.push(ident.clone());
            if seen_object_bounds.insert(quote!(#ty).to_string()) {
                bounds.push(quote! { #ty: #p::GameObject + #p::GameObjectDispatch });
            }
        }
    }

    let base_field = match base_field {
        Some(field) => field,
        None => {
            return syn::Error::new_spanned(struct_name, "Nenhum campo foi marcado com #[base]")
                .into_compile_error()
                .into();
        }
    };

    let apply_transform = quote! {
        let inherit = !self.#base_field.top_level;
        self.#base_field.transform.apply_parent(&parent_base.transform, inherit);
    };
    let ensure_started = quote! {
        if !self.is_started() {
            self.dispatch_start(ctx, parent_base);
        }
        #apply_transform
    };

    let (impl_generics, ty_generics, where_clause) = receiver.generics.split_for_impl();
    let where_tokens = if let Some(wc) = where_clause {
        quote! { #wc, Self: #p::GameObject, #(#bounds),* }
    } else {
        quote! { where Self: #p::GameObject, #(#bounds),* }
    };

    let injected_methods = pending_component_impls
        .iter()
        .map(|(ident, ty, trait_path)| {
            quote! {
                impl #p::IComponent<#ty> for #struct_name {
                    fn get_self(&self) -> & #ty {
                        &self.#ident
                    }
                    fn get_self_mut(&mut self) -> &mut #ty {
                        &mut self.#ident
                    }
                    fn get_self_and_base_mut(&mut self) -> (&mut #ty, &mut #p::Base) {
                        (&mut self.#ident, &mut self.#base_field)
                    }
                }
                impl #trait_path for #struct_name {}
            }
        });
    let assert_base_type = quote! {
        const _: () = {
            #[allow(non_snake_case)]
            fn __assert_base_field #impl_generics (v: &#struct_name #ty_generics) #where_clause {
                let _base: &#p::Base = &v.#base_field;
            }
        };
    };
    quote! {
        #assert_base_type
        #(#injected_methods)*
        impl #impl_generics #p::GameObjectBase for #struct_name #ty_generics {
            fn base(&self) -> &#p::Base {
                &self.#base_field
            }

            fn base_mut(&mut self) -> &mut #p::Base {
                &mut self.#base_field
            }
        }

        impl #impl_generics #p::GameObjectDispatch for #struct_name #ty_generics #where_tokens {
            fn is_pending_removal(&self) -> bool {
                self.base().pending_removal
            }

            fn dispatch_start(&mut self, ctx: &mut impl #p::EngineApi, parent_base: &#p::Base) {
                if self.is_started() {
                    return;
                }
                #apply_transform
                ctx.register_alive(self.base().id);
                #register_subscriptions
                self.start(ctx);
                #(self.#component_fields.start(ctx, &mut self.#base_field);)*
                #(self.#object_fields.dispatch_start(ctx, &self.#base_field);)*
                self.mark_as_started();
            }

            fn dispatch_events(&mut self, ctx: &mut impl #p::EngineApi) {
                #event_dispatch_block
                #broadcast_read_block
                #(self.#object_fields.dispatch_events(ctx);)*
            }

            fn dispatch_update(&mut self, ctx: &mut impl #p::EngineApi, parent_base: &#p::Base, delta: f32) {
                #ensure_started
                self.update(ctx, delta);
                #(self.#component_fields.update(ctx, &mut self.#base_field, delta);)*
                #(self.#object_fields.dispatch_update(ctx, &self.#base_field, delta);)*
            }

            fn dispatch_late_update(&mut self, ctx: &mut impl #p::EngineApi, parent_base: &#p::Base, delta: f32) {
                #ensure_started
                self.late_update(ctx, delta);
                #(self.#component_fields.late_update(ctx, &mut self.#base_field, delta);)*
                #(self.#object_fields.dispatch_late_update(ctx, &self.#base_field, delta);)*
            }

            fn dispatch_fixed_update(&mut self, ctx: &mut impl #p::EngineApi, parent_base: &#p::Base, delta: f32) {
                #ensure_started
                self.fixed_update(ctx, delta);
                #(self.#component_fields.fixed_update(ctx, &mut self.#base_field, delta);)*
                #(self.#object_fields.dispatch_fixed_update(ctx, &self.#base_field, delta);)*
            }

            fn dispatch_draw(&mut self, renderer: &mut impl #p::RenderApi, parent_base: &#p::Base, blending: f32) {
                #apply_transform
                self.draw(renderer, blending);
                #(self.#component_fields.draw(renderer, &self.#base_field, blending);)*
                #(self.#object_fields.dispatch_draw(renderer, &self.#base_field, blending);)*
            }

            fn dispatch_destroy(&mut self, ctx: &mut impl #p::EngineApi) {
                self.destroy(ctx);

                #unregister_subscriptions
                #(self.#component_fields.destroy(ctx, &self.#base_field);)*
                #(self.#object_fields.dispatch_destroy(ctx);)*
                
                ctx.abort_tasks_of(self.base().id);
                ctx.unregister_alive(self.base().id);
                ctx.destroy(self.base().id);
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

fn gen_dispatch_method(
    variants: &[&Ident],
    name: &str,
    sig: proc_macro2::TokenStream,
    ret: proc_macro2::TokenStream,
    call_args: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let method = Ident::new(name, proc_macro2::Span::call_site());
    quote! {
        fn #method(#sig) #ret {
            match self {
                #(Self::#variants(inner) => inner.#method(#call_args),)*
            }
        }
    }
}

fn derive_object_dispatch_enum(
    input: &syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, TokenStream> {
    let crate_name = get_crate_name();
    let p = quote!(::#crate_name::prelude);
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

    let v = &variant_idents;

    let dispatch_methods: Vec<_> = [
        (
            "dispatch_start",
            quote!(&mut self, ctx: &mut impl #p::EngineApi, parent_base: &#p::Base),
            quote!(),
            quote!(ctx, parent_base),
        ),
        (
            "dispatch_events",
            quote!(&mut self, ctx: &mut impl #p::EngineApi),
            quote!(),
            quote!(ctx),
        ),
        (
            "dispatch_update",
            quote!(&mut self, ctx: &mut impl #p::EngineApi, parent_base: &#p::Base, delta: f32),
            quote!(),
            quote!(ctx, parent_base, delta),
        ),
        (
            "dispatch_late_update",
            quote!(&mut self, ctx: &mut impl #p::EngineApi, parent_base: &#p::Base, delta: f32),
            quote!(),
            quote!(ctx, parent_base, delta),
        ),
        (
            "dispatch_fixed_update",
            quote!(&mut self, ctx: &mut impl #p::EngineApi, parent_base: &#p::Base, delta: f32),
            quote!(),
            quote!(ctx, parent_base, delta),
        ),
        (
            "dispatch_draw",
            quote!(&mut self, renderer: &mut impl #p::RenderApi, parent_base: &#p::Base, blending: f32),
            quote!(),
            quote!(renderer, parent_base, blending),
        ),
        (
            "dispatch_destroy",
            quote!(&mut self, ctx: &mut impl #p::EngineApi),
            quote!(),
            quote!(ctx),
        ),
        ("is_pending_removal", quote!(&self), quote!(-> bool), quote!()),
    ]
    .into_iter()
    .map(|(name, sig, ret, args)| gen_dispatch_method(v, name, sig, ret, args))
    .collect();

    let base_methods: Vec<_> = [
        ("base", quote!(&self), quote!(-> &#p::Base), quote!()),
        (
            "base_mut",
            quote!(&mut self),
            quote!(-> &mut #p::Base),
            quote!(),
        ),
    ]
    .into_iter()
    .map(|(name, sig, ret, args)| gen_dispatch_method(v, name, sig, ret, args))
    .collect();

    Ok(quote! {
        impl #p::GameObjectDispatch for #name {
            #(#dispatch_methods)*
        }
        impl #p::GameObjectBase for #name {
            #(#base_methods)*
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
    let crate_name = get_crate_name();
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;

    let dispatch_impl = match derive_object_dispatch_enum(&input) {
        Ok(tokens) => tokens,
        Err(err) => return err,
    };

    let scene_impl = quote! {
        impl ::#crate_name::prelude::Scene for #name {
            fn get_dispatch(&mut self) -> &mut impl ::#crate_name::prelude::GameObjectDispatch {
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
