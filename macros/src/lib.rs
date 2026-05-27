use darling::{Error, FromDeriveInput, FromField, ast::Data};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, Meta, Path, Token, Type, parse::Parse, punctuated::Punctuated};

#[derive(Debug, FromField)]
#[darling(attributes(game))]
struct GameField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    base: bool,
    #[darling(default)]
    component: bool,
    #[darling(default)]
    object: bool,
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
        _ => return Err(Error::custom("Use formato: subscribe(metodo: Tipo)")),
    };

    let parser = Punctuated::<Subscription, Token![,]>::parse_terminated;

    let subs = list
        .parse_args_with(parser)
        .map_err(|e| Error::custom(e.to_string()).with_span(meta))?;
    Ok(subs.into_iter().collect())
}

#[proc_macro_derive(GameObject, attributes(game))]
pub fn scene_tree(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let receiver = match GameReceiver::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let subscribe_block = if !receiver.subscribe.is_empty() {
        let arms = receiver.subscribe.iter().map(|sub| {
            let event_ty = &sub.event_type;
            let handler_ident = &sub.handler;

            quote! {
                if let Some(payload) = any_event.downcast_ref::<#event_ty>(){
                    self.#handler_ident(ctx, payload);
                }
            }
        });
        quote! {
            if let ::alone_engine::GlobalEvent::Broadcast(any_event) = event{
                #(#arms)*
            }
        }
    } else {
        quote! {}
    };

    let connect_block = if !receiver.connect.is_empty() {
        let arms = receiver.connect.iter().map(|sub| {
            let event_ty = &sub.event_type;
            let handler_ident = &sub.handler;

            quote! {
                if let Some(payload) = any_event.downcast_ref::<#event_ty>(){
                    self.#handler_ident(ctx, payload);
                }
            }
        });
        quote! {
            if let ::alone_engine::GlobalEvent::Targeted(id, any_event) = event{
                if &self.base().id == id{
                    #(#arms)*
                    return;
                }
            }

        }
    } else {
        quote! {}
    };

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
                return syn::Error::new_spanned(ty, "Precisa ser do tipo Transform2D")
                    .to_compile_error()
                    .into();
            }
        } else if field.component {
            component_fields.push(ident.clone());
            bounds.push(quote! {#ty: ::alone_engine::Component});
        } else if field.object {
            object_fields.push(ident.clone());
            bounds.push(
                quote! {#ty: ::alone_engine::GameObject + ::alone_engine::GameObjectDispatch},
            );
        }
    }
    let apply_transform = quote! {
        let inherit = !self.#base_field.top_level;
        self.#base_field.transform.apply_parent(&parent_base.transform, inherit);
    };
    let (impl_generics, ty_generics, where_clause) = receiver.generics.split_for_impl();

    let where_tokens = if let Some(wc) = where_clause {
        quote! {#wc,  Self: ::alone_engine::GameObject, #(#bounds),*}
    } else {
        quote! {where  Self: ::alone_engine::GameObject, #(#bounds),*}
    };
    quote! {
        impl #impl_generics ::alone_engine::GameObjectBase for #struct_name #ty_generics {
            fn base(&self) -> &::alone_engine::Base {
                &self.#base_field
            }

            fn base_mut(&mut self) -> &mut ::alone_engine::Base {
                &mut self.#base_field
            }
        }
        impl #impl_generics ::alone_engine::GameObjectDispatch for #struct_name #ty_generics #where_tokens {
            fn is_pending_removal(&self) -> bool{
                self.base().pending_removal
            }
            fn dispatch_start(&mut self, ctx: &mut impl ::alone_engine::EngineApi, parent_base: &::alone_engine::Base) {
                if self.is_started(){
                    return;
                }
                self.start(ctx);
                #(self.#component_fields.start(ctx, &mut self.#base_field);)*
                #( self.#object_fields.dispatch_start(ctx, &self.#base_field); )*
            }
            fn dispatch_message(&mut self, ctx: &mut impl ::alone_engine::EngineApi){
                {
                    if ctx.mail_box_is_empty(){
                        return;
                    }
                }
                {
                    let mailbox = ctx.mailbox();
                    if let Some(msgs) = mailbox.remove(&self.base().id){
                        for msg in msgs {
                            if let Some(message) = msg.downcast_ref::<<Self as ::alone_engine::GameObject>::Message>(){
                                self.on_message(ctx, message)
                            } else{
                                println!("Tipo de evento incompativel recebido");
                            }
                        }
                    }
                }
                if ctx.mail_box_is_empty(){
                    return;
                }
                #(self.#object_fields.dispatch_message(ctx);)*
            }
            fn dispatch_event(&mut self, ctx: &mut impl ::alone_engine::EngineApi, event: &::alone_engine::GlobalEvent){

                #subscribe_block
                #connect_block
                #(self.#component_fields.on_event(ctx, &mut self.#base_field, event);)*
                #(self.#object_fields.dispatch_event(ctx, event);)*
            }
            fn dispatch_update(&mut self, ctx: &mut impl ::alone_engine::EngineApi, parent_base: &::alone_engine::Base, delta: f32) {
                #apply_transform
                if !self.is_started(){
                    self.dispatch_start(ctx, parent_base);
                    self.on_start();
                }
                self.update(ctx, delta);

                #(self.#component_fields.update(ctx, &mut self.#base_field, delta);)*

                #( self.#object_fields.dispatch_update(ctx, &self.#base_field, delta); )*
            }
            fn dispatch_late_update(&mut self, ctx: &mut impl ::alone_engine::EngineApi, parent_base: &::alone_engine::Base, delta: f32) {
                #apply_transform
                if !self.is_started(){
                    self.dispatch_start(ctx, parent_base);
                    self.on_start();
                }
                self.late_update(ctx, delta);

                #(self.#component_fields.late_update(ctx, &mut self.#base_field, delta);)*

                #( self.#object_fields.dispatch_late_update(ctx, &self.#base_field, delta); )*
            }
            fn dispatch_fixed_update(&mut self, ctx: &mut impl ::alone_engine::EngineApi, parent_base: &::alone_engine::Base, delta: f32) {
                #apply_transform
                if !self.is_started(){
                    self.dispatch_start(ctx, parent_base);
                    self.on_start();
                }
                self.fixed_update(ctx, delta);

                #(self.#component_fields.fixed_update(ctx, &mut self.#base_field, delta);)*

                #( self.#object_fields.dispatch_fixed_update(ctx, &self.#base_field, delta); )*
            }
            fn dispatch_draw(&mut self, renderer: &mut impl ::alone_engine::RenderApi, parent_base: &::alone_engine::Base, blending: f32) {
                #apply_transform
                self.draw(renderer, blending);
                
                #(self.#component_fields.draw(renderer, &self.#base_field, blending);)*
                #( self.#object_fields.dispatch_draw(renderer, &self.#base_field, blending); )*

            }
            fn dispatch_destroy(&mut self, ctx: &mut impl ::alone_engine::EngineApi) {
                self.destroy(ctx);
                #(self.#component_fields.destroy(ctx, &self.#base_field);)*
                #( self.#object_fields.dispatch_destroy(ctx); )*
            }
        }
    }
    .into()
}

fn type_is_base(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            if let Some(seg) = type_path.path.segments.last()
                && seg.ident == "Base"
            {
                return true;
            }
            false
        }
        _ => false,
    }
}

#[proc_macro_derive(Scene)]
pub fn scene_dispatch_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;

    let data = match input.data {
        syn::Data::Enum(d) => d,
        _ => {
            return syn::Error::new_spanned(name, "Scene so pode ser utilizado em enum")
                .into_compile_error()
                .into();
        }
    };

    let variants: Vec<_> = data.variants.into_iter().map(|v| v.ident).collect();

    quote! {
        impl ::alone_engine::Scene for #name {
            fn get_dispatch(&mut self) -> &mut impl ::alone_engine::GameObjectDispatch{
                match self{
                    #(Self::#variants(inner) => inner)*
                }
            }

        }
    }
    .into()
}
