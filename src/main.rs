/**
  *  LakituBot
  *
  *  A performance focused, modular chat bot agnostic of service with Duppy API integration
  *
  */

#[macro_use]
extern crate lazy_static;
extern crate rand;

pub mod plugins;
mod ascii_logos;

use std::env;
use std::sync::RwLock;
use std::alloc::System;
use std::convert::TryInto;
use std::collections::HashMap;
use rand::{Rng, thread_rng};
use config::{File, Config, Environment};
use lakitu_lib::api::LakituPlugin;
use crate::plugins::{LakituPlugins, LakituPluginProxy, LakituEvents};

#[global_allocator]
static ALLOCATOR: System = System;

lazy_static! {
    static ref SETTINGS: RwLock<Config> = RwLock::new(Config::default());
    static ref EVENT_MANAGER: RwLock<Box<LakituEvents>> = RwLock::new(Box::new(LakituEvents::new()));
}

fn main() {
    let mut rng = thread_rng();

    let len: i32 = ascii_logos::LOGOS.len() as i32;
    let sel: usize = rng.gen_range(0..len).try_into().unwrap();

    let logo: &str = ascii_logos::LOGOS[sel];

    println!("{}", logo);

    println!("Platform Agnostic, Modular, High-Performance Chat Bot Written in Rust by LittleBigBug");
    println!("LakituBot starting..\n");

    SETTINGS.write()?
        .merge(File::with_name("Config")).unwrap()
        .merge(File::with_name("Config.local").required(false)).unwrap()
        .merge(Environment::with_prefix("APP")).unwrap();

    let mut plugin_manager = LakituPlugins::new();

    let ext: &str = match env::consts::OS {
        "windows" => "dll",
        _ => "so",
    };

    let files = glob(format!("./target/*.{}", ext)).expect("Failed to read glob pattern");

    for file in files {
        unsafe {
            plugin_manager.load(file.unwrap().display())
                .expect("Plugin loading failed");
        }
    }

    let plugins: &HashMap<String, LakituPluginProxy> = plugin_manager.get_plugins();

    for (name, plugin) in plugins {
        let version = plugin.get_version();
        let author = plugin.get_author();

        println!("Enabling Plugin {} v{} by {}", name, version, author);
        plugin.plugin_enable();

        // Allow plugins to register events
        let manager = EVENT_MANAGER.write()?;
        plugin.register_events(manager);
    }
}
