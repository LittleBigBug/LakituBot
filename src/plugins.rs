use std::io;
use std::rc::Rc;
use std::ffi::OsStr;
use std::sync::{RwLock, RwLockWriteGuard};
use std::collections::HashMap;
use libloading::Library;
use lakitu_lib::api::{LakituPlugin, LakituPluginRegistrar, LakituPluginDeclaration};
use lakitu_lib::platform::event::{LakituEventManager, LakituEvent, LakituEventID, LakituEventHandler, LakituEventHandlerID};

pub struct LakituPluginProxy {
    plugin: Box<dyn LakituPlugin>,
    _lib: Rc<Library>,
}

impl LakituPlugin for LakituPluginProxy {
    fn get_name(&self) -> &str {
        self.plugin.get_name()
    }

    fn get_version(&self) -> &str {
        self.plugin.get_version()
    }

    fn get_author(&self) -> &str {
        self.plugin.get_author()
    }

    fn get_description(&self) -> &str {
        self.plugin.get_description()
    }

    fn plugin_enable(&self) -> Result<(), Err> {
        self.plugin.plugin_enable()
    }

    fn plugin_disable(&self) -> Result<(), Err> {
        self.plugin.plugin_disable()
    }

    fn register_events(&self, event_manager: RwLockWriteGuard<Box<dyn LakituEventManager>>) {
        self.plugin.register_events(event_manager)
    }
}

pub struct PluginRegistrar {
    plugins: HashMap<String, LakituPluginProxy>,
    lib: Rc<Library>,
}

impl PluginRegistrar {
    fn new(lib: Rc<Library>) -> PluginRegistrar {
        PluginRegistrar {
            lib, plugins: HashMap::default(),
        }
    }
}

impl LakituPluginRegistrar for PluginRegistrar {
    fn register_plugin(&mut self, plugin: Box<dyn LakituPlugin>) {
        let name = plugin.get_name();
        let proxy = LakituPluginProxy {
            plugin, _lib: Rc::clone(&self.lib),
        };

        self.plugins.insert(name.into(), proxy);
    }
}

pub struct LakituEvents {
    events: RwLock<Vec<dyn LakituEvent>>,
}

impl LakituEvents {
    pub fn new() -> Self {
        Self { events: RwLock::new(Vec::new()) }
    }
}

impl LakituEventManager for LakituEvents {
    fn get_events(&self) -> &RwLock<Vec<dyn LakituEvent>> {
        &self.events
    }

    fn register_event(&self, event: Box<dyn LakituEvent>) -> usize {
        let mut events = self.get_events().write()?;
        let index = events.len();

        self.get_events().write()?.push(event);
        index
    }

    fn register_event_handler(&self, index: usize, event_handler: Box<LakituEventHandlerID<T>>) {
        let mut events = self.get_events().write()?;
        let mut event = events.get(index).unwrap();

        event.add_handler(event_handler);
    }
}

pub struct LakituPlugins {
    plugins: HashMap<String, LakituPluginProxy>,
    libraries: Vec<Rc<Library>>,
}

impl LakituPlugins {
    pub fn new () -> LakituPlugins {
        LakituPlugins::default()
    }

    pub fn get_plugins(&self) -> &HashMap<String, LakituPluginProxy> {
        &self.plugins
    }

    /*
        Unsafe because this assumes the plugin was initiated with lakitu_lib::api::export_plugin
     */
    pub unsafe fn load<P: AsRef<OsStr>>(&mut self, library_path: P) -> io::Result<()> {
        let library = Rc::new(Library::new(library_path)?);

        let decl = library
            .get::<*mut LakituPluginDeclaration>(b"lakitu_plugin_declaration\0")?
            .read();

        if decl.rustc_version != lakitu_lib::RUSTC_VERSION
            || decl.lakitu_lib_version != lakitu_lib::LAKITU_LIB_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Plugin version mismatch (is everything up to date?)",
            ));
        }

        let mut registrar = PluginRegistrar::new(Rc::clone(&library));

        (decl.register)(&mut registrar);

        self.plugins.extend(registrar.plugins);
        self.libraries.push(library);

        Ok(())
    }
}