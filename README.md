# Conv
converter tool.

![Screenshot](.github/images/window.png)

## Features

- **Base64** — Base64, Base64URL, and deflated SAML conversion
- **Binary** — hexadecimal, byte-string, and character-code conversion
- **Escape** — URL, JavaScript, HTML entity, and UTF-7 escaping
- **Crypt** — MD5, SHA, and AES encryption/decryption
- **Regex** — regular-expression replacement and grep-like filtering
- **Data** — JSON, TOML, YAML, CSV, and XML conversion with jq filters
- **Formatting** — JSON, TOML, YAML, XML, HTML, CSS, JavaScript, TypeScript, and SQL formatting
- **Diff** — structural diff for source code
- **Spreadsheet** — CSV viewing, filtering, sorting, and pivot tables

## CSV and spreadsheets

Conv provides lightweight CSV conversion and spreadsheet operations such as filtering,
sorting, and pivot tables.

### Pivot tables

Use **Pivot…** in the Spreadsheet view to drag CSV fields into **Rows**,
**Columns**, and **Values**, then select an aggregation.

The current pivot-table support is intentionally lightweight:

- **Rows:** one or more fields
- **Columns:** exactly one field
- **Values:** exactly one field
- **Aggregations:** `Count`, `Sum`, `Average`, `Minimum`, and `Maximum`
- `Count` counts non-empty, non-null Values only.
- `Sum`, `Average`, `Minimum`, and `Maximum` require finite numeric values.
  Empty and null values are ignored; numeric input may use commas such as `1,234.5`.
- Pivot output is limited to **512 columns**. Avoid using high-cardinality fields,
  such as IDs or timestamps, in Columns.
- Spreadsheet display filters are not currently applied to pivot calculations.
- Pivot results can be copied back to the input as CSV, and **Reset pivot** restores
  the table that was present before pivoting.

Pivot tables do not currently support multiple Values fields, multiple aggregations per
field, subtotals, grand totals, calculated fields, custom null-value display, or date
grouping.

For SQL queries and more advanced data exploration, use **DataGrip + DuckDB**.


## JSON ↔ XML conversion

JSON has no native representation for XML attributes or text nodes. The Data converter
uses the following conventions when converting between JSON and XML:

| XML construct | JSON representation |
| --- | --- |
| Attribute | `"@name": "value"` |
| Text content | `"#text": "value"` |
| CDATA section | `"#cdata": "value"` |
| Repeated child elements | An array |

For example, the following JSON:

```json
{
  "root": {
    "book": {
      "category": {
        "@cover": "paperback",
        "#text": "web"
      },
      "title": {
        "@lang": "en",
        "#text": "Learning XML"
      },
      "author": "Erik T. Ray",
      "year": 2003,
      "price": 39.95
    }
  }
}
```

is converted to:

```xml
<root>
  <book>
    <category cover="paperback">web</category>
    <title lang="en">Learning XML</title>
    <author>Erik T. Ray</author>
    <year>2003</year>
    <price>39.95</price>
  </book>
</root>
```

JSON-to-XML conversion requires an object containing exactly one root element.

## ⚠️ macOS

The macOS release archives are not notarized by Apple. After downloading and extracting
an archive from GitHub Releases, macOS may report that `conv` is damaged or cannot be opened.

Only if you downloaded the archive from this repository's GitHub Releases, remove the
download quarantine attribute and then open the application:

```bash
xattr -dr com.apple.quarantine conv.app
open conv.app
```

Alternatively, after macOS blocks the application, open **System Settings → Privacy & Security**
and select **Open Anyway**.

To inspect the quarantine attributes for troubleshooting:

```bash
xattr -lr conv.app
```

## License
The source code is licensed MIT. The website content is licensed CC BY 4.0,see LICENSE.