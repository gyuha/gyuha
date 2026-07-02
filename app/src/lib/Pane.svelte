<script>
  import { computeRows } from "./rows.js";
  import { formatSize, formatDate } from "./format.js";

  let { pane, active, drives, onActivate, onSetCursor, onOpen, onSort, onDrive } =
    $props();

  const ROW_H = 22;

  let listEl = $state(null);
  let scrollTop = $state(0);
  let viewportH = $state(400);

  let rows = $derived(computeRows(pane));
  let total = $derived(rows.length);
  let start = $derived(Math.max(0, Math.floor(scrollTop / ROW_H) - 5));
  let visCount = $derived(Math.ceil(viewportH / ROW_H) + 10);
  let end = $derived(Math.min(total, start + visCount));
  let visible = $derived(rows.slice(start, end));

  let markedCount = $derived(pane.selected.size);
  let selectedBytes = $derived(
    pane.entries
      .filter((e) => !e.isDir && pane.selected.has(e.path))
      .reduce((a, e) => a + e.size, 0),
  );

  // Keep the cursor row inside the viewport.
  $effect(() => {
    const c = pane.cursor;
    if (!listEl) return;
    const top = c * ROW_H;
    const bottom = top + ROW_H;
    if (top < listEl.scrollTop) {
      listEl.scrollTop = top;
    } else if (bottom > listEl.scrollTop + viewportH) {
      listEl.scrollTop = bottom - viewportH;
    }
  });

  function onScroll(e) {
    scrollTop = e.currentTarget.scrollTop;
  }

  function sortArrow(key) {
    if (pane.sortKey !== key) return "";
    return pane.sortAsc ? " ▲" : " ▼";
  }
</script>

<div
  class="pane"
  class:active
  onmousedown={() => onActivate()}
  role="group"
  tabindex="-1"
>
  <div class="drivebar">
    {#each drives as d (d.path)}
      <button onclick={() => onDrive(d.path)} title={`${d.name} — ${d.path}`}>
        {d.path}
      </button>
    {/each}
  </div>

  <div class="pathbar">
    {pane.path}
    {#if pane.filter}
      <span class="filter">　[필터: {pane.filter}]</span>
    {/if}
  </div>

  <div class="colhead">
    <div class="c-name" onclick={() => onSort("name")} role="button" tabindex="-1">
      이름{sortArrow("name")}
    </div>
    <div class="c-ext" onclick={() => onSort("extension")} role="button" tabindex="-1">
      확장자{sortArrow("extension")}
    </div>
    <div class="c-size" onclick={() => onSort("size")} role="button" tabindex="-1">
      크기{sortArrow("size")}
    </div>
    <div class="c-date" onclick={() => onSort("modified")} role="button" tabindex="-1">
      수정일{sortArrow("modified")}
    </div>
  </div>

  <div class="list" bind:this={listEl} bind:clientHeight={viewportH} onscroll={onScroll}>
    <div class="spacer" style={`height:${total * ROW_H}px`}>
      <div class="rows" style={`transform:translateY(${start * ROW_H}px)`}>
        {#each visible as row, i (row.path + row.name)}
          {@const idx = start + i}
          <div
            class="row"
            class:dir={row.isDir}
            class:cursor={idx === pane.cursor}
            class:marked={!row.isParent && pane.selected.has(row.path)}
            onmousedown={() => {
              onActivate();
              onSetCursor(idx);
            }}
            ondblclick={() => onOpen(row)}
            role="row"
            tabindex="-1"
          >
            <div class="c-name">{row.isDir ? "📁" : "📄"} {row.name}</div>
            <div class="c-ext">{row.isParent ? "" : row.extension}</div>
            <div class="c-size">{formatSize(row.size, row.isDir)}</div>
            <div class="c-date">{formatDate(row.modified)}</div>
          </div>
        {/each}
      </div>
    </div>
  </div>

  <div class="pane-footer">
    <span>
      {#if markedCount > 0}
        {markedCount}개 선택 · {formatSize(selectedBytes, false)}
      {:else}
        {pane.entries.length}개 항목
      {/if}
    </span>
    <span>{pane.entries.filter((e) => e.isDir).length} 폴더</span>
  </div>
</div>
