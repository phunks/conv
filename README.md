# Conv
converter tool.

![Screenshot](.github/images/window.png)

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