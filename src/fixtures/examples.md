# md:// URI Examples

## Basic addressing

Whole file:

```
md://computing/01-PACKAGE.md
```

Specific section:

```
md://computing/01-PACKAGE.md#the-big-picture
```

Subsection (parent/child):

```
md://computing/01-PACKAGE.md#the-big-picture/layer-1-language-runtime
```

## Element addressing

First figure in file:

```
md://computing/01-PACKAGE.md:figure:0
```

Named figure:

```
md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler
```

Named table row:

```
md://languages/10-GO.md#concurrency-model:table:0[row=Goroutine]
```

Named table cell:

```
md://languages/10-GO.md#concurrency-model:table:0[row=Goroutine,col=Stack Size]
```

Named figure box:

```
md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler[box=SCHEDULER]
```

## Query parameters

Select specific columns from a table:

```
md://data/metrics.md:table:stats?select=name,value
```

Filter rows:

```
md://data/metrics.md:table:stats?filter=status=passing
```

Count elements:

```
md://data/metrics.md:table:stats?count
```

Top N results:

```
md://data/metrics.md:table:stats?top=5
```

## Addressing the same element multiple ways

All four URIs below refer to the same element (named preferred):

```
md://languages/10-GO.md:figure:goroutine-scheduler     (named — most stable)
md://languages/10-GO.md:figure:goroutine               (prefix match)
md://languages/10-GO.md:figure:scheduler               (substring match)
md://languages/10-GO.md:figure:0                       (index — least stable)
```
