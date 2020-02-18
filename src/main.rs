#[macro_use]
extern crate lazy_static;
extern crate stderrlog;
extern crate log;
	use log::*;

mod ruleset;
mod runtimeconfig;
mod catalog;
mod version;

use std::collections::*;
use crate::catalog::Kit;
use crate::runtimeconfig::RuntimeConfig;

fn main()
{
	let rc = runtimeconfig::from_args();
	init_logger(&rc);
	info!("Using {:?}", rc);

	if rc.help || rc.input.is_empty()
	{
		if rc.input.is_empty()
		{
			warn!("KitCat was started without any argument (at least -input is required) - help will be shown");
		}

		runtimeconfig::print_help();
		return;
	}

    let samples = catalog::collect(&rc.input);
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