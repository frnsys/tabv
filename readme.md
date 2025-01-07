# tabv

A simple CSV viewer.

```bash
# Open a single file.
tabv path/to/some.csv

# Open multiple files.
tabv path/to/csv/dir
```

One particular feature is multi-sheet CSVs. Basically multiple CSVs can be placed into a file, with each sheet/table preceded by a line starting with `#>` and then a name for the sheet. For example:

```csv
#>Sheet 1
a,b,c
a,b,c
#>Sheet 2
a,b,c
a,b,c
```
