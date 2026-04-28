# md:// Element Types and Kinds

| type | kinds | detected-by | name-source | example-uri |
|------|-------|-------------|-------------|-------------|
| figure | flowchart, box, sequence, timeline, graph | Fenced block with box-drawing characters | First text-only line inside fence | `:figure.flowchart:arch-overview` |
| table | key-value, schema, comparison, data | Markdown pipe table | First column cell (row names) or header | `:table:0[row=Binding]` |
| chart | bar, line, scatter, stacked, pie | ASCII bar chart or plot | Bar label text | `:chart.bar:0[bar=Option A]` |
| math | — | fence_info: math/latex/tex or rendered fraction bars | First line label | `:math:0` |
| tree | ascii, org, taxonomy | fence_info: tree/org/taxonomy or └─── patterns | First line label | `:tree:0` |
| slide | — | fence_info: slide or proof:slide | Title line | `:slide:0` |
| dashboard | — | fence_info: dashboard or proof:region | Name attribute | `:dashboard:0` |
| text | prose, list, code | Prose paragraph, list, fenced code block | First N words of content | `:text:0` |
| heading | h1, h2, h3 | Markdown heading lines | The heading text | `:heading:overview` |
| section | — | Heading + all content until next same-level heading | Normalized heading text | `#concurrency-model` |
