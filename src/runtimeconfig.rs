use std::path::Path;
extern crate stderrlog;
extern crate log;
	use log::*;


use std::env;

pub struct RuntimeConfig
{
	pub me: String,
	pub help: bool, // -h
	pub dry: bool, // -d
	pub verbose: bool, // -v
	pub soft: bool, // -s
	pub copy: bool, // -c
	pub kits: Vec<String>, // -k
	pub trunc: u8, // -t
	pub rules: String, // -r
	pub input_path: String,// -i
	pub output_path: String // -o
}

impl std::fmt::Debug for RuntimeConfig
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
	{
	    writeln!(f, "{{ [RUNTIMECONFIG] dry: {:?} verbose: {:?} soft: {:?} copy: {:?} kits: {:?} trunc: {:?} rules: {:?} input-path: {:?} output-path: {:?} }}",
    		self.dry,
    		self.verbose,
    		self.soft,
    		self.copy,
    		self.kits,
    		self.trunc,
    		self.rules,
    		self.input_path,
    		self.output_path
		)
	}
}

const T_INDICATOR: &str = "-";

const T_DRY: &str = "d";
const T_VERBOSE: &str = "v";
const T_SOFT: &str = "s";
const T_COPY: &str = "c";
const T_KITS: &str = "k";
const T_TRUNC: &str = "t";
const T_RULES: &str = "r";
const T_INPUT: &str = "i";
const T_OUTPUT: &str = "o";
const T_HELP: &str = "h";

fn setup_default_config() -> RuntimeConfig
{
	// DEFAULTS:
	RuntimeConfig {
		me: String::from(""),
		help: false,
		dry: false,
		verbose: false, 
		soft: false, 
		copy: false, 
		kits: vec![], 
		trunc: 0, // <-- 0 = no truncating
		rules: String::from(""),
		input_path: add_trailing_slash(&to_absolute_path(".")),
		output_path: String::from("*_remapped")
	}
}

fn add_trailing_slash(to: &str) -> String 
{
	let mut out = to.to_string();
	let last: char = *to.chars().rev().take(1).collect::<Vec<char>>().get(0).unwrap();

	if last != '/' && last != '\\'
	{
		out.push_str("/");
	}

	out.to_string()
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
	let mut out = process_args(args);

	out.output_path = add_trailing_slash(&target_name(&out.output_path, &out.input_path));
	out
}

fn target_name(output_path: &String, input_path: &str) -> String
{
	let source_path = Path::new(&input_path);
	let name = source_path.file_name().unwrap().to_string_lossy(); // Name of the working-directory
	let output_path_out = output_path.replace("*", &name); // * (if given) will be replaced with the name of the working-directory
	output_path_out.to_string()
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
		T_KITS => config.kits = read_buffer(&token, &buffer, 1),
		T_TRUNC => config.trunc = read_buffer(&token, &buffer, 1).get(0).unwrap().parse::<u8>().unwrap(),
		T_RULES => config.rules = String::from(read_buffer(&token, &buffer, 1).get(0).unwrap()),
		T_OUTPUT => config.output_path = String::from(read_buffer(&token, &buffer, 1).get(0).unwrap()),
		T_INPUT => config.input_path = add_trailing_slash(&to_absolute_path(read_buffer(&token, &buffer, 1).get(0).unwrap())),
		T_HELP => config.help = true,
		_ => { 
				warn!("Unknown token {:?}", token)
		}
	}
}

pub fn print_help()
{
	println!("KITCAT - Help");
	println!("");
	println!("OPTIONS");
	println!("");

	println!("-help / -h:");
	println!("\tPrint this help.");

	println!("");

	println!("-dry / -d:");
	println!("\tSetup and process all files without copying/linking them.");

	println!("");

	println!("-verbose / -v:");
	println!("\tPrint more information to the stdout while processing.");

	println!("");

	println!("-soft / -s:");
	println!("\tCreate soft-link instead of hard-link.");

	println!("");

	println!("-copy / -c:");
	println!("\tCopy files instead of linking (if set: -s won’t be used).");

	println!("");

	println!("-kits / -k:");
	println!("\tSpace-separated list of names. Just process this kits(s). The name check is done on the configured group-index and it's value.");

	println!("");

	println!("-trunc / -t:");
	println!("\tTruncate all kits containing less than „-t“ samples.");

	println!("");

	println!("-rules / -r:");
	println!("\tPath to a ini-file, overriding the internal ruleset.");

	println!("");

	println!("-input / -i:");
	println!("\tAll files inside this directory will be checked against the input-rule (without the leading input-directory-path; default is the directory of the kitcat-binary (.) )");

	println!("");

	println!("-output / -o:");
	println!("\tOutput-directory - files will be written into this directory (use an asterisk (*) to use the input-directories base-name; default is *_remapped)");
}