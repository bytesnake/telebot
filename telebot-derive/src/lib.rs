    #![feature(try_from)]
    #![recursion_limit="150"]

    extern crate log;
    extern crate proc_macro;
    #[macro_use]
    extern crate quote;
    extern crate syn;

    use proc_macro::TokenStream;
    use std::collections::BTreeMap;

    #[proc_macro_derive(setter)]
    pub fn derive_setter(input: TokenStream) -> TokenStream {
        let ast = syn::parse_macro_input(&input.to_string()).unwrap();
        let expanded = expand_setter(ast);
        expanded.to_string().parse().unwrap()
    }

    fn expand_setter(ast: syn::MacroInput) -> quote::Tokens {
        let config = config_from(&ast.attrs);

        let query_kind = config.get("query").map(|tmp| syn::Lit::from(tmp.as_str()));
        //let file_kind = config.get("file_kind").map(|tmp| syn::Ident::from(tmp.as_str()));

        let fields: Vec<_> = match ast.body {
            syn::Body::Struct(syn::VariantData::Struct(ref fields)) => {
                fields.iter().map(|f| (f.ident.as_ref().unwrap(), &f.ty)).collect()
            },
            syn::Body::Struct(syn::VariantData::Unit) => {
                vec![]
            },
            _ => panic!("#[derive(getters)] can only be used with braced structs"),
        };

        let name = &ast.ident;
        let is_option_ident = |ref f: &(&syn::Ident, &syn::Ty)| -> bool {
            match *f.1 {
                syn::Ty::Path(_, ref path) => {
                    match path.segments.first().unwrap().ident.as_ref() {
                        "Option" => true,
                        _ => false
                    }
                },
                _ => false
            }
        };

        let field_compulsory: Vec<_> = fields.iter().filter(|f| !is_option_ident(&f))
            .filter(|f| f.0.as_ref() != "kind" && f.0.as_ref() != "id")
            .map(|f| syn::Ident::from(format!("_{}", f.0.as_ref())))
            .collect();

        let field_optional: Vec<_> = fields.iter().filter(|f| is_option_ident(&f)).map(|f| f.0).collect();
        let field_optional2 = field_optional.clone();

        let field_compulsory2: Vec<_> = fields.iter().map(|f| f.0).filter(|f| f.as_ref() != "kind" && f.as_ref() != "id").collect();


        let field_compulsory3 = field_compulsory.clone();
        let values: Vec<_> = fields.iter().filter(|f| f.0.as_ref() != "kind" && f.0.as_ref() != "id").map(|f| {
            match *f.1 {
                syn::Ty::Path(_, ref path) => {
                    match path.segments.first().unwrap().ident.as_ref() {
                        "Option" => return syn::Ident::from("None"),
                        _ => return syn::Ident::from(format!("_{}", f.0.as_ref()))
                    }
                },
                _ => return syn::Ident::from("None")
            }
        }).collect();

        //let ty_compulsory: Vec<_> = fields.iter().map(|f| f.1).collect();
        let ty_compulsory2: Vec<_> = fields.iter().filter(|f| f.0.as_ref() != "kind" && f.0.as_ref() != "id").map(|f| f.1).collect();
        let ty_optional: Vec<_> = fields.iter().filter(|f| is_option_ident(&f)).map(|f| {
            if let syn::Ty::Path(_, ref path) = *f.1 {
                if let syn::PathParameters::AngleBracketed(ref param) = path.segments.first().unwrap().parameters {
                    if let &syn::Ty::Path(_, ref path) = param.types.first().unwrap() {
                        return (*path).clone();
                    }
                }
            }

            panic!("no sane type!");
        }).collect();

        //println!("{:?}", ty_optional.first());

        //let trait_name = syn::Ident::from(format!("Function{}",  name.as_ref()));
        //let wrapper_name = syn::Ident::from(format!("Wrapper{}", name.as_ref()));

        if let Some(query_name) = query_kind {
            quote! {
                impl #name {
                    #[allow(dead_code)]
                    pub fn new(#( #field_compulsory3: #ty_compulsory2, )*) -> #name {
                        let id = Uuid::new_v4();

                        #name { kind: #query_name.into(), id: id.hyphenated().to_string(), #( #field_compulsory2: #values, )* }
                    }
                    #(
                        pub fn #field_optional<S>(mut self, val: S) -> Self where S: Into<#ty_optional> {
                            self.#field_optional2 = Some(val.into());

                            self
                        }
                    )*
                }

            }
        } else {
            quote! {
                impl #name {
                    #[allow(dead_code)]
                    pub fn new(#( #field_compulsory3: #ty_compulsory2, )*) -> #name {
                        #name { #( #field_compulsory2: #values, )* }
                    }
                    #(
                        pub fn #field_optional<S>(mut self, val: S) -> Self where S: Into<#ty_optional> {
                            self.#field_optional2 = Some(val.into());

                            self
                        }
                    )*
                }

            }
        }
    }

    #[proc_macro_derive(TelegramFunction)]
    pub fn derive_telegram_sendable(input: TokenStream) -> TokenStream {
        let ast = syn::parse_macro_input(&input.to_string()).unwrap();
        let expanded = expand_function(ast);
        expanded.to_string().parse().unwrap()
    }

fn expand_function(ast: syn::MacroInput) -> quote::Tokens {
    let config = config_from(&ast.attrs);

    let function = config.get("call").unwrap();
    let function = syn::Lit::Str((*function).clone(), syn::StrStyle::Cooked);
    let bot_function = syn::Ident::from(config.get("function").unwrap().as_str());
    let answer = syn::Ident::from(config.get("answer").unwrap().as_str());
    let file_kind = config.get("file_kind").map(|tmp| syn::Ident::from(tmp.as_str()));

    let fields: Vec<_> = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => {
            fields.iter().map(|f| (f.ident.as_ref().unwrap(), &f.ty)).collect()
        },
        syn::Body::Struct(syn::VariantData::Unit) => {
            vec![]
        },
        _ => panic!("#[derive(getters)] can only be used with braced structs"),
    };


        /*for field in &fields {
        println!("{:?}", field.1);
    }*/

    let name = &ast.ident;
    let is_option_ident = |ref f: &(&syn::Ident, &syn::Ty)| -> bool {
        match *f.1 {
            syn::Ty::Path(_, ref path) => {
                match path.segments.first().unwrap().ident.as_ref() {
                    "Option" => true,
                    _ => false
                }
            },
            _ => false
        }
    };

    let field_compulsory: Vec<_> = fields.iter().filter(|f| !is_option_ident(&f))
        .map(|f| syn::Ident::from(format!("_{}", f.0.as_ref()))).collect();

    let field_optional: Vec<_> = fields.iter().filter(|f| is_option_ident(&f)).map(|f| f.0).collect();
    let field_optional2 = field_optional.clone();

    let field_compulsory2: Vec<_> = fields.iter().map(|f| f.0).collect();
    let field_compulsory3 = field_compulsory.clone();
    let values: Vec<_> = fields.iter().map(|f| {
        match *f.1 {
            syn::Ty::Path(_, ref path) => {
                match path.segments.first().unwrap().ident.as_ref() {
                    "Option" => return syn::Ident::from("None"),
                    _ => return syn::Ident::from(format!("_{}", f.0.as_ref()))
                }
            },
            _ => return syn::Ident::from("None")
        }
    }).collect();

    let ty_compulsory: Vec<_> = fields.iter().map(|f| f.1).collect();
    let ty_compulsory2 = ty_compulsory.clone();
    let ty_optional: Vec<_> = fields.iter().filter(|f| is_option_ident(&f)).map(|f| {
        if let syn::Ty::Path(_, ref path) = *f.1 {
            if let syn::PathParameters::AngleBracketed(ref param) = path.segments.first().unwrap().parameters {
                if let &syn::Ty::Path(_, ref path) = param.types.first().unwrap() {
                    return (*path).clone();
                }
            }
        }

        panic!("no sane type!");
    }).collect();

    //println!("{:?}", ty_optional.first());

    let trait_name = syn::Ident::from(format!("Function{}",  name.as_ref()));
    let wrapper_name = syn::Ident::from(format!("Wrapper{}", name.as_ref()));

    let tokens = quote! {
        #[allow(dead_code)]
        pub struct #wrapper_name {
            bot: Rc<Bot>,
            inner: #name,
            file: Result<file::FileList, Error>
        }
    };

    if let Some(file_kind) = file_kind {
        let file_kind_name = syn::Lit::Str(format!("{}", file_kind), syn::StrStyle::Cooked);
        quote! {
            #tokens

            pub trait #trait_name {
                 fn #bot_function(&self, #( #field_compulsory: #ty_compulsory, )*) -> #wrapper_name;
            }

            impl #trait_name for RcBot {
                fn #bot_function(&self, #( #field_compulsory3: #ty_compulsory2, )*) -> #wrapper_name {
                    #wrapper_name { inner: #name { #( #field_compulsory2: #values, )* }, bot: self.inner.clone(), file: Ok(file::FileList(Vec::new())) }
                }
            }
            impl #wrapper_name {
                pub fn send<'a>(self) -> impl Future<Item=(RcBot, objects::#answer), Error=Error> + 'a{
                    use futures::future::result;
                    use futures::IntoFuture;

                    let cloned_bot = self.bot.clone();

                    result::<#wrapper_name, Error>(Ok(self))
                        .and_then(move |mut tmp| {
                            let #wrapper_name { bot, mut inner, mut file } = tmp;

                            match file {
                                Ok(files) => {
                                    inner.#file_kind = files.to_metadata();

                                    match serde_json::to_value(&inner) {
                                        Ok(msg) => {
                                            Ok((bot, msg, files.into_files()))
                                        },
                                        Err(err) => Err(Error::from(err.context(ErrorKind::JsonSerialize)))
                                     }
                                },
                                Err(err) => Err(err)
                            }
                        })
                        .and_then(move |(bot, msg, files)| {
                            let bot = bot.clone();
                            let bot2 = bot.clone();
                            let msg_str = serde_json::to_string(&msg).unwrap();

                            files.ok_or(Error::from(ErrorKind::NoFile)).into_future()
                                .and_then(move |files| {
                                    bot.fetch_formdata(#function, &msg, files, #file_kind_name)
                                })
                                .or_else(move |_| {
                                    bot2.fetch_json(#function, &msg_str)
                                })
                        })
                        .and_then(move |answer| {
                            let bot = RcBot { inner: cloned_bot };

                            serde_json::from_str::<objects::#answer>(&answer)
                                .map(|json| (bot, json))
                                .map_err(|x| Error::from(x.context(ErrorKind::JsonParse)))
                        })
                }

                #(
                    pub fn #field_optional<S>(mut self, val: S) -> Self where S: Into<#ty_optional> {
                        self.inner.#field_optional2 = Some(val.into());

                        self
                    }
                )*

                pub fn file<S>(mut self, val: S) -> Self where S: TryInto<file::File> {
                    let val = val.try_into();

                    let new_val = match self.file {
                        Ok(mut filelist) => {
                            match val {
                                Ok(val) => {
                                    filelist.push(file::FileWithCaption::new_empty(val));
                                    Ok(filelist)
                                },
                                Err(err) => {
                                    Err(Error::from(ErrorKind::NoFile))
                                }
                            }
                        },
                        Err(_) => {
                            Err(Error::from(ErrorKind::NoFile))
                        }
                    };

                    self.file = new_val;

                    self
                }
            }
        }
    } else {
        quote! {
            #tokens

            pub trait #trait_name {
                 fn #bot_function(&self, #( #field_compulsory: #ty_compulsory, )*) -> #wrapper_name;
            }

            impl #trait_name for RcBot {
                fn #bot_function(&self, #( #field_compulsory3: #ty_compulsory2, )*) -> #wrapper_name {
                    #wrapper_name { inner: #name { #( #field_compulsory2: #values, )* }, bot: self.inner.clone(), file: Ok(file::FileList(Vec::new())) }
                }
            }
            impl #wrapper_name {
                pub fn send<'a>(self) -> impl Future<Item=(RcBot, objects::#answer), Error=Error> + 'a{
                    use futures::future::result;
                    result(serde_json::to_string(&self.inner))
                        .map_err(|e| Error::from(e.context(ErrorKind::JsonSerialize)))
                        .and_then(move |msg| {
                            let obj = self.bot.fetch_json(#function, &msg)
                                .and_then(move |x| {
                                    let bot = RcBot {
                                        inner: self.bot.clone(),
                                    };

                                    serde_json::from_str::<objects::#answer>(&x)
                                        .map(|json| (bot, json))
                                        .map_err(|x| Error::from(x.context(ErrorKind::JsonParse)))
                                });

                            Box::new(obj)
                        })
                }

                #(
                    pub fn #field_optional<S>(mut self, val: S) -> Self where S: Into<#ty_optional> {
                        self.inner.#field_optional2 = Some(val.into());

                        self
                    }
                )*
            }
        }
    }
}

fn config_from(attrs: &[syn::Attribute]) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    for attr in attrs {
        if let syn::MetaItem::NameValue(ref name, ref value) = attr.value {
            let name = format!("{}", name);
            let value = match value.clone() {
                syn::Lit::Str(value, _) => value,
                _ => panic!("bla")
            };
            result.insert(name, value);
        }
    }
    result
}

