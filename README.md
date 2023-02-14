# smbios-dumper
WIP SMBIOS data viewing tool for windows

## Usage

`smbios-dumper.exe` - prints out the SmbiosProcessorInfo and SmbiosBoardInfo tables

`smbios-dumper.exe dump` - dumps SMBIOS to `smbios_dump.bin` which you can then open in an hex editor
## Todo
* output file args
* string support (it currently does not parse the strings in the table)
* json export
* structs for the other tables, make the code not terrible
