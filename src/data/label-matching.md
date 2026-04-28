# md:// Label Matching Algorithm

| priority | strategy | example-label | example-query | matches |
|----------|----------|---------------|---------------|---------|
| 1 | Exact match | goroutine-scheduler | goroutine-scheduler | yes |
| 2 | Starts-with prefix | goroutine-scheduler | goroutine | yes |
| 3 | Substring | goroutine-scheduler | scheduler | yes |
| 4 | Numeric index | (any label) | 0 | first element |
| 5 | Ambiguous — error | goroutine-scheduler, goroutine-pool | goroutine | LabelAmbiguous |
