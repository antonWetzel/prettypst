# Changelog

## 2024.12.11

- **BREAKING** | Correct spelling mistake (seperate $\to$ separate)

## 2024.10.20

- Update dependencies

## 2024.09.29

- **Fix** | Higher priority for labels to fix labels for headings
- Update tests

## 2024.07.29

- **ADD** | Only format label in Markup 
- Update tests

## 2024.06.24

- **ADD** | Better formatting for `(grid|table).(cell|header|footer)`
- Update tests

## 2024.05.21

- **CHANGE** | Ignore trailing comma only in **arrays** with one item
- Add some tests
- Update **typst-syntax**

## 2023.12.06 | Update 1.1

- **BREAKING** | change settings to `kabab-case`
- **ADD** |  setting for methods, which arguments are formatted as columns
	- add `[columns-commands]` section
	- use `<method-name> = "<argument-name>`
	- ```toml
		[columns-commands]
		grid = "columns"
		gridx = "columns"
		table = "columns"
		tablex = "columns"
		```
	- `<argument-name>`, is the name of the named argument, where the column count is specified
- **ADD** | setting for padding around `,` in arguments
	- ```toml
		[comma]
		space-before = false
		space-after = true
		```

## 2023.11.21 | Release
