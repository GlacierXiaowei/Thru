use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::sync::mpsc::channel;
use std::path::Path;
use anyhow::Result;

pub fn watch_directory(path: &Path, callback: impl Fn(&str)) -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            for path in &event.paths {
                if let EventKind::Create(_) = event.kind {
                    if let Some(name) = path.file_name() {
                        tx.send(name.to_string_lossy().to_string()).ok();
                    }
                }
            }
        }
    })?;

    watcher.watch(path, RecursiveMode::NonRecursive)?;

    println!("👁️  监控中: {:?}", path);
    println!("   按 Ctrl+C 停止\n");

    for filename in rx {
        callback(&filename);
    }

    Ok(())
}