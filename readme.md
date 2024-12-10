# Prettypst


## Formatter for Typst!

|                 Before                  |            Default Style             |           OTBS Style           |
| :-------------------------------------: | :----------------------------------: | :----------------------------: |
| ![Before formatting](images/before.png) | ![Default style](images/default.png) | ![OTBS style](images/otbs.png) |


## Install

	cargo install --git=https://github.com/antonWetzel/prettypst.git --locked

## Usage / Features

- Prettypst might change your work, use only if you can recover a previous version.
- Insert or remove trailing comma to change between single line and multiline
- Any feature requests are welcome

### Terminal

	prettypst [--help]                          | Print help
	prettypst <file>                            | Format file inplace
	prettypst --use-std-in ...                  | Use terminal as input
	prettypst ... -o <output-file>              | Change output file
	prettypst ... --use-std-out                 | Use terminal as output
	prettypst --style=[default | otbs] ...      | Change the used style
	prettypst --use-configuration ...           | Load a configuration from "prettypst.toml"
	prettypst --file-location=<path> ...        | Change file location to search for configuration (use with --use-std-in)

### VSCodium or VSCode

- install custom-local-formatters
- use prettypst for typst
	- ```json
		"customLocalFormatters.formatters": [
			{
				"command": "prettypst --use-std-in --use-std-out --file-location=${file}",
				"languages": [
					"typst",
				],
			}
		],
   		```
- change command to use wanted style or configuration
	- `--style=otbs`
	- `--use-configuration`
		- `prettypst.toml` must be in the same folder as the formatted file or a parent folder

### Helix

- Add `prettypst` to your `languages.toml`
- ```toml
  [[language]]
  name = "typst"
  # ...
  formatter = { command = "prettypst", args = ["--use-std-in", "--use-std-out"] }
  ```
- change command to use wanted style or configuration
	- `--style=otbs`
	- `--use-configuration`
		- `prettypst.toml` must be in the current working directory or a parent folder

## Settings

- create configuration with `prettypst [--style=otbs] --save-configuration`

```gdscript
indentation = 0 | 1 | ... # use 0 for tabs
separate-label = false | true # insert space before a label
final-newline = false | true # insert a final newline at the end

[preserve-newline]
content = false | true # preserve single newlines in markup
math = false | true # preserve single newline in equations

[block]
long-block-style = "separate" | "compact" # '[' and ']' on the same or separate lines as the content

[term]
space-before = false | true # insert a space before the ':' in terms
space-after = false | true # insert a space after the ':' in terms

[named-argument]
space-before = false | true # insert a space before the ':' in named arguments
space-after = false | true # insert a space after the ':' in named arguments

[dictionary-entry]
space-before = false | true # insert a space before the ':' in dictionary entries
space-after = false | true # insert a space after the ':' in dictionary entries

[import-statement]
space-before = false | true # insert a space before the ':' in import statements
space-after = false | true # insert a space after the ':' in import statements

[comma]
space-before = false | true # insert a space before the ',' in arguments
space-after = false | true # insert a space after the ',' in arguments


[columns]
comma = "end-of-content" | "end-of-cell" # how to align commas in tables

[heading]
blank-lines-before = 0 | 1 | ... # blank lines before a heading
blank-lines-after = 0 | 1 | ... # blank lines after a heading

[columns-commands]
grid = "columns"     # format the "grid", "gridx", ... command with
gridx = "columns"    # columns specified by the named argument "columns"
table = "columns"
tablex = "columns"
```