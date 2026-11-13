//! OSC 8 hyperlinks: text that is a link to a URI, which the terminal opens when it is clicked.
//!
//! A program marks text as a link by writing `ESC ] 8 ; params ; URI ST` before it and
//! `ESC ] 8 ; ; ST` after it, where ST is `ESC \` or BEL. `params` is empty or `key=value` pairs
//! separated by `:`; the one key defined is `id`. The URI is printable ASCII (bytes 32 to 126),
//! as the OSC 8 specification requires: other characters are percent-encoded.
//!
//! BoxMux keeps a link with every cell of its text, in a box's content, in a choice's label and
//! in a terminal box, and writes it to the terminal around the cells that carry it
//! ([`crate::runtime::display`]). A terminal treats cells with the same URI and id as one link,
//! so a link's text that BoxMux wraps or cuts onto several lines stays one link. A link written
//! without an id is given one made from its URI ([`Link::new`]), so each of its lines carries
//! the same id. An OSC 8 sequence that does not have this form is dropped: it changes no link
//! and is never shown as text.

use std::sync::Arc;

/// Ends the link the terminal is in.
pub const CLOSE: &str = "\x1b]8;;\x1b\\";

/// A hyperlink: where it goes and the id that makes its cells one link.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Link {
    uri: String,
    id: String,
}

/// What an OSC 8 sequence does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkChange {
    /// The text that follows is this link.
    Open(Arc<Link>),
    /// The text that follows is no link.
    Close,
}

impl Link {
    /// The link to `uri` with `id`, or, when `id` is `None`, with an id made from the URI: every
    /// link to the same URI without an id of its own gets the same one.
    pub fn new(uri: &str, id: Option<&str>) -> Link {
        let id = match id {
            Some(id) => id.to_string(),
            None => format!("boxmux-{:016x}", fnv1a(uri.as_bytes())),
        };
        Link {
            uri: uri.to_string(),
            id,
        }
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    /// The sequence that starts this link: `ESC ] 8 ; id=<id> ; <uri> ESC \`.
    pub fn open_sequence(&self) -> String {
        format!("\x1b]8;id={};{}\x1b\\", self.id, self.uri)
    }
}

/// What the OSC sequence `body` (the text between `ESC ]` and its terminator) does to the link
/// text is in; `None` when it is not OSC 8, or is a malformed one.
pub fn parse_osc(body: &str) -> Option<LinkChange> {
    let rest = body.strip_prefix("8;")?;
    let Some((params, uri)) = rest.split_once(';') else {
        log::debug!("dropped an OSC 8 sequence without a URI field: {:?}", body);
        return None;
    };
    parse_osc8(params, uri)
}

/// What OSC 8 with `params` and `uri` does: an empty URI ends the link; otherwise the params
/// must be `key=value` pairs separated by `:` and both must be printable ASCII. `None` when
/// they are malformed.
pub fn parse_osc8(params: &str, uri: &str) -> Option<LinkChange> {
    if uri.is_empty() {
        return Some(LinkChange::Close);
    }
    let printable = |text: &str| text.bytes().all(|b| (0x20..=0x7e).contains(&b));
    if !printable(uri) || !printable(params) {
        log::debug!(
            "dropped an OSC 8 link with a character outside printable ASCII: {:?} {:?}",
            params,
            uri
        );
        return None;
    }
    let mut id = None;
    if !params.is_empty() {
        for pair in params.split(':') {
            let Some((key, value)) = pair.split_once('=') else {
                log::debug!(
                    "dropped an OSC 8 link whose parameter {:?} is not key=value",
                    pair
                );
                return None;
            };
            if key == "id" && !value.is_empty() {
                id = Some(value);
            }
        }
    }
    Some(LinkChange::Open(Arc::new(Link::new(uri, id))))
}

/// The 64-bit FNV-1a hash of `bytes`: the same on every run and platform, so a link's made-up
/// id stays the same from one frame to the next.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opened(body: &str) -> Link {
        match parse_osc(body) {
            Some(LinkChange::Open(link)) => (*link).clone(),
            other => panic!("{:?} opens no link: {:?}", body, other),
        }
    }

    #[test]
    fn a_link_keeps_its_uri_and_id() {
        let link = opened("8;id=docs;https://example.com/a?b=c;d");
        assert_eq!(link.uri(), "https://example.com/a?b=c;d");
        assert_eq!(link.id(), "docs");
        assert_eq!(
            link.open_sequence(),
            "\x1b]8;id=docs;https://example.com/a?b=c;d\x1b\\"
        );
    }

    #[test]
    fn other_parameters_are_accepted_and_only_the_id_is_kept() {
        let link = opened("8;foo=bar:id=x;https://example.com");
        assert_eq!(link.id(), "x");
    }

    #[test]
    fn a_link_without_an_id_gets_one_made_from_its_uri() {
        let a = opened("8;;https://example.com/a");
        assert!(a.id().starts_with("boxmux-"), "{}", a.id());
        assert_eq!(a, opened("8;;https://example.com/a"), "the same every time");
        assert_eq!(
            a,
            opened("8;id=;https://example.com/a"),
            "an empty id is none"
        );
        assert_ne!(a.id(), opened("8;;https://example.com/b").id());
    }

    #[test]
    fn an_empty_uri_ends_the_link() {
        assert_eq!(parse_osc("8;;"), Some(LinkChange::Close));
        assert_eq!(parse_osc("8;id=x;"), Some(LinkChange::Close));
    }

    #[test]
    fn malformed_links_and_other_commands_change_nothing() {
        for body in [
            "8",
            "8;https://example.com",
            "8;oops;https://example.com",
            "8;id=x:bare;https://example.com",
            "8;;https://example.com/日本",
            "8;;https://example.com/\u{7}",
            "8;;https://example.com/\u{9b}",
            "0;a title",
            "52;c;aGk=",
        ] {
            assert_eq!(parse_osc(body), None, "{:?}", body);
        }
    }
}
