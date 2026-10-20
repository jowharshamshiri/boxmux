//! Saving box positions back to the configuration file.
//!
//! Only the `position` of the moved or resized box changes; the rest of the file (comments,
//! ordering, quoting, formatting) is left byte for byte as it was. The edit is verified by
//! parsing the result before anything is written, and the file is replaced atomically.

use crate::model::bounds::InputBounds;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Replaces the `position` of box `box_id` in the configuration text. Fails, leaving nothing
/// changed, when the box or its `position` cannot be found unambiguously.
pub fn set_box_position(
    text: &str,
    box_id: &str,
    position: &InputBounds,
) -> Result<String, String> {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let mut found: Option<String> = None;
    for (index, line) in lines.iter().enumerate() {
        let Some(key_column) = id_key_column(line, box_id) else {
            continue;
        };
        let Some(edited) = edit_position(&lines, index, key_column, position) else {
            continue;
        };
        if found.is_some() {
            return Err(format!(
                "box '{}' appears more than once in the file; its position was not saved",
                box_id
            ));
        }
        found = Some(edited);
    }
    let edited = found.ok_or_else(|| {
        format!(
            "cannot find the position of box '{}' in the file; it was not saved",
            box_id
        )
    })?;
    verify(text, &edited, box_id, position)?;
    Ok(edited)
}

/// The column of the `id` key when `line` is `id: <box_id>` (optionally as the first entry of
/// a sequence item, quoted, or followed by a comment).
fn id_key_column(line: &str, box_id: &str) -> Option<usize> {
    let trimmed_end = line.trim_end_matches(['\n', '\r']);
    let content = trimmed_end.trim_start();
    let mut column = trimmed_end.len() - content.len();
    let mut rest = content;
    if let Some(after_dash) = rest.strip_prefix("- ") {
        let after = after_dash.trim_start();
        column += rest.len() - after.len();
        rest = after;
    }
    let value = rest.strip_prefix("id:")?.trim();
    let value = match value.find(" #") {
        Some(comment) => value[..comment].trim_end(),
        None => value,
    };
    let unquoted = value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
        .unwrap_or(value);
    (unquoted == box_id).then_some(column)
}

/// The indentation of a line, or `None` for blank and comment-only lines.
fn indentation(line: &str) -> Option<usize> {
    let content = line.trim_start();
    if content.trim().is_empty() || content.starts_with('#') {
        None
    } else {
        Some(line.len() - content.len())
    }
}

/// Rewrites the `position` entry of the mapping whose `id` key sits at `key_column` on line
/// `id_line`. Returns `None` when that mapping has no `position` key (it is not a box).
fn edit_position(
    lines: &[&str],
    id_line: usize,
    key_column: usize,
    position: &InputBounds,
) -> Option<String> {
    // The mapping's lines: from the item start to the first line indented less than its keys
    // (or a new sequence item at the same level).
    let start = (0..=id_line)
        .rev()
        .find(|&i| i == id_line || indentation(lines[i]).is_some_and(|indent| indent < key_column))
        .map(|i| if i == id_line { id_line } else { i + 1 })
        .unwrap_or(0);
    let item_start = if start > 0
        && lines[start - 1].trim_start().starts_with("- ")
        && indentation(lines[start - 1]).is_some_and(|indent| indent < key_column)
    {
        start - 1
    } else {
        start
    };
    let end = (id_line + 1..lines.len())
        .find(|&i| indentation(lines[i]).is_some_and(|indent| indent < key_column))
        .unwrap_or(lines.len());

    let key_line = (item_start..end).find(|&i| {
        let line = lines[i];
        let content_start = line.len() - line.trim_start().len();
        let content = line.trim_start();
        let (column, rest) = match content.strip_prefix("- ") {
            Some(after) => (
                content_start + 2 + (after.len() - after.trim_start().len()),
                after.trim_start(),
            ),
            None => (content_start, content),
        };
        column == key_column && rest.starts_with("position:")
    })?;

    let key_text = lines[key_line];
    let value_start = key_text.find("position:")? + "position:".len();
    let value = key_text[value_start..].trim_end_matches(['\n', '\r']);
    let newline = &key_text[key_text.trim_end_matches(['\n', '\r']).len()..];
    let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();

    if value.trim_start().starts_with('{') {
        let comment = trailing_comment(value);
        out[key_line] = format!(
            "{} {{x1: \"{}\", y1: \"{}\", x2: \"{}\", y2: \"{}\"}}{}{}",
            &key_text[..value_start],
            position.x1,
            position.y1,
            position.x2,
            position.y2,
            comment,
            newline
        );
        return Some(out.concat());
    }

    // Block mapping: the four coordinates are on the following, deeper-indented lines.
    let mut replaced = 0;
    for i in key_line + 1..end {
        let Some(indent) = indentation(lines[i]) else {
            continue;
        };
        if indent <= key_column {
            break;
        }
        let line = lines[i];
        let content = line.trim_start();
        for (name, value) in [
            ("x1", position.x1),
            ("y1", position.y1),
            ("x2", position.x2),
            ("y2", position.y2),
        ] {
            if let Some(rest) = content.strip_prefix(name).and_then(|r| r.strip_prefix(':')) {
                let comment = trailing_comment(rest.trim_end_matches(['\n', '\r']));
                let line_end = &line[line.trim_end_matches(['\n', '\r']).len()..];
                out[i] = format!(
                    "{}{}: \"{}\"{}{}",
                    &line[..indent],
                    name,
                    value,
                    comment,
                    line_end
                );
                replaced += 1;
            }
        }
    }
    (replaced == 4).then(|| out.concat())
}

/// The comment at the end of a value, with the whitespace that separates it from the value:
/// `"50%   # half"` gives `"   # half"`.
fn trailing_comment(value: &str) -> &str {
    let Some(hash) = value
        .char_indices()
        .find(|&(i, c)| c == '#' && value[..i].ends_with([' ', '\t']))
        .map(|(i, _)| i)
    else {
        return "";
    };
    let start = value[..hash].trim_end_matches([' ', '\t']).len();
    &value[start..]
}

/// Checks that the edited text parses and differs from the original only in the box's
/// position.
fn verify(
    original: &str,
    edited: &str,
    box_id: &str,
    position: &InputBounds,
) -> Result<(), String> {
    let mut before: serde_yaml::Value = serde_yaml::from_str(original)
        .map_err(|e| format!("the configuration file no longer parses: {}", e))?;
    let after: serde_yaml::Value = serde_yaml::from_str(edited).map_err(|e| {
        format!(
            "saving the position of box '{}' would break the file: {}",
            box_id, e
        )
    })?;
    let expected = serde_yaml::to_value(position).map_err(|e| e.to_string())?;
    if !set_position_in_value(&mut before, box_id, expected) {
        return Err(format!("box '{}' is not in the configuration file", box_id));
    }
    if before != after {
        return Err(format!(
            "saving the position of box '{}' would change more than its position; it was not saved",
            box_id
        ));
    }
    Ok(())
}

fn set_position_in_value(
    node: &mut serde_yaml::Value,
    box_id: &str,
    position: serde_yaml::Value,
) -> bool {
    match node {
        serde_yaml::Value::Mapping(mapping) => {
            let is_box = mapping.get("id").and_then(|v| v.as_str()) == Some(box_id)
                && mapping.contains_key("position");
            if is_box {
                mapping.insert("position".into(), position);
                return true;
            }
            mapping
                .iter_mut()
                .any(|(_, value)| set_position_in_value(value, box_id, position.clone()))
        }
        serde_yaml::Value::Sequence(items) => items
            .iter_mut()
            .any(|item| set_position_in_value(item, box_id, position.clone())),
        _ => false,
    }
}

/// Replaces `path` with `contents` atomically: the new text is written to a temporary file in
/// the same directory, flushed to disk, given the original's permissions, then renamed over it.
pub fn write_atomically(path: &Path, contents: &str) -> Result<(), String> {
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .ok_or_else(|| format!("'{}' is not a file path", path.display()))?
        .to_string_lossy();
    let temporary = directory.join(format!(".{}.boxmux-{}.tmp", name, std::process::id()));
    let permissions = std::fs::metadata(path)
        .map_err(|e| format!("cannot read '{}': {}", path.display(), e))?
        .permissions();
    let result = (|| -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        std::fs::set_permissions(&temporary, permissions)?;
        std::fs::rename(&temporary, path)
    })();
    if let Err(e) = result {
        let _ = std::fs::remove_file(&temporary);
        return Err(format!("cannot write '{}': {}", path.display(), e));
    }
    Ok(())
}

/// Saves a new position for `box_id` into the configuration file at `path`.
pub fn save_box_position(path: &Path, box_id: &str, position: &InputBounds) -> Result<(), String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read '{}': {}", path.display(), e))?;
    let edited = set_box_position(&text, box_id, position)?;
    write_atomically(path, &edited)
}

/// An exclusive claim on a configuration file, so two boxmux instances never write the same
/// file. Released (and its lock file removed) when dropped or when the process ends.
pub struct ConfigLock {
    path: PathBuf,
    _file: File,
}

impl ConfigLock {
    pub fn acquire(config: &Path) -> Result<ConfigLock, String> {
        let path = PathBuf::from(format!("{}.lock", config.display()));
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(|e| format!("cannot create '{}': {}", path.display(), e))?;
        match file.try_lock() {
            Ok(()) => Ok(ConfigLock { path, _file: file }),
            Err(std::fs::TryLockError::WouldBlock) => Err(format!(
                "'{}' is open in another boxmux instance; start this one with --lock to open it read-only",
                config.display()
            )),
            Err(std::fs::TryLockError::Error(e)) => {
                Err(format!("cannot lock '{}': {}", path.display(), e))
            }
        }
    }
}

impl Drop for ConfigLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x1: &str, y1: &str, x2: &str, y2: &str) -> InputBounds {
        InputBounds::parse(x1, y1, x2, y2).unwrap()
    }

    const BLOCK: &str = "\
# dashboard
app:
  layouts:
    - id: main   # the main layout
      children:
        - id: logs
          title: Logs  # keep me
          position:
            x1: '0%'
            y1: \"0%\"
            x2: 50%   # half
            y2: 100%
          content: hi
        - id: other
          position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
";

    #[test]
    fn block_position_is_rewritten_and_everything_else_kept() {
        let edited = set_box_position(BLOCK, "logs", &pos("10%", "5%", "60%", "90%")).unwrap();
        assert!(edited.contains("# dashboard"));
        assert!(edited.contains("title: Logs  # keep me"));
        assert!(edited.contains("x1: \"10%\"\n"));
        assert!(edited.contains("x2: \"60%\"   # half\n"));
        assert!(edited.contains("position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}"));
        let changed: Vec<&str> = BLOCK
            .lines()
            .zip(edited.lines())
            .filter(|(a, b)| a != b)
            .map(|(a, _)| a)
            .collect();
        assert_eq!(changed.len(), 4, "{:?}", changed);
    }

    #[test]
    fn flow_position_is_rewritten() {
        let edited = set_box_position(BLOCK, "other", &pos("40%", "0%", "100%", "50%")).unwrap();
        assert!(edited.contains("position: {x1: \"40%\", y1: \"0%\", x2: \"100%\", y2: \"50%\"}\n"));
        assert!(edited.contains("x1: '0%'"));
    }

    #[test]
    fn layouts_and_choices_with_the_same_id_are_not_touched() {
        let text = "\
app:
  layouts:
    - id: logs
      children:
        - id: menu
          position: {x1: 0%, y1: 0%, x2: 10%, y2: 10%}
          choices:
            - id: logs
              content: x
        - id: logs_box
          position: {x1: 0%, y1: 0%, x2: 10%, y2: 10%}
";
        assert!(set_box_position(text, "logs", &pos("1", "1", "2", "2")).is_err());
        let edited = set_box_position(text, "menu", &pos("1", "1", "2", "2")).unwrap();
        assert!(edited.contains("- id: logs\n      children"));
    }

    #[test]
    fn a_missing_box_is_an_error() {
        assert!(set_box_position(BLOCK, "nope", &pos("1", "1", "2", "2")).is_err());
    }

    #[test]
    fn atomic_write_keeps_permissions_and_leaves_no_temporary_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.yaml");
        std::fs::write(&path, BLOCK).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
        }
        save_box_position(&path, "logs", &pos("1%", "2%", "3%", "4%")).unwrap();
        assert!(std::fs::read_to_string(&path)
            .unwrap()
            .contains("x1: \"1%\""));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o640);
        }
        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(leftovers, vec![std::ffi::OsString::from("app.yaml")]);
    }

    #[test]
    fn a_second_lock_on_the_same_file_is_refused_until_the_first_is_released() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.yaml");
        std::fs::write(&path, "app: {}").unwrap();
        let first = ConfigLock::acquire(&path).unwrap();
        assert!(ConfigLock::acquire(&path).is_err());
        drop(first);
        assert!(ConfigLock::acquire(&path).is_ok());
    }
}
