extern crate lazy_static;

extern crate stderrlog;
extern crate log;
	use log::*;

extern crate regex;
	use regex::*;

use ini::Ini;
use ini::ini::Properties;

use std::collections::*;

pub struct Ruleset
{
	pub groups: HashMap<String, String>,
	pub rearranges: HashMap<String, String>,

	pub recheck: String,
	pub input: String,
	pub output: String,
	pub index: String,
	pub input_order: HashMap<String, usize>
}

const PH_GROUP: &str = r"group";
const PH_SAMPLE: &str = r"sample";
const PH_KIT: &str = r"kit";
const PH_VARIATION: &str = r"variation";
const PH_EXTENSION: &str = r"extension";

const DEF_RULE_GROUP: &str = r"([a-zA-Z0-9 ]*)";
const DEF_RULE_SAMPLE: &str = r"([a-zA-Z0-9]*)";
const DEF_RULE_KIT: &str = r"([a-zA-Z0-9]*)";
const DEF_RULE_VARIATION: &str = r"([a-zA-Z0-9 ]*)";
const DEF_RULE_EXTENSION: &str = r"([wav|WAV|mp3|MP3]*)";
const DEF_RULE_INDEX: &str = PH_KIT;
const DEF_RULE_RECHECK: &str = r"^([0-9a-zA-Z]{1,2})$";

// Helpers to keep the path clean: 
const DEF_RULE_TRIMMER: &str = r"[ ]?([\|/|.])[ ]?";
const DEF_RULE_TRIMMER_TO: &str = r"$1";

lazy_static! {
	static ref TRIMMER_REGEX: Regex = Regex::new(DEF_RULE_TRIMMER).unwrap();
}

impl std::fmt::Debug for Ruleset
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
	{
	    writeln!(f, "{{ [RULES] input: {:?} output: {:?} input_order: {:?} recheck: {:?} index: {:?} groups: {:?} }}",
    		self.input,
    		self.output,
    		self.input_order,
    		self.recheck,
    		self.index,
    		self.groups
		)
	}
}

pub fn get_group_order(raw_input_str: &str, groups: &HashMap<String, String>) -> HashMap<String, usize>
{
	let mut check: HashMap<String, usize> = HashMap::new();
	let mut out: HashMap<String, usize> = HashMap::new();
	let mut i = 0;

	for (group_name, _group_regex) in groups.iter()
	{
		match raw_input_str.find(&with_brackets(group_name)) 
		{
			None => continue,
			Some(i) => { check.insert(group_name.to_string(), i); }
		}
	}
	
	if check.len() == 0
	{
		panic!("Input string does not contain any matching groups - cannot proceed");
	}

	let mut check_: Vec<(String, usize)> = check.into_iter().collect();
	check_.sort_by(|a, b| { a.1.cmp(&b.1) });

	for (group, _c) in check_.iter()
	{
		i+=1;
		out.insert(group.to_string(), i);
	}

	out
}

pub fn apply_output_rule(to_str: &mut String, replacements: &HashMap<String, String>)
{
	for (group, replacement) in replacements.iter()
	{
		*to_str = to_str.trim().replace(&with_brackets(group), replacement.trim());
	}

	*to_str = TRIMMER_REGEX.replace_all(to_str, DEF_RULE_TRIMMER_TO).to_string();
}

pub fn apply_input_groups(to_str: &mut String, replacements: &HashMap<String, String>)
{
	for (group, replacement) in replacements.iter()
	{
		*to_str = to_str.replace(&with_brackets(group), replacement);
	}
}

pub fn setup_default_ruleset() -> Ruleset
{
	let raw_input_str = &[&with_brackets(PH_GROUP), "/", &with_brackets(PH_SAMPLE), " ?", &with_brackets(PH_KIT), &with_brackets(PH_VARIATION), r"?\.", &with_brackets(PH_EXTENSION)].join("");
	let raw_output_str = &[&with_brackets(PH_KIT), "/", &with_brackets(PH_SAMPLE), " ", &with_brackets(PH_VARIATION), ".", &with_brackets(PH_EXTENSION)].join("");
	let recheck_rule = DEF_RULE_RECHECK;
	let index_rule = DEF_RULE_INDEX;
	let mut groups: HashMap<String, String> = HashMap::new();

	groups.insert(PH_GROUP.to_string(), DEF_RULE_GROUP.to_string());
	groups.insert(PH_VARIATION.to_string(), DEF_RULE_VARIATION.to_string());
	groups.insert(PH_KIT.to_string(), DEF_RULE_KIT.to_string());
	groups.insert(PH_SAMPLE.to_string(), DEF_RULE_SAMPLE.to_string());
	groups.insert(PH_EXTENSION.to_string(), DEF_RULE_EXTENSION.to_string());

	let mut rearranges: HashMap<String, String> = HashMap::new();

	rearranges.insert(PH_SAMPLE.to_string(), PH_KIT.to_string());

	let mut out = Ruleset {
		input: raw_input_str.to_string(),
		output: raw_output_str.to_string(),
		input_order: get_group_order(&raw_input_str, &groups),
		index: index_rule.to_string(),
		rearranges: rearranges,
		recheck: recheck_rule.to_string(),
		groups: groups
	};

	apply_input_groups(&mut out.input, &out.groups);
	out
}

pub fn setup_custom_ruleset(by_file: &str) -> Ruleset
{
	let conf = Ini::load_from_file(by_file).unwrap();
	let io = conf.general_section();

	let s_groups: &Properties = match conf.section(Some("groups"))
	{
		None => panic!("Missing groups section in {:?}", by_file),
		Some(s) => s
	};

	let outter: Properties; // expands lifetime in None-arm below to this scope
	let s_rearranges: &Properties = match conf.section(Some("rearrange"))
	{
		None => { warn!("Missing rearrange section in {:?}", by_file); outter = Properties::new(); &outter },
		Some(s) => s
	};

	let raw_input_str: String = match io.get("input")
	{
		None => panic!("Missing input configuration in {:?}", by_file),
		Some(s) => s.to_string()
	};

	let raw_output_str: String = match io.get("output")
	{
		None => panic!("Missing output configuration in {:?}", by_file),
		Some(s) => s.to_string()
	};
	
	let recheck_rule: String = match io.get("recheck")
	{
		None => { warn!("Missing recheck configuration in {:?}", by_file); "".to_string() },
		Some(s) => s.to_string()
	};

	let index_rule: String = match io.get("index")
	{
		None => panic!("Missing index configuration in {:?}", by_file),
		Some(s) => s.to_string()
	};

	let mut groups: HashMap<String, String> = HashMap::new();

	for (name, regex) in s_groups.iter() 
	{
		groups.insert(name.to_string(), regex.to_string());
	}

	if !groups.contains_key(&index_rule)
	{
		panic!("Index-group {:?} does not exist in groups section of {:?}", index_rule, by_file);
	}

	let mut rearranges: HashMap<String, String> = HashMap::new();

	for (name, regex) in s_rearranges.iter()
	{
		rearranges.insert(name.to_string(), regex.to_string());
	}

	let mut out = Ruleset {
		input: raw_input_str.to_string(),
		output: raw_output_str,
		rearranges: rearranges,
		recheck: recheck_rule,
		input_order: get_group_order(&raw_input_str, &groups),
		index: index_rule,
		groups: groups,
	};

	apply_input_groups(&mut out.input, &out.groups);
	out
}

pub fn map_capture(cap: &Captures, input_order: &HashMap<String,usize>) -> HashMap<String, String>
{
	let mut out: HashMap<String, String> = HashMap::new();

	for (group, index) in input_order.iter()
	{
		out.insert(group.to_string(), cap[*index].trim().to_string());
	}

	out
}

pub fn with_brackets(group_name: &str) -> String
{
	let mut out = String::new();
	out.push_str("{"); 
	out.push_str(group_name);
	out.push_str("}"); 
	out
}