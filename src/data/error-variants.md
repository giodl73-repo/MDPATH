# md:// Error Variants

| variant | cause | resolution |
|---------|-------|------------|
| FileNotFound | The path component does not point to an existing file | Check that the path is relative to the proof root, not the current directory |
| SectionNotFound | The heading-path segment does not match any heading in the file | Verify the heading text is normalized (lowercase, spaces→dashes) |
| SectionAmbiguous | Multiple headings normalize to the same segment | Use a longer heading path (parent/child) to disambiguate |
| ElementNotFound | No element of the requested type exists at the selector | Check the element type, kind, and whether the file contains that element |
| LabelNotFound | A named selector does not match any element label | Verify the label — try substring matching or use a numeric index |
| LabelAmbiguous | Multiple elements match the named selector | Use a more specific label or switch to numeric index |
| SubSelectorInvalid | A sub-selector key is not valid for this element type | Check supported sub-selectors for the element type (row/col for tables, box for figures) |
| InvalidUri | URI syntax is malformed | Validate the URI against the grammar |
| ParseError | The target file could not be parsed as markdown | Check for encoding issues or file corruption |
