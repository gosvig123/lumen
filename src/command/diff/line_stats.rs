use super::diff_algo::count_added_removed;
use super::types::{FileDiff, FileStatus};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileLineStats {
    pub added: usize,
    pub removed: usize,
    pub binary: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatKind {
    Added,
    Removed,
}

pub fn compute_file_line_stats(file_diffs: &[FileDiff]) -> Vec<FileLineStats> {
    file_diffs
        .iter()
        .map(|diff| {
            let (added, removed) = if diff.is_binary {
                (0, 0)
            } else {
                count_added_removed(&diff.old_content, &diff.new_content)
            };
            FileLineStats {
                added,
                removed,
                binary: diff.is_binary,
            }
        })
        .collect()
}

pub fn total_line_stats(stats: &[FileLineStats]) -> (usize, usize) {
    stats.iter().fold((0, 0), |(added, removed), stat| {
        (added + stat.added, removed + stat.removed)
    })
}

pub fn display_parts(status: FileStatus, stats: FileLineStats) -> Vec<(StatKind, String)> {
    if stats.binary {
        return Vec::new();
    }
    match status {
        FileStatus::Added => vec![(StatKind::Added, format!("+{}", stats.added))],
        FileStatus::Deleted => vec![(StatKind::Removed, format!("-{}", stats.removed))],
        FileStatus::Modified => vec![
            (StatKind::Added, format!("+{}", stats.added)),
            (StatKind::Removed, format!("-{}", stats.removed)),
        ],
    }
}

pub fn display_width(status: FileStatus, stats: FileLineStats) -> usize {
    display_parts(status, stats)
        .iter()
        .map(|(_, text)| text.chars().count() + 1)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diff(status: FileStatus, old: &str, new: &str) -> FileDiff {
        FileDiff {
            filename: "file.txt".into(),
            old_content: old.into(),
            new_content: new.into(),
            status,
            is_binary: false,
        }
    }

    #[test]
    fn computes_and_totals_per_file_stats() {
        let files = vec![
            diff(FileStatus::Modified, "one\ntwo\n", "one\nthree\nfour\n"),
            diff(FileStatus::Added, "", "new\n"),
        ];
        let stats = compute_file_line_stats(&files);

        assert_eq!(stats[0], FileLineStats { added: 2, removed: 1, binary: false });
        assert_eq!(stats[1], FileLineStats { added: 1, removed: 0, binary: false });
        assert_eq!(total_line_stats(&stats), (3, 1));
    }

    #[test]
    fn formats_counts_by_file_status() {
        let stats = FileLineStats { added: 4, removed: 2, binary: false };

        assert_eq!(display_parts(FileStatus::Added, stats), vec![(StatKind::Added, "+4".into())]);
        assert_eq!(display_parts(FileStatus::Deleted, stats), vec![(StatKind::Removed, "-2".into())]);
        assert_eq!(display_parts(FileStatus::Modified, stats).len(), 2);
        assert_eq!(display_width(FileStatus::Modified, stats), 6);

        let binary = FileLineStats { binary: true, ..stats };
        assert!(display_parts(FileStatus::Modified, binary).is_empty());
        assert_eq!(display_width(FileStatus::Modified, binary), 0);
    }
}
