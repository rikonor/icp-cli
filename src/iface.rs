use anyhow::Error;
use async_trait::async_trait;
use wasmtime::{
    component::{types::ComponentItem, Component},
    Engine,
};

#[derive(Debug, PartialEq)]
pub struct Interface {
    /// Name of the interface (e.g., "math/lib")
    pub name: String,

    /// Functions provided by this interface
    pub funcs: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct ComponentInterfaces {
    pub imports: Vec<Interface>,
    pub exports: Vec<Interface>,
}

#[async_trait]
pub trait DetectIfaces: Sync + Send {
    async fn detect(&self, ngn: &Engine, cmpnt: &Component) -> Result<ComponentInterfaces, Error>;
}

pub struct IfaceDetector;

#[async_trait]
impl DetectIfaces for IfaceDetector {
    async fn detect(&self, ngn: &Engine, cmpnt: &Component) -> Result<ComponentInterfaces, Error> {
        let typ = cmpnt.component_type();

        // imports
        let mut imps: Vec<Interface> = Vec::new();

        for imp in typ.imports(ngn) {
            let (name, itm) = imp;

            let (iface, itm) = match itm {
                ComponentItem::ComponentInstance(itm) => (name, itm),
                _ => continue,
            };

            let mut fs = vec![];

            for exp in itm.exports(ngn) {
                let (name, itm) = exp;

                let (f, _) = match itm {
                    ComponentItem::ComponentFunc(itm) => (name, itm),
                    _ => continue,
                };

                fs.push(f.to_string());
            }

            imps.push(Interface {
                name: iface.to_string(),
                funcs: fs,
            });
        }

        // exports
        let mut exps: Vec<Interface> = Vec::new();

        for exp in typ.exports(ngn) {
            let (name, itm) = exp;

            let (iface, itm) = match itm {
                ComponentItem::ComponentInstance(itm) => (name, itm),
                _ => continue,
            };

            let mut fs = vec![];

            for exp in itm.exports(ngn) {
                let (name, itm) = exp;

                let (f, _) = match itm {
                    ComponentItem::ComponentFunc(itm) => (name, itm),
                    _ => continue,
                };

                fs.push(f.to_string());
            }

            exps.push(Interface {
                name: iface.to_string(),
                funcs: fs,
            });
        }

        Ok(ComponentInterfaces {
            imports: imps,
            exports: exps,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use wasmtime::Config;

    #[tokio::test]
    async fn test_detect() -> Result<(), Error> {
        let mut cfg = Config::new();
        let cfg = cfg.async_support(true);

        let ngn = Engine::new(cfg)?;

        let wat = r#"
          (component
            ;; Import the misc interface from the host
            (import "local:host/misc"
              (instance
                (export "print"
                  (func (param "s" string)))))

            ;; Define a core module that will implement our cli interface
            (core module $impl
              (memory (export "mem") 1)

              ;; Define the run function that returns a u8 (i32 in core wasm)
              (func $local:guest/cli#run (result i32)
                i32.const 42)  ;; Just return a constant value

              ;; Export the function with the expected name
              (export "local:guest/cli#run" (func $local:guest/cli#run))
            )

            ;; Create an instance of our core module
            (core instance $instance (instantiate $impl))

            ;; Lift the core function to a component function
            (func $run (result u8)
              (canon lift
                (core func $instance "local:guest/cli#run")))

            ;; Export the cli interface
            (instance $cli
              (export "run" (func $run)))

            (export "local:guest/cli" (instance $cli))
          )
        "#;

        let cmpnt = Component::new(&ngn, wat)?;

        // Create a detector
        let ifaces = IfaceDetector
            .detect(&ngn, &cmpnt)
            .await
            .context("failed to detect imports and exports")?;

        assert_eq!(
            ifaces,
            ComponentInterfaces {
                imports: vec![Interface {
                    name: "local:host/misc".to_string(),
                    funcs: vec!["print".to_string()],
                }],
                exports: vec![Interface {
                    name: "local:guest/cli".to_string(),
                    funcs: vec!["run".to_string()],
                }],
            }
        );

        Ok(())
    }
}
