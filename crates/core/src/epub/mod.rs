pub mod parser;

pub use parser::{EpubParser, ParsedChapter, ParsedEpub, ParsedMetadata, TocEntry};

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::domain::book::Genre;
    use crate::error::CoreError;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    pub(crate) fn write_sample_epub(path: &std::path::Path) {
        let file = std::fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let stored = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

        zip.start_file("mimetype", stored).unwrap();
        zip.write_all(b"application/epub+zip").unwrap();

        let deflated = SimpleFileOptions::default();
        zip.start_file("META-INF/container.xml", deflated).unwrap();
        zip.write_all(
            br#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
        )
        .unwrap();

        zip.start_file("OEBPS/content.opf", deflated).unwrap();
        zip.write_all(
            r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" unique-identifier="bookid" version="2.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Ficciones</dc:title>
    <dc:creator>Jorge Luis Borges</dc:creator>
    <dc:language>es</dc:language>
    <dc:subject>Science Fiction</dc:subject>
    <dc:description>Cuentos fantásticos.</dc:description>
    <dc:identifier id="bookid">urn:uuid:12345</dc:identifier>
    <meta name="cover" content="cover-img"/>
  </metadata>
  <manifest>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
    <item id="cover-img" href="cover.png" media-type="image/png"/>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine toc="ncx">
    <itemref idref="ch1"/>
  </spine>
</package>"#.as_bytes(),
        )
        .unwrap();

        zip.start_file("OEBPS/toc.ncx", deflated).unwrap();
        zip.write_all(
            r#"<?xml version="1.0" encoding="utf-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <navMap>
    <navPoint id="nav1" playOrder="1">
      <navLabel><text>Tlön, Uqbar, Orbis Tertius</text></navLabel>
      <content src="ch1.xhtml"/>
    </navPoint>
  </navMap>
</ncx>"#.as_bytes(),
        )
        .unwrap();

        zip.start_file("OEBPS/cover.png", stored).unwrap();
        zip.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
            .unwrap();

        zip.start_file("OEBPS/ch1.xhtml", deflated).unwrap();
        zip.write_all(
            r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
  <head><title>Capítulo 1</title></head>
  <body><p>Debo a la conjunción de un espejo y de una enciclopedia el descubrimiento de Uqbar.</p></body>
</html>"#.as_bytes(),
        )
        .unwrap();

        zip.finish().unwrap();
    }

    #[test]
    fn test_parse_missing_file_returns_not_found() {
        let res = EpubParser::parse_file("/tmp/non_existent_book_12345.epub");
        assert!(matches!(res, Err(CoreError::NotFound(_))));
    }

    #[test]
    fn test_parse_real_epub_file() {
        let epub_path = std::env::temp_dir().join("test_ficciones_parser.epub");
        write_sample_epub(&epub_path);

        let parsed = EpubParser::parse_file(&epub_path).expect("debe parsear archivo epub real");
        assert_eq!(parsed.metadata.title, "Ficciones");
        assert_eq!(parsed.metadata.author, "Jorge Luis Borges");
        assert_eq!(parsed.metadata.language, "es");
        assert_eq!(parsed.metadata.genre, Genre::SciFi);
        assert_eq!(parsed.chapters.len(), 1);
        assert_eq!(parsed.chapters[0].title, "Tlön, Uqbar, Orbis Tertius");
        assert!(parsed.chapters[0].raw_xhtml.contains("Uqbar"));
        assert!(parsed.cover.is_some());
        assert_eq!(parsed.cover.as_ref().unwrap().1, "png");
        assert_eq!(parsed.toc.len(), 1);

        let _ = std::fs::remove_file(&epub_path);
    }
}
