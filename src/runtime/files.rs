//! File boxes: a box's `file` shown in its file tab and read again when the platform's file
//! watching (inotify, FSEvents, kqueue, ReadDirectoryChangesW) reports a change, never by polling.
//!
//! A box watches the folder its file is in, which tells it when the file is created, removed or
//! replaced, and the file itself while it exists, which tells it when the file is written on
//! every platform (a kqueue folder watch says nothing of its files' contents) and follows a file
//! that is a symbolic link. While the folder does not exist either, the box watches the deepest
//! folder of the path that does and moves down as the missing folders are made. The changes
//! reported between two frames are read once, before the next frame is drawn.

use super::event::Event;
use super::Runtime;
use crate::model::stream::StreamStatus;
use crate::runtime::display::Display;
use crossbeam_channel::Sender;
use notify::event::{EventKind, ModifyKind};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// How many times arming a box's watches starts again when the folders of its path change
/// while it is arming them, before it gives up and says so.
const ARM_ATTEMPTS: usize = 16;

/// Every file box of the app and the watches they hold.
pub(super) struct FileBoxes {
    sender: Sender<Event>,
    /// Relative `file` paths are relative to this folder: the configuration file's.
    base_dir: PathBuf,
    /// Made when the first file box needs it; the error says why it could not be.
    watcher: Option<Result<RecommendedWatcher, String>>,
    /// How many of the boxes' needs each watched path serves, folders and files alike.
    watched: HashMap<PathBuf, usize>,
    boxes: HashMap<String, FileBox>,
}

/// One file box: what it shows and what it watches.
struct FileBox {
    /// The path as the configuration gives it.
    file: String,
    /// The configured path, made absolute.
    path: PathBuf,
    follow: bool,
    /// The deepest existing folder of `path`, as watched.
    folder: Option<PathBuf>,
    /// `path` under the watched `folder`: the path events name it by.
    watch_path: PathBuf,
    /// The file the box watches itself, when it exists: `watch_path`, with every symbolic link
    /// resolved where the platform reports events by the resolved path.
    file_watch: Option<PathBuf>,
    /// What changed since the box was last read.
    stale: Option<Change>,
    /// Whether the tab shows the file's text, read since the box was made or since the file
    /// was last missing or unreadable. The first read of a followed file shows its end.
    read_once: bool,
    /// For a followed file: how many bytes of it the tab shows, and the bytes after its last
    /// newline, shown as the tab's last line until the rest of their line is written.
    shown: u64,
    partial: Vec<u8>,
}

/// What a change reported for a box asks of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Change {
    /// The file was written: read it (a followed file, from where the tab ends).
    Written,
    /// The file or a folder of its path was created, removed, renamed or replaced: watch the
    /// path again and read the file whole.
    Moved,
}

/// What reading a file box found.
enum Reading {
    /// The file's whole text, as the tab's lines.
    Whole(Vec<String>),
    /// Lines to add to the end of the tab; `replaces_last` when the tab's last line was a line
    /// still being written, which they now show whole.
    Appended {
        lines: Vec<String>,
        replaces_last: bool,
    },
    Missing,
    Failed(String),
}

impl FileBoxes {
    pub(super) fn new(sender: Sender<Event>, base_dir: PathBuf) -> Self {
        FileBoxes {
            sender,
            base_dir,
            watcher: None,
            watched: HashMap::new(),
            boxes: HashMap::new(),
        }
    }

    /// The absolute path a box's `file` names.
    fn resolve(&self, file: &str) -> PathBuf {
        let path = Path::new(file);
        let joined = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_dir.join(path)
        };
        std::path::absolute(&joined).unwrap_or(joined)
    }

    /// Notes what a watch event says changed, for the boxes it concerns.
    pub(super) fn note(&mut self, event: notify::Result<notify::Event>) {
        let event = match event {
            Ok(event) => event,
            Err(e) => {
                // A watch that failed may have missed changes: every box looks again.
                log::warn!("watching files: {}", e);
                self.mark_all(Change::Moved);
                return;
            }
        };
        if event.need_rescan() || event.paths.is_empty() {
            self.mark_all(Change::Moved);
            return;
        }
        // Reading a file is an access, and the platform reports it; nothing it reports for
        // an access or a change of metadata changes what a box shows.
        if matches!(
            event.kind,
            EventKind::Access(_) | EventKind::Modify(ModifyKind::Metadata(_))
        ) {
            return;
        }
        let written = matches!(
            event.kind,
            EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Any | ModifyKind::Other)
        );
        for file_box in self.boxes.values_mut() {
            for path in &event.paths {
                let change =
                    if path == &file_box.watch_path || file_box.file_watch.as_ref() == Some(path) {
                        if written {
                            Change::Written
                        } else {
                            Change::Moved
                        }
                    } else if file_box.watch_path.starts_with(path) {
                        // A folder of the path: it changed, appeared or went away.
                        Change::Moved
                    } else {
                        continue;
                    };
                file_box.stale = file_box.stale.max(Some(change));
            }
        }
    }

    fn mark_all(&mut self, change: Change) {
        for file_box in self.boxes.values_mut() {
            file_box.stale = file_box.stale.max(Some(change));
        }
    }

    /// Whether no box is registered and nothing is watched.
    #[cfg(test)]
    pub(super) fn is_idle(&self) -> bool {
        self.boxes.is_empty() && self.watched.is_empty()
    }

    pub(super) fn has_stale(&self) -> bool {
        self.boxes.values().any(|file_box| file_box.stale.is_some())
    }

    /// The watcher, made the first time a box needs one.
    fn watcher(&mut self) -> Result<&mut RecommendedWatcher, String> {
        if self.watcher.is_none() {
            let sender = self.sender.clone();
            self.watcher = Some(
                notify::recommended_watcher(move |event| {
                    // The runtime is gone when this fails; there is nobody left to tell.
                    let _ = sender.send(Event::Files(event));
                })
                .map_err(|e| format!("Cannot watch files for changes: {}", e)),
            );
        }
        match self.watcher.as_mut().expect("made above") {
            Ok(watcher) => Ok(watcher),
            Err(e) => Err(e.clone()),
        }
    }

    /// Adds one need of a watch on `path`. A path already watched is watched anew, so the watch
    /// is of what the path names now: a file replaced since keeps no watch on the file it was.
    fn hold(&mut self, path: &Path) -> Result<(), String> {
        let count = self.watched.get(path).copied().unwrap_or(0);
        let watcher = self.watcher()?;
        if count > 0 {
            unwatch(watcher, path);
        }
        if let Err(e) = watcher.watch(path, RecursiveMode::NonRecursive) {
            if count > 0 {
                self.watched.remove(path);
            }
            return Err(e.to_string());
        }
        self.watched.insert(path.to_path_buf(), count + 1);
        Ok(())
    }

    /// Drops one need of the watch on `path`, ending the watch with its last need.
    fn release(&mut self, path: &Path) {
        let Some(count) = self.watched.get_mut(path) else {
            return;
        };
        *count -= 1;
        if *count == 0 {
            self.watched.remove(path);
            if let Some(Ok(watcher)) = self.watcher.as_mut() {
                unwatch(watcher, path);
            }
        }
    }

    /// Watches what box `box_id` needs watched now: the deepest existing folder of its path,
    /// and its file when the file exists.
    fn arm(&mut self, box_id: &str) -> Result<(), String> {
        for _ in 0..ARM_ATTEMPTS {
            let path = self.boxes[box_id].path.clone();
            let Some(folder) = path.ancestors().skip(1).find(|folder| folder.is_dir()) else {
                return Err("No folder of this path exists".to_string());
            };
            let watched_folder = real_folder(folder);
            let below = path
                .strip_prefix(folder)
                .expect("a folder is a prefix of the path it is a folder of");
            let watch_path = watched_folder.join(below);
            // Watched anew even when it is the folder watched before: one removed and made again
            // since is another folder, which the old watch does not see.
            match self.hold(&watched_folder) {
                Ok(()) => {}
                // It went away after it was found: look again.
                Err(_) if !folder.is_dir() => continue,
                Err(e) => {
                    return Err(format!(
                        "Cannot watch {} for changes: {}",
                        watched_folder.display(),
                        e
                    ))
                }
            }
            let file_box = self.boxes.get_mut(box_id).expect("the box is registered");
            let old_folder = file_box.folder.replace(watched_folder);
            file_box.watch_path = watch_path.clone();
            if let Some(old) = old_folder {
                self.release(&old);
            }
            // A folder of the path made while the one above it was being watched sent no event
            // to this box: watch the deepest one now.
            if path
                .ancestors()
                .skip(1)
                .take_while(|ancestor| *ancestor != folder)
                .any(Path::is_dir)
            {
                continue;
            }
            if let Some(old) = self
                .boxes
                .get_mut(box_id)
                .expect("the box is registered")
                .file_watch
                .take()
            {
                self.release(&old);
            }
            if watch_path.is_file() {
                let file_watch = real_file(&watch_path);
                match self.hold(&file_watch) {
                    Ok(()) => {
                        self.boxes
                            .get_mut(box_id)
                            .expect("the box is registered")
                            .file_watch = Some(file_watch);
                    }
                    // Removed after it was found: the folder's watch tells of it.
                    Err(_) if !watch_path.exists() => {}
                    Err(e) => return Err(format!("Cannot watch this file for changes: {}", e)),
                }
            }
            return Ok(());
        }
        Err("The folders of this path kept changing while they were being watched".to_string())
    }

    /// Drops box `box_id` and every watch only it needed.
    pub(super) fn forget(&mut self, box_id: &str) {
        if let Some(file_box) = self.boxes.remove(box_id) {
            for path in file_box.folder.iter().chain(file_box.file_watch.iter()) {
                self.release(path);
            }
        }
    }
}

/// Ends the watch on `path`. A watch the platform has already ended, because what it watched
/// was removed, is no longer there to end.
fn unwatch(watcher: &mut RecommendedWatcher, path: &Path) {
    if let Err(e) = watcher.unwatch(path) {
        if !matches!(
            e.kind,
            notify::ErrorKind::WatchNotFound | notify::ErrorKind::PathNotFound
        ) {
            log::warn!("cannot stop watching {}: {}", path.display(), e);
        }
    }
}

/// The path a folder's events name it by: with its symbolic links resolved on Unix, where
/// FSEvents reports resolved paths (a temporary folder under /var is /private/var).
fn real_folder(folder: &Path) -> PathBuf {
    #[cfg(unix)]
    {
        std::fs::canonicalize(folder).unwrap_or_else(|_| folder.to_path_buf())
    }
    #[cfg(not(unix))]
    {
        folder.to_path_buf()
    }
}

/// The path a watch on the file at `path` reports its events by. FSEvents watches the file a
/// symbolic link names and reports it by its own path; the other platforms report the path
/// they were given.
fn real_file(path: &Path) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }
    #[cfg(not(target_os = "macos"))]
    {
        path.to_path_buf()
    }
}

/// The lines of text that ends at a line's end.
fn lines_of(bytes: &[u8]) -> Vec<String> {
    crate::text::sanitize_output(&String::from_utf8_lossy(bytes))
}

/// Splits `bytes` after its last newline: the whole lines, and what follows them.
fn split_at_line_end(bytes: &[u8]) -> (&[u8], &[u8]) {
    match bytes.iter().rposition(|byte| *byte == b'\n') {
        Some(end) => bytes.split_at(end + 1),
        None => (&[], bytes),
    }
}

impl FileBox {
    /// Reads the file: whole, unless it is followed and was only written to since the last
    /// read, when what was added to its end is read. A followed file that is now shorter than
    /// what the tab shows was truncated, and is read whole.
    fn read(&mut self, change: Change) -> Reading {
        let path = self.watch_path.clone();
        match std::fs::metadata(&path) {
            Ok(metadata) if metadata.is_dir() => {
                return Reading::Failed("This is a folder, not a file".to_string())
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Reading::Missing,
            Err(e) => return Reading::Failed(format!("Cannot read this file: {}", e)),
        }
        let result = if self.follow && change == Change::Written && self.read_once {
            self.read_appended(&path)
        } else {
            self.read_whole(&path)
        };
        match result {
            Ok(reading) => reading,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Reading::Missing,
            Err(e) => Reading::Failed(format!("Cannot read this file: {}", e)),
        }
    }

    fn read_whole(&mut self, path: &Path) -> std::io::Result<Reading> {
        let bytes = std::fs::read(path)?;
        let (lines, rest) = split_at_line_end(&bytes);
        let mut shown = lines_of(lines);
        shown.extend(lines_of(rest));
        self.shown = bytes.len() as u64;
        self.partial = rest.to_vec();
        Ok(Reading::Whole(shown))
    }

    fn read_appended(&mut self, path: &Path) -> std::io::Result<Reading> {
        let mut file = std::fs::File::open(path)?;
        let length = file.metadata()?.len();
        if length < self.shown {
            return self.read_whole(path);
        }
        file.seek(SeekFrom::Start(self.shown))?;
        let mut added = Vec::new();
        file.read_to_end(&mut added)?;
        self.shown += added.len() as u64;
        let replaces_last = !self.partial.is_empty();
        let mut text = std::mem::take(&mut self.partial);
        text.extend_from_slice(&added);
        let (lines, rest) = split_at_line_end(&text);
        let mut shown = lines_of(lines);
        shown.extend(lines_of(rest));
        self.partial = rest.to_vec();
        Ok(Reading::Appended {
            lines: shown,
            replaces_last,
        })
    }
}

/// What a file box's tab says, dimmed, while its file does not exist: that it waits for the
/// file, and the file's path as the configuration gives it.
fn missing_placeholder(file: &str) -> Vec<String> {
    vec![
        "\u{1b}[2mWaiting for this file to appear:\u{1b}[22m".to_string(),
        format!(
            "\u{1b}[2m{}\u{1b}[22m",
            crate::text::sanitize_output(file).join(" ")
        ),
    ]
}

impl<D: Display> Runtime<D> {
    /// Makes the file boxes match the app's: a box whose `file` or `follow` changed, or that is
    /// new, is watched and read now; a box that is gone stops watching.
    pub(super) fn sync_file_boxes(&mut self) {
        let mut wanted: Vec<(String, String, PathBuf, bool)> = Vec::new();
        for layout in &self.ctx.app.layouts {
            for muxbox in layout.get_all_muxboxes() {
                if let Some(file) = &muxbox.file {
                    wanted.push((
                        muxbox.id.clone(),
                        file.clone(),
                        self.files.resolve(file),
                        muxbox.follow == Some(true),
                    ));
                }
            }
        }
        let gone: Vec<String> = self
            .files
            .boxes
            .iter()
            .filter(|(id, file_box)| {
                !wanted.iter().any(|(wanted_id, file, path, follow)| {
                    wanted_id == *id
                        && file == &file_box.file
                        && path == &file_box.path
                        && *follow == file_box.follow
                })
            })
            .map(|(id, _)| id.clone())
            .collect();
        for id in gone {
            self.files.forget(&id);
        }
        for (id, file, path, follow) in wanted {
            if self.files.boxes.contains_key(&id) {
                continue;
            }
            self.files.boxes.insert(
                id,
                FileBox {
                    file,
                    watch_path: path.clone(),
                    path,
                    follow,
                    folder: None,
                    file_watch: None,
                    stale: Some(Change::Moved),
                    read_once: false,
                    shown: 0,
                    partial: Vec::new(),
                },
            );
        }
        self.read_file_boxes();
    }

    /// Reads every file box that changed since it was last read, into its file tab.
    pub(super) fn read_file_boxes(&mut self) {
        let stale: Vec<(String, Change)> = self
            .files
            .boxes
            .iter_mut()
            .filter_map(|(id, file_box)| file_box.stale.take().map(|change| (id.clone(), change)))
            .collect();
        for (box_id, change) in stale {
            let armed = if change == Change::Moved {
                self.files.arm(&box_id)
            } else {
                Ok(())
            };
            let file_box = self
                .files
                .boxes
                .get_mut(&box_id)
                .expect("a stale box is registered");
            let reading = match armed {
                // Watching first and reading after, so a change made as it is read is reported.
                Ok(()) => file_box.read(change),
                Err(e) => Reading::Failed(e),
            };
            let first_read = !file_box.read_once;
            if matches!(reading, Reading::Whole(_) | Reading::Appended { .. }) {
                file_box.read_once = true;
            } else {
                // The file that appears next is read whole, and a followed one shows its end.
                file_box.read_once = false;
                file_box.shown = 0;
                file_box.partial.clear();
            }
            let follow = file_box.follow;
            let file = file_box.file.clone();
            let Some(muxbox) = self.ctx.app.get_muxbox_by_id_mut(&box_id) else {
                continue;
            };
            let stream_id = format!("{}/file", box_id);
            let Some(stream) = muxbox.streams.get_mut(&stream_id) else {
                continue;
            };
            match reading {
                Reading::Whole(lines) => {
                    stream.content = lines;
                    stream.status = StreamStatus::Idle;
                }
                Reading::Appended {
                    lines,
                    replaces_last,
                } => {
                    if replaces_last {
                        stream.content.pop();
                    }
                    stream.content.extend(lines);
                    stream.status = StreamStatus::Idle;
                }
                Reading::Missing => {
                    stream.content = missing_placeholder(&file);
                    stream.status = StreamStatus::Idle;
                }
                Reading::Failed(reason) => {
                    stream.content = vec![
                        reason.clone(),
                        crate::text::sanitize_output(&file).join(" "),
                    ];
                    stream.status = StreamStatus::Failed(reason);
                }
            }
            // A followed file keeps to its end while the reader is there; one who scrolled up
            // to read stays where they are.
            let visible = muxbox
                .get_selected_stream()
                .is_some_and(|stream| stream.id == stream_id);
            if follow && visible && (first_read || muxbox.current_vertical_scroll() >= 100.0) {
                muxbox.vertical_scroll = Some(100.0);
            }
            self.dirty = true;
        }
    }
}
