# md:// URI Components

| component | required | syntax | example | description |
|-----------|----------|--------|---------|-------------|
| scheme | yes | `md://` | `md://` | Always `md://` — identifies the URI scheme |
| path | yes | file path relative to root | `languages/10-GO.md` | Path to the markdown file from the proof root |
| heading-path | no | `#segment[/segment]*` | `#concurrency-model` | Section within the file — normalized heading text |
| type | no | `:[type:]` | `:figure:` | Element type: figure, table, chart, text, heading |
| kind | no | `type.kind` | `figure.flowchart` | Sub-type of the element |
| selector | no | string or integer | `goroutine-scheduler` or `2` | Named label (preferred) or numeric index (fallback) |
| sub-selector | no | `[key=value]` | `[row=Binding]` | Target within an element: row, col, box, bar |
| query | no | `?key=value` | `?select=name,value` | Post-resolution filtering or projection |
