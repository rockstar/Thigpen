use std::fmt::Write;
use std::path::Path;

#[derive(Debug)]
struct PublicIdentifier {
    r#type: String,
    name: String,
}

impl From<syn::UseTree> for PublicIdentifier {
    fn from(value: syn::UseTree) -> Self {
        match value {
            syn::UseTree::Path(usepath) => (*usepath.tree).into(),
            syn::UseTree::Name(name) => Self {
                name: name.ident.to_string(),
                r#type: "use".into(),
            },
            syn::UseTree::Rename(rename) => Self {
                name: rename.rename.to_string(),
                r#type: "use".into(),
            },
            syn::UseTree::Glob(_) => Self {
                name: "*".into(),
                r#type: "use".into(),
            },
            syn::UseTree::Group(_group) => Self {
                name: "group".into(),
                r#type: "use".into(),
            }, // TODO
        }
    }
}

/// A root crate
#[derive(Debug)]
#[allow(dead_code)]
pub struct Lib {
    name: String,
    path: Box<Path>,
    interface: Vec<PublicIdentifier>,
    modules: Vec<Module>,
}

impl Lib {
    pub fn from_path(name: &str, path: &Path) -> Self {
        assert_eq!(path.file_name(), Some(std::ffi::OsStr::new("lib.rs")));

        let contents = std::fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("Could not read {}: {}", path.to_str().unwrap(), err));
        let parsed_file = syn::parse_file(&contents).expect("Unable to parse file.");

        let children: Vec<Module> = parsed_file
            .items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Mod(inner) => {
                    let module = Module::from_path(
                        &format!("{}::{}", name, inner.ident),
                        path.parent().unwrap(),
                    );
                    Some(module)
                }
                _ => None,
            })
            .collect();

        let interface: Vec<PublicIdentifier> = parsed_file
            .items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Const(item) => {
                    if let syn::Visibility::Public(_) = item.vis {
                        Some(PublicIdentifier {
                            name: item.ident.to_string(),
                            r#type: "const".into(),
                        })
                    } else {
                        None
                    }
                }
                syn::Item::Enum(item) => {
                    if let syn::Visibility::Public(_) = item.vis {
                        Some(PublicIdentifier {
                            name: item.ident.to_string(),
                            r#type: "enum".into(),
                        })
                    } else {
                        None
                    }
                }
                syn::Item::ExternCrate(item) => {
                    if let syn::Visibility::Public(_) = item.vis {
                        Some(PublicIdentifier {
                            name: item.ident.to_string(),
                            r#type: "extern crate".into(),
                        })
                    } else {
                        None
                    }
                }
                syn::Item::Fn(item) => {
                    if let syn::Visibility::Public(_) = item.vis {
                        Some(PublicIdentifier {
                            name: item.sig.ident.to_string(),
                            r#type: "fn".into(),
                        })
                    } else {
                        None
                    }
                }
                syn::Item::ForeignMod(_) => None,
                syn::Item::Impl(_) => None,
                syn::Item::Macro(_) => None,
                syn::Item::Mod(item) => {
                    if let syn::Visibility::Public(_) = item.vis {
                        Some(PublicIdentifier {
                            name: item.ident.to_string(),
                            r#type: "mod".into(),
                        })
                    } else {
                        None
                    }
                }
                syn::Item::Static(item) => {
                    if let syn::Visibility::Public(_) = item.vis {
                        Some(PublicIdentifier {
                            name: item.ident.to_string(),
                            r#type: "static".into(),
                        })
                    } else {
                        None
                    }
                }
                syn::Item::Struct(item) => {
                    if let syn::Visibility::Public(_) = item.vis {
                        Some(PublicIdentifier {
                            name: item.ident.to_string(),
                            r#type: "struct".into(),
                        })
                    } else {
                        None
                    }
                }
                syn::Item::Trait(item) => {
                    if let syn::Visibility::Public(_) = item.vis {
                        Some(PublicIdentifier {
                            name: item.ident.to_string(),
                            r#type: "trait".into(),
                        })
                    } else {
                        None
                    }
                }
                syn::Item::TraitAlias(item) => {
                    if let syn::Visibility::Public(_) = item.vis {
                        Some(PublicIdentifier {
                            name: item.ident.to_string(),
                            r#type: "trait alias".into(),
                        })
                    } else {
                        None
                    }
                }
                syn::Item::Type(item) => {
                    if let syn::Visibility::Public(_) = item.vis {
                        Some(PublicIdentifier {
                            name: item.ident.to_string(),
                            r#type: "type".into(),
                        })
                    } else {
                        None
                    }
                }
                syn::Item::Union(item) => {
                    if let syn::Visibility::Public(_) = item.vis {
                        Some(PublicIdentifier {
                            name: item.ident.to_string(),
                            r#type: "union".into(),
                        })
                    } else {
                        None
                    }
                }
                syn::Item::Use(item) => {
                    if let syn::Visibility::Public(_) = item.vis {
                        Some(item.tree.clone().into())
                    } else {
                        None
                    }
                }
                syn::Item::Verbatim(_) => None,
                _ => None,
            })
            .collect();

        Self {
            name: name.into(),
            path: path.into(),
            interface,
            modules: children,
        }
    }

    #[allow(dead_code)]
    fn create_dot(&self) -> String {
        let mut dot = String::new();
        writeln!(&mut dot, "graph graphname {{").unwrap();
        for module in self.modules.iter() {
            writeln!(&mut dot, "\"{}\" -- \"{}\"", self.name, module.usepath).unwrap();
            module.write_dot(&mut dot);
        }
        writeln!(&mut dot, "}}").unwrap();

        dot
    }

    pub fn create_mermaid(&self) -> String {
        let mut mermaid = String::new();
        write!(
            &mut mermaid,
            r#"---
title: {} entity diagram
---
erDiagram
"#,
            self.name
        )
        .unwrap();
        self.modules.iter().for_each(|module| {
            writeln!(
                &mut mermaid,
                "  \"{}\" ||--|{{ \"{}\" : \"\"",
                self.name, module.usepath
            )
            .unwrap();
            module.write_mermaid(&mut mermaid);
        });
        writeln!(&mut mermaid, "  \"{}\" {{", self.name).unwrap();
        self.interface.iter().for_each(|item| {
            writeln!(&mut mermaid, "    {} {}", item.r#type, item.name).unwrap();
        });
        writeln!(&mut mermaid, "  }}").unwrap();

        mermaid
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct Module {
    name: String,
    usepath: String,
    path: Option<Box<Path>>,
    interface: Vec<PublicIdentifier>,
    children: Vec<Module>,
    dependencies: Vec<Module>,
}

impl Module {
    fn from_path(usepath: &str, path: &Path) -> Self {
        assert_ne!(path.file_name(), Some(std::ffi::OsStr::new("lib.rs")));
        assert_ne!(path.extension(), Some(std::ffi::OsStr::new("rs")));
        assert!(path.exists(), "Path at {:?} does not exist", path);
        assert!(path.is_dir(), "Path at {:?} is not a directory", path);

        let name = usepath.split("::").last().unwrap();

        let paths: Vec<std::path::PathBuf> = vec![
            {
                let mut path = path.to_path_buf();
                path.push(name);
                path.push("mod");
                path.set_extension("rs");
                path
            },
            {
                let mut path = path.to_path_buf();
                path.push(name);
                path.set_extension("rs");
                path
            },
        ]
        .into_iter()
        .filter(|path| path.exists())
        .collect();
        if paths.is_empty() {
            // Either the module lives inside the declaring file, or the module
            // doesn't exist on disk. Either way, drop a "dummy" module here.
            return Self {
                name: name.into(),
                usepath: usepath.into(),
                path: None,
                interface: vec![],
                children: vec![],
                dependencies: vec![],
            };
        }

        let (children, interface): (Vec<Module>, Vec<PublicIdentifier>) = paths
            .iter()
            .map(|modpath| {
                log::debug!("Reading/parsing {:?}", path);

                let contents = std::fs::read_to_string(modpath).unwrap_or_else(|err| {
                    panic!("Could not read {}: {}", modpath.to_str().unwrap(), err)
                });
                let parsed_file = syn::parse_file(&contents).expect("Unable to parse file.");
                let modules: Vec<Module> = parsed_file
                    .items
                    .iter()
                    .filter_map(|item| match item {
                        syn::Item::Mod(inner) => {
                            let mut path = path.to_path_buf();
                            path.push(name);
                            if path.exists() {
                                Some(Module::from_path(
                                    &format!("{}::{}", usepath, inner.ident),
                                    &path,
                                ))
                            } else {
                                None
                            }
                        }
                        syn::Item::Use(_inner) => None,
                        _ => None,
                    })
                    .collect();

                let interface: Vec<PublicIdentifier> = parsed_file
                    .items
                    .iter()
                    .filter_map(|item| match item {
                        syn::Item::Const(item) => {
                            if let syn::Visibility::Public(_) = item.vis {
                                Some(PublicIdentifier {
                                    name: item.ident.to_string(),
                                    r#type: "const".into(),
                                })
                            } else {
                                None
                            }
                        }
                        syn::Item::Enum(item) => {
                            if let syn::Visibility::Public(_) = item.vis {
                                Some(PublicIdentifier {
                                    name: item.ident.to_string(),
                                    r#type: "enum".into(),
                                })
                            } else {
                                None
                            }
                        }
                        syn::Item::ExternCrate(item) => {
                            if let syn::Visibility::Public(_) = item.vis {
                                Some(PublicIdentifier {
                                    name: item.ident.to_string(),
                                    r#type: "extern crate".into(),
                                })
                            } else {
                                None
                            }
                        }
                        syn::Item::Fn(item) => {
                            if let syn::Visibility::Public(_) = item.vis {
                                Some(PublicIdentifier {
                                    name: item.sig.ident.to_string(),
                                    r#type: "fn".into(),
                                })
                            } else {
                                None
                            }
                        }
                        syn::Item::ForeignMod(_) => None,
                        syn::Item::Impl(_) => None,
                        syn::Item::Macro(_) => None,
                        syn::Item::Mod(item) => {
                            if let syn::Visibility::Public(_) = item.vis {
                                Some(PublicIdentifier {
                                    name: item.ident.to_string(),
                                    r#type: "mod".into(),
                                })
                            } else {
                                None
                            }
                        }
                        syn::Item::Static(item) => {
                            if let syn::Visibility::Public(_) = item.vis {
                                Some(PublicIdentifier {
                                    name: item.ident.to_string(),
                                    r#type: "static".into(),
                                })
                            } else {
                                None
                            }
                        }
                        syn::Item::Struct(item) => {
                            if let syn::Visibility::Public(_) = item.vis {
                                Some(PublicIdentifier {
                                    name: item.ident.to_string(),
                                    r#type: "struct".into(),
                                })
                            } else {
                                None
                            }
                        }
                        syn::Item::Trait(item) => {
                            if let syn::Visibility::Public(_) = item.vis {
                                Some(PublicIdentifier {
                                    name: item.ident.to_string(),
                                    r#type: "trait".into(),
                                })
                            } else {
                                None
                            }
                        }
                        syn::Item::TraitAlias(item) => {
                            if let syn::Visibility::Public(_) = item.vis {
                                Some(PublicIdentifier {
                                    name: item.ident.to_string(),
                                    r#type: "trait alias".into(),
                                })
                            } else {
                                None
                            }
                        }
                        syn::Item::Type(item) => {
                            if let syn::Visibility::Public(_) = item.vis {
                                Some(PublicIdentifier {
                                    name: item.ident.to_string(),
                                    r#type: "type".into(),
                                })
                            } else {
                                None
                            }
                        }
                        syn::Item::Union(item) => {
                            if let syn::Visibility::Public(_) = item.vis {
                                Some(PublicIdentifier {
                                    name: item.ident.to_string(),
                                    r#type: "union".into(),
                                })
                            } else {
                                None
                            }
                        }
                        syn::Item::Use(item) => {
                            if let syn::Visibility::Public(_) = item.vis {
                                Some(item.tree.clone().into())
                            } else {
                                None
                            }
                        }
                        syn::Item::Verbatim(_) => None,
                        _ => None,
                    })
                    .collect();

                (modules, interface)
            })
            .fold((vec![], vec![]), |mut acc, x| {
                acc.0.extend(x.0);
                acc.1.extend(x.1);
                acc
            });

        Self {
            name: name.into(),
            usepath: usepath.into(),
            path: Some(path.into()),
            interface,
            children,
            dependencies: vec![],
        }
    }

    fn write_dot(&self, mut wrt: &mut dyn std::fmt::Write) {
        for module in self.children.iter() {
            writeln!(&mut wrt, "\"{}\" -- \"{}\"", self.usepath, module.usepath).unwrap();
            module.write_dot(wrt);
        }
    }

    fn write_mermaid(&self, mut wrt: &mut dyn std::fmt::Write) {
        self.children.iter().for_each(|module| {
            writeln!(
                &mut wrt,
                "  \"{}\" ||--|{{ \"{}\" : \"\"",
                self.usepath, module.usepath
            )
            .unwrap();
            module.write_mermaid(&mut wrt);
        });

        writeln!(&mut wrt, "  \"{}\" {{", self.usepath).unwrap();
        self.interface.iter().for_each(|item| {
            writeln!(&mut wrt, "    {} {}", item.r#type, item.name).unwrap();
        });
        writeln!(&mut wrt, "  }}").unwrap();
    }
}
