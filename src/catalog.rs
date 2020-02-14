extern crate stderrlog;
extern crate log;
	use log::*;

extern crate walkdir;
	use walkdir::WalkDir;

extern crate regex;
	use regex::*;

use std::result::Result::Err;
use std::path::Path;
use std::path::PathBuf;
use std::convert::TryInto;
use std::collections::*;
use crate::runtimeconfig::RuntimeConfig;
use crate::ruleset;
	use crate::ruleset::*;

pub struct Kit 
{
	pub name: String,
	pub samples: Vec<Sample>
}

impl std::fmt::Debug for Kit
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
	{
	    writeln!(f, "{{ [KIT] name: {:?} samples: {:?} }}",
    		self.name,
    		self.samples
		)
	}
}

pub struct Sample 
{
	pub source_path: String,
	pub target_path: String,
	pub fields: HashMap<String, String>
}

impl std::fmt::Debug for Sample
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
	{
	    writeln!(f, "{{ [SAMPLE] source_path: {:?} target_path: {:?} fields: {:?} }}",
    		self.source_path,
    		self.target_path,
    		self.fields
		)
	}
}

fn clone_sample(s: &Sample) -> Sample
{
	let mut out = Sample {
		source_path: String::from(&s.source_path),
		target_path: String::from(&s.target_path),
		fields: HashMap::<String,String>::new()
	};

	for (key, value) in s.fields.iter()
	{
		out.fields.insert(key.to_string(), value.to_string());
	}

	out
}

pub fn process_dataset(dataset: Vec<String>, rcon: &RuntimeConfig) -> HashMap<String, Kit>
{
	let mut out: HashMap<String, Kit> = HashMap::new();
	let ruleset: ruleset::Ruleset; 

	if rcon.rules != ""
	{
		info!("Using custom configuration-file {:?}", rcon.rules);

		ruleset = setup_custom_ruleset(&rcon.rules);

	}
	else 
	{
		info!("Setting up default-configuration");

		ruleset = setup_default_ruleset();
	}

	info!("Using Ruleset {:?}", ruleset);

	let input_rule = Regex::new(&ruleset.input).unwrap();
	let _output_rule = &ruleset.output;// just a string
	let recheck_rule = Regex::new(&ruleset.recheck).unwrap();// compile here, not in loop

	info!("Applying input-rule {:?} on {:?}", input_rule, rcon.path);
	info!("Found {:?} paths - processing...", dataset.len());

	for path in dataset
	{
		debug!("Processing {:?}...", path);

		if input_rule.is_match(&path)
		{
			let cap = input_rule.captures(&path).unwrap();
				debug!("Captured {:?}", cap);
			
			let sample = &process_capture(cap, &rcon, &ruleset, &recheck_rule);
				debug!("Created sample {:?}", sample);

			let index_value = sample.fields.get(&ruleset.index).unwrap();

			if out.contains_key(index_value)
			{
				out.get_mut(index_value).unwrap().samples.push(clone_sample(&sample));
			}
			else 
			{
				let new_kit = Kit {
					name: String::from(index_value),
					samples: vec![clone_sample(&sample)]
				};

				debug!("Created Kit {:?}", new_kit);

				out.insert(String::from(index_value), new_kit);
			}
		}
		else 
		{
			warn!("{:?} does not match on {:?}", path, input_rule);
		}
	}
	
	out
}

pub fn process_capture(cap: Captures, rcon: &RuntimeConfig, ruleset: &Ruleset, recheck_rule: &Regex) -> Sample
{
	let mut _path = rcon.path.to_string();
	let last: char = *_path.chars().rev().take(1).collect::<Vec<char>>().get(0).unwrap();

	if last == '/' || last == '\\'
	{
		debug!("Trimming trailing slash of {:?}", _path);
		_path = _path.get(0..(_path.len() - 1)).unwrap().to_string();
	}

	let mut _target_pre = rcon.name.replace("*", &_path);
	_target_pre.push_str(&["/", &ruleset.output].join(""));

	_path.push_str(&["/", &cap[0]].join(""));

	debug!("Mapping {:?} by order {:?}", cap, ruleset.input_order);

	let mut matched_groups = map_capture(&cap, &ruleset.input_order);
	
	for (group, replace_str) in ruleset.rearranges.iter()
	{
		let recheck_val = match matched_groups.get(group) 
		{
			None => panic!("Failed on reading recheck value with given group name {:?} out of matching group {:?} - this group name is not defined in groups section of {:?}", group, matched_groups, rcon.rules),
			Some(d) => d
		};

		if recheck_rule.is_match(recheck_val)
		{
			let mut new_value = String::from(replace_str);

			apply_output_rule(&mut new_value, &matched_groups);
			
			let (r_group, _r_value) = matched_groups.remove_entry(group).unwrap();

			info!("Recheck-rule matched on field {:?} - created new value by {:?} -> {:?}", group, replace_str, new_value);

			matched_groups.insert(r_group.to_string(), new_value.to_string());
		}
	}

	ruleset::apply_output_rule(&mut _target_pre, &matched_groups);

	Sample {
		source_path: _path,
		target_path: _target_pre,
		fields: matched_groups
	}
}

pub fn apply_filters<'a,'b>(processed_dataset: &'a mut HashMap<String, Kit>, rcon: &'b RuntimeConfig) -> &'a mut HashMap<String, Kit>
{
	if rcon.trunc > 0
	{
		info!("Applying truncate-filter - every kit less than {:?} samples will be truncated", rcon.trunc);

		processed_dataset.retain(|_name, kit|
		{
			let retain = kit.samples.len() >= rcon.trunc.try_into().unwrap();

			if !retain 
			{
				info!("Truncating {:?}", kit);
			}
			
			retain
		});
	}

	if rcon.kits.len() > 0
	{
		info!("Applying kits-filter - every kit not in {:?} will be truncated", rcon.kits);

		processed_dataset.retain(|name, kit|
		{
			let retain = { rcon.kits.contains(name) };

			if !retain
			{
				info!("Truncating {:?}", kit);
			}

			retain
		});
	}

	processed_dataset
}

pub fn write_dataset(processed_dataset: &HashMap<String, Kit>, rcon: &RuntimeConfig)
{
	if rcon.dry 
	{
		info!("Not writing {:?} kits", processed_dataset.len());	
	}
	else 
	{
		info!("Writing {:?} kits", processed_dataset.len());
	}

	let mut total_samples_failed = 0;
	let mut total_samples_written = 0;

	for (_name, kit) in processed_dataset.iter()
	{
		let current_samples = &kit.samples;

		for sample in current_samples.into_iter()
		{
			if rcon.dry
			{
				continue;
			}

			let filepath = PathBuf::from(&sample.target_path);
			let path = filepath.with_file_name("");

			if !path.exists()
			{
				info!("Path {:?} does not exist - trying to create", path);

				match std::fs::create_dir_all(&path)
				{
					Err(e) => { error!("Creating path {:?} failed: {:?} - skipping sample {:?}", path, e, sample); total_samples_failed+=1; continue },
					Ok(_) => {}
				}
			}

			if rcon.copy 
			{
				match copy_sample(sample)
				{
					true => total_samples_written+=1,
					false => total_samples_failed+=1
				}
			}
			else
			{
				match link_sample(sample, rcon.soft)
				{
					true => total_samples_written+=1,
					false => total_samples_failed+=1
				}
			}
		}
	}

	info!("Wrote {:?} samples, {:?} failed, {:?} total", total_samples_written, total_samples_failed, total_samples_failed+total_samples_written)
}

fn copy_sample(sample: &Sample) -> bool
{
	debug!("Copying {:?} to {:?}", sample.source_path, sample.target_path);

	let source = Path::new(&sample.source_path);
	let target = &Path::new(&sample.target_path);

	match std::fs::copy(source, target)
	{
		Err(e) => { error!("Creating softlink for {:?} failed: {:?}", sample, e); return false },
		Ok(_) => { return true }
	}
}

fn link_sample(sample: &Sample, soft: bool) -> bool
{
	let source = Path::new(&sample.source_path);
	let target = &Path::new(&sample.target_path);

	if soft 
	{
		debug!("Linking (soft) {:?} to {:?}", sample.source_path, sample.target_path);

		#[cfg(target_family = "windows")]
			match std::os::windows::fs::symlink_file(source, target)
			{
				Err(e) => { error!("Creating softlink for {:?} failed: {:?}", sample, e); return false },
				Ok(_) => return true
			}
		
		#[cfg(target_family = "unix")]
			match std::os::unix::fs::symlink(source, target)
			{
				Err(e) => { error!("Creating softlink for {:?} failed: {:?}", sample, e); return false },
				Ok(_) => return true
			}
	}
	else 
	{
		debug!("Linking (hard) {:?} to {:?}", sample.source_path, sample.target_path);
		
		match std::fs::hard_link(source, target)
		{
			Err(e) => { error!("Creating hardlink for {:?} failed: {:?}", sample, e); return false },
			Ok(_) => return true
		}
	}
}

pub fn collect(from: &str) -> Vec<String>
{
	info!("Collecting samples from {:?}", from);
	let mut samples: Vec<String> = vec![];

	for e in WalkDir::new(&from).into_iter().filter_map(|e| e.ok()) 
	{
        if e.metadata().unwrap().is_file() 
        {
            samples.push(String::from(e.path().strip_prefix(&from).unwrap().to_str().unwrap()));
        }
    }

    info!("Found {:?} samples", samples.len());

    samples
}