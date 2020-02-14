extern crate stderrlog;
extern crate log;
	use log::*;

use std::env;

pub struct RuntimeConfig
{
	pub me: String,
	pub dry: bool, // -d
	pub verbose: bool, // -v
	pub soft: bool, // -s
	pub copy: bool, // -c
	pub path: String,// -p
	pub kits: Vec<String>, // -k
	pub trunc: u8, // -t
	pub name: String, // -n
	pub rules: String // -r
}

impl std::fmt::Debug for RuntimeConfig
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
	{
	    writeln!(f, "{{ [RUNTIMECONFIG] dry: {:?} verbose: {:?} soft: {:?} copy: {:?} path: {:?} kits: {:?} trunc: {:?} name: {:?} rules: {:?} }}",
    		self.dry,
    		self.verbose,
    		self.soft,
    		self.copy,
    		self.path,
    		self.kits,
    		self.trunc,
    		self.name,
    		self.rules
		)
	}
}

const T_INDICATOR: &str = "-";

const T_DRY: &str = "d";
const T_VERBOSE: &str = "v";
const T_SOFT: &str = "s";
const T_COPY: &str = "c";
const T_PATH: &str = "p";
const T_KITS: &str = "k";
const T_TRUNC: &str = "t";
const T_NAME: &str = "n";
const T_RULES: &str = "r";

fn setup_default_config() -> RuntimeConfig
{
	// DEFAULTS:
	RuntimeConfig {
		me: String::from(""),
		dry: false,
		verbose: false, 
		soft: false, 
		copy: false, 
		path: to_absolute_path("."),
		kits: vec![], 
		trunc: 0, // <-- 0 = no truncating
		name: String::from("*_remapped"), 
		rules: String::from("")
	}
}

fn to_absolute_path(path: &str) -> String
{
	// also this will fail if the path does not exist
	let pb = std::path::PathBuf::from(path); 
	pb.canonicalize().unwrap().as_path().to_str().unwrap().to_string()
}


pub fn from_args() -> RuntimeConfig
{
	let args: Vec<String> = env::args().collect();

	process_args(args)
}

fn process_args(args: Vec<String>) -> RuntimeConfig 
{
	let mut out = setup_default_config();
	let mut buffer: Vec<String> = vec![];
	let mut proc_token = "";

	for (i, elem) in args.iter().enumerate()
	{
		if i == 0
		{
			out.me = String::from(elem);
			
			continue;
		}

		if elem.get(0..1).unwrap() == T_INDICATOR
		{	
			process_token(proc_token, &mut out, &buffer);

			if elem.len() < 2
			{
				warn!("Missing token name after indicator {:?} - skipping", T_INDICATOR);
				continue;
			}
		
			proc_token = elem.get(1..2).unwrap();
			buffer.clear();
		}
		else 
		{
			buffer.push(String::from(elem));
		}
	};

	process_token(proc_token, &mut out, &buffer); out
}

fn read_buffer(token: &str, buffer: &Vec<String>, ensure_amount: usize) -> Vec<String>
{
	if buffer.len() < ensure_amount
	{
		panic!("Too few arguments for parameter {:?} - got {:?} need at least {:?}", token, buffer.len(), ensure_amount)
	}

	buffer.to_vec()
}

fn process_token(token: &str, config: &mut RuntimeConfig, buffer: &Vec<String>)
{
	if token == ""
	{
		return;
	}

	match token
	{
		T_VERBOSE => config.verbose = true,
		T_DRY => config.dry = true,
		T_SOFT => config.soft = true,
		T_COPY => config.copy = true,
		T_PATH => config.path = to_absolute_path(read_buffer(&token, &buffer, 1).get(0).unwrap()),
		T_KITS => config.kits = read_buffer(&token, &buffer, 1),
		T_TRUNC => config.trunc = read_buffer(&token, &buffer, 1).get(0).unwrap().parse::<u8>().unwrap(),
		T_NAME => config.name = String::from(read_buffer(&token, &buffer, 1).get(0).unwrap()).replace("/", "").replace("\\", ""),
		T_RULES => config.rules = String::from(read_buffer(&token, &buffer, 1).get(0).unwrap()),
		_ => { 
				warn!("Unknown token {:?}", token)
		}
	}
}