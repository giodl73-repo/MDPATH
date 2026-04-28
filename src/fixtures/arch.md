# mdpath Architecture

## Resolution Pipeline

The resolver runs in three phases:

```
mdpath resolution pipeline

┌──────────────────────────────────────────────────────┐
│  Phase 1: Parse                                      │
│  md://languages/10-GO.md → read file → ParsedDoc    │
└──────────────┬───────────────────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────────────────┐
│  Phase 2: Navigate                                   │
│  heading-path → locate Section within ParsedDoc     │
└──────────────┬───────────────────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────────────────┐
│  Phase 3: Select                                     │
│  type + selector → find Element → apply sub-selectors│
└──────────────────────────────────────────────────────┘
```

## URI Components

The five addressing components stack from least to most specific:

```
uri-components

  md://path
      │
      └── #heading-path
              │
              └── :type.kind:
                      │
                      └── selector
                              │
                              └── [sub-selector]
```

## Module Map

```
module-map

┌─────────────────────────────────┐
│ mdpath public API               │
│ parse() · resolve()             │
│ BatchResolver                   │
├────────┬────────┬───────────────┤
│ uri    │ parser │ resolver      │
│ MdUri  │ doc    │ navigate      │
│ types  │ elems  │ select        │
├────────┴──┬─────┴───────────────┤
│ label     │ selector            │
│ matching  │ sub-selector        │
│           │ kind detection      │
└───────────┴─────────────────────┘
```

## Element Type Detection

```
type-detection

Input: fenced code block
  │
  ├── contains box-drawing chars?  → figure
  ├── starts with bar chart?       → chart.bar
  ├── starts with line plot?       → chart.line
  └── other content?               → text.code

Input: markdown pipe table
  │
  ├── col 1 = key, col 2 = value?  → table.key-value
  ├── col headers present?         → table.data
  └── comparison structure?        → table.comparison
```
