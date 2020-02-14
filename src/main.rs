extern crate stderrlog;
extern crate log;
	use log::*;

mod ruleset;
mod runtimeconfig;
mod catalog;

use std::collections::*;
use crate::catalog::Kit;
use crate::runtimeconfig::RuntimeConfig;

fn main()
{
	let rc = runtimeconfig::from_args();
	init_logger(&rc);
	info!("Using {:?}", rc);

    let samples = catalog::collect(&rc.path);
    let mut kits: HashMap<String, Kit> = catalog::process_dataset(samples, &rc);

    catalog::apply_filters(&mut kits, &rc);
    catalog::write_dataset(&kits, &rc);
}

fn init_logger(rc: &RuntimeConfig)
{
	let verbosity = if rc.verbose { 2 } else { 0 } ;
	
	stderrlog::new()
    .module(module_path!())
    .verbosity(verbosity)
	.timestamp(stderrlog::Timestamp::Off)
    .init()
    .unwrap();
}