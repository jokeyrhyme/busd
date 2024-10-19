use anyhow::{Error, Result};
use quick_xml::{events::Event, Reader};

const EXPECTED_DOCTYPE_PARTS: &[&str] = &[
    "busconfig",
    "PUBLIC",
    r#""-//freedesktop//DTD"#,
    "D-Bus",
    "Bus",
    "Configuration",
    r#"1.0//EN""#,
    r#""http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd""#,
];

/// implements [`dbus-daemon`'s Configuration File](https://dbus.freedesktop.org/doc/dbus-daemon.1.html#configuration_file)
#[derive(Clone, Debug, Default)]
pub struct BusConfig {}

impl BusConfig {
    pub fn parse(s: &str) -> Result<Self> {
        let mut reader = Reader::from_reader(s.as_bytes());
        let mut has_dtd = false;
        loop {
            match reader.read_event()? {
                Event::DocType(bytes_text) => {
                    has_dtd = true;
                    let content = bytes_text.unescape()?;
                    // this approach does through away extra whitespace within quoted strings,
                    // which means we do allow through some unexpected DOCTYPEs
                    // TODO: consider whether added complexity is worth 100% strictness for DOCTYPEs
                    let dtd_parts: Vec<&str> = content.as_ref().split_whitespace().collect();
                    if dtd_parts != EXPECTED_DOCTYPE_PARTS {
                        assert_eq!(dtd_parts, EXPECTED_DOCTYPE_PARTS);
                        return Err(Error::msg(
                            "incorrect/incomplete `<!DOCTYPE ...> declaration`",
                        ));
                    }
                    // we currently don't bother with the case of multiple DTDs
                }
                Event::Start(bytes_start) => {
                    if !has_dtd {
                        return Err(Error::msg(
                            "must specify `<!DOCTYPE ...>` before root element",
                        ));
                    }
                    if bytes_start.name().as_ref() != b"busconfig" {
                        return Err(Error::msg("root element must be `<busconfig>`"));
                    }
                    break;
                }
                Event::Eof => break,
                _ => {}
            }
        }
        Ok(Self {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn bus_config_from_str_without_dtd_error() {
        let input = r#"<busconfig></busconfig>"#;
        BusConfig::parse(input).expect("should parse XML input");
    }

    #[test]
    #[should_panic]
    fn bus_config_parse_with_unexpected_dtd_error() {
        let input = r#"<!DOCTYPE busconfig PUBLIC
        "-//W3C//DTD XHTML Basic 1.1//EN"
        "http://www.w3.org/TR/xhtml-basic/xhtml-basic11.dtd">
        <busconfig></busconfig>
        "#;
        BusConfig::parse(input).expect("should parse XML input");
    }

    #[test]
    #[should_panic]
    fn bus_config_parse_with_unexpected_root_element_error() {
        let input = r#"<!DOCTYPE foo PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN"
        "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
        <foo></foo>
        "#;
        BusConfig::parse(input).expect("should parse XML input");
    }

    #[test]
    fn bus_config_parse_with_dtd_and_root_element_ok() {
        let input = r#"<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN"
        "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
        <busconfig></busconfig>
        "#;
        BusConfig::parse(input).expect("should parse XML input");
    }
}
