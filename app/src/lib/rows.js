// Build the visible, sorted, filtered row list for a pane.
// Directories always sort before files; ".." is always the first row.
export function computeRows(pane) {
  let entries = pane.entries;
  if (pane.filter) {
    const f = pane.filter.toLowerCase();
    entries = entries.filter((e) => e.name.toLowerCase().includes(f));
  }

  const dir = pane.sortAsc ? 1 : -1;
  const sorted = [...entries].sort((a, b) => {
    if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;
    let r;
    switch (pane.sortKey) {
      case "size":
        r = a.size - b.size;
        break;
      case "modified":
        r = (a.modified || 0) - (b.modified || 0);
        break;
      case "extension":
        r = a.extension.localeCompare(b.extension) || a.name.localeCompare(b.name);
        break;
      default:
        r = a.name.localeCompare(b.name, undefined, { numeric: true });
    }
    return r * dir;
  });

  const rows = [];
  if (pane.parentPath != null) {
    rows.push({
      name: "..",
      path: pane.parentPath,
      isDir: true,
      isParent: true,
      isSymlink: false,
      size: 0,
      modified: null,
      extension: "",
    });
  }
  for (const e of sorted) rows.push(e);
  return rows;
}
