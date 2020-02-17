![logo](https://repository-images.githubusercontent.com/240513735/fed78c00-4f48-11ea-87c0-1d82d3cd57fa)

Command-line tool designed to catalog audio sample libraries.

> This project was made to gain first experiences in the rust programming-language.
	
# KitCat
---

## Example

### This command

	kitcat -i /path/to/My Sample Library/Drums -o /path/to/My Sample Library/Kits

### Transforms this directory-structure

	- My Sample Library
 		- Drums
 			- Kick
 				- Kick Kit_A.wav
				- Kick Kit_B.wav
				- Kick Kit_C.wav
				- ...

 			- Hat
 				- Hat Kit_A.wav
				- Hat Kit_B.wav
				- Hat Kit_C.wav
				- ...
				
 			- Snare
 				- Snare Kit_A.wav
 				- Snare Kit_B.wav
 				- Rim Kit_A.wav
 				- Roll Kit_B.wav
				- ...

 			- ...

 		- ...

### To this

	- My Sample Library
		- Kits
			- Kit_A
				- Kick.wav
				- Hat.wav
				- Snare.wav
				- Rim.wav
				- ...

			- Kit_B
				- Kick.wav
				- Snare.wav
				- Roll.wav
				- ...
				 	
			- Kit_C
				- Kick.wav
				- Hat.wav
				- ...
		- ...
			

## Runtime configuration
Arguments that can be passed to the kitcat-binary. All arguments must be prefixed with a dash "-" followed by the parameter(-short)-name.

|parameter|short|description|
|:-------|:---:|:----------|
| input   | i   | input-directory - all files inside this directory will be checked against the input-rule (without the leading input-directory-path; default is the directory of the kitcat-binary (.) ) . |
| output 	| o  | output-directory - files will be written into this directory (use an asterisk (*) to use the input-directories base-name; default is \*_remapped) |
| soft 	| s 	 | Create soft-link instead of hard-link |
| copy 	| c 	 | Copy files instead of linking (if set: -s won’t be used) |
| truncate | t | truncate all kits containing less than „-t“ samples |
| kits 	| k 	 | Space-separated list of names. Just process this kits(s). The name check is done by the configured group-index. |
| verbose | v | Print more information to the stdout while processing |
| dry 	| d  | no files will be written at all |
| rules | r | Path to a ini-file, overriding the internal ruleset (explained in "custom ruleset definition" section below) |
| help | h | Print list of possible arguments |

## Custom ruleset definition
Add the lines below to an ini-file of your choice and pass it's path to the "-r" argument explained above. KitCat will use the rules defined inside this file instead of using the internal default-ruleset. Check the examples-directory for further explanation of the configuration file.

	input = "{group}/{sample} ?{kit}{variation}?\.{extension}"
	output = "{kit}/{sample} {variation}.{extension}"
	recheck = "^([[0-9a-zA-Z]{1,2}])$"
	index = kit
	
	[groups]
	group = "([a-zA-Z0-9 ]*)"
	sample = "([a-zA-Z0-9]*)"
	kit = "([a-zA-Z0-9]*)"
	variation = "([a-zA-Z0-9 ]*)"
	extension = "([(wav|WAV|mp3|MP3)]*)"
	
	[rearrange]
	kit = "{sample} - {group}/{kit}_"