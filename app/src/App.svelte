<script>
  import { SvelteSet } from "svelte/reactivity";
  import Pane from "./lib/Pane.svelte";
  import { computeRows } from "./lib/rows.js";
  import { baseName } from "./lib/format.js";
  import * as api from "./lib/api.js";

  function makePane(id) {
    return {
      id,
      path: "",
      parentPath: null,
      entries: [],
      cursor: 0,
      selected: new SvelteSet(),
      sortKey: "name",
      sortAsc: true,
      filter: "",
    };
  }

  let panes = $state([makePane(0), makePane(1)]);
  let active = $state(0);
  let drives = $state([]);
  let modal = $state(null);
  let progress = $state({ active: false, current: "", copied: 0, total: 0 });
  let modalInput = $state(null);

  let activePane = $derived(panes[active]);
  let otherPane = $derived(panes[active === 0 ? 1 : 0]);
  let activeRows = $derived(computeRows(activePane));
  let cursorRow = $derived(activeRows[activePane.cursor]);

  // Re-arm the file-system watcher on the active pane's directory.
  $effect(() => {
    const p = panes[active].path;
    if (p) api.watchDir(p).catch(() => {});
  });

  $effect(() => {
    // Clamp cursor whenever the visible row set shrinks (filter/reload).
    if (activePane.cursor >= activeRows.length) {
      activePane.cursor = Math.max(0, activeRows.length - 1);
    }
  });

  let fsTimer;
  $effect(() => {
    const unlisteners = [];
    (async () => {
      unlisteners.push(
        await api.onFsChange(() => {
          clearTimeout(fsTimer);
          fsTimer = setTimeout(() => {
            reloadPane(panes[0]);
            reloadPane(panes[1]);
          }, 150);
        }),
      );
      unlisteners.push(
        await api.onCopyProgress((p) => {
          progress = { ...progress, current: p.currentFile, copied: p.copied, total: p.total };
        }),
      );
    })();
    return () => unlisteners.forEach((u) => u && u());
  });

  init();
  async function init() {
    let home = "/";
    try {
      home = await api.homeDir();
    } catch (_) {}
    await navigate(panes[0], home);
    await navigate(panes[1], home);
    try {
      drives = await api.listDrives();
    } catch (_) {}
  }

  async function navigate(pane, path, focusName) {
    try {
      const listing = await api.listDir(path);
      pane.path = listing.path;
      pane.parentPath = listing.parent;
      pane.entries = listing.entries;
      pane.selected.clear();
      pane.filter = "";
      let cur = 0;
      if (focusName) {
        const rows = computeRows(pane);
        const i = rows.findIndex((r) => r.name === focusName);
        if (i >= 0) cur = i;
      }
      pane.cursor = cur;
    } catch (e) {
      showError(String(e));
    }
  }

  async function reloadPane(pane) {
    try {
      const listing = await api.listDir(pane.path);
      pane.entries = listing.entries;
      pane.parentPath = listing.parent;
      const existing = new Set(pane.entries.map((e) => e.path));
      for (const p of [...pane.selected]) {
        if (!existing.has(p)) pane.selected.delete(p);
      }
      const rows = computeRows(pane);
      if (pane.cursor >= rows.length) pane.cursor = Math.max(0, rows.length - 1);
    } catch (_) {}
  }

  function open(pane, row) {
    if (!row) return;
    if (row.isDir) {
      const focus = row.isParent ? baseName(pane.path) : undefined;
      navigate(pane, row.path, focus);
    } else {
      api.openPath(row.path).catch((e) => showError(String(e)));
    }
  }

  function moveCursor(delta) {
    const c = activePane.cursor + delta;
    activePane.cursor = Math.max(0, Math.min(activeRows.length - 1, c));
  }

  function toggleMark() {
    const row = activeRows[activePane.cursor];
    if (row && !row.isParent) {
      if (activePane.selected.has(row.path)) activePane.selected.delete(row.path);
      else activePane.selected.add(row.path);
    }
    if (activePane.cursor < activeRows.length - 1) activePane.cursor += 1;
  }

  function opTargets() {
    if (activePane.selected.size > 0) return [...activePane.selected];
    const row = activeRows[activePane.cursor];
    if (row && !row.isParent) return [row.path];
    return [];
  }

  function setSort(pane, key) {
    if (pane.sortKey === key) pane.sortAsc = !pane.sortAsc;
    else {
      pane.sortKey = key;
      pane.sortAsc = true;
    }
    pane.cursor = 0;
  }

  function showError(message) {
    modal = { type: "error", message };
  }

  // ---- operations that open modals ----
  function askMkdir() {
    modal = { type: "mkdir", value: "" };
  }
  function askRename() {
    const row = activeRows[activePane.cursor];
    if (!row || row.isParent) return;
    modal = { type: "rename", value: row.name, path: row.path };
  }
  function askDelete(permanent) {
    const targets = opTargets();
    if (!targets.length) return;
    modal = { type: "confirmDelete", targets, permanent };
  }
  function askTransfer(op) {
    const targets = opTargets();
    if (!targets.length) return;
    if (otherPane.path === activePane.path) {
      showError("원본과 대상 경로가 같습니다");
      return;
    }
    modal = { type: op === "copy" ? "confirmCopy" : "confirmMove", targets, dest: otherPane.path, op };
  }

  // ---- modal confirm handlers ----
  async function confirmModal() {
    const m = modal;
    if (!m) return;
    try {
      if (m.type === "mkdir") {
        if (!m.value.trim()) return;
        await api.createDir(activePane.path, m.value.trim());
        modal = null;
        await reloadPane(activePane);
      } else if (m.type === "rename") {
        if (!m.value.trim()) return;
        await api.renamePath(m.path, m.value.trim());
        modal = null;
        await reloadPane(activePane);
      } else if (m.type === "confirmDelete") {
        modal = null;
        await api.deletePaths(m.targets, !m.permanent);
        activePane.selected.clear();
        await reloadPane(activePane);
      } else if (m.type === "confirmCopy" || m.type === "confirmMove") {
        const targets = m.targets;
        const dest = m.dest;
        const op = m.op;
        modal = null;
        progress = { active: true, current: "", copied: 0, total: 0 };
        try {
          if (op === "copy") await api.copyPaths(targets, dest);
          else await api.movePaths(targets, dest);
        } finally {
          progress = { ...progress, active: false };
        }
        activePane.selected.clear();
        await reloadPane(panes[0]);
        await reloadPane(panes[1]);
      } else if (m.type === "error") {
        modal = null;
      }
    } catch (e) {
      showError(String(e));
    }
  }

  function cancelModal() {
    modal = null;
  }

  // ---- global keyboard ----
  function onKeydown(e) {
    if (modal || progress.active) return;
    const k = e.key;

    if (k === "Tab") {
      e.preventDefault();
      active = active === 0 ? 1 : 0;
      return;
    }
    if (k === "ArrowDown") return (e.preventDefault(), moveCursor(1));
    if (k === "ArrowUp") return (e.preventDefault(), moveCursor(-1));
    if (k === "PageDown") return (e.preventDefault(), moveCursor(15));
    if (k === "PageUp") return (e.preventDefault(), moveCursor(-15));
    if (k === "Home") return (e.preventDefault(), (activePane.cursor = 0));
    if (k === "End")
      return (e.preventDefault(), (activePane.cursor = Math.max(0, activeRows.length - 1)));
    if (k === "Enter") return (e.preventDefault(), open(activePane, cursorRow));
    if (k === " " || k === "Insert") return (e.preventDefault(), toggleMark());
    if (k === "Backspace") {
      e.preventDefault();
      if (activePane.filter) activePane.filter = activePane.filter.slice(0, -1);
      else if (activePane.parentPath != null)
        navigate(activePane, activePane.parentPath, baseName(activePane.path));
      return;
    }
    if (k === "Escape") {
      e.preventDefault();
      activePane.filter = "";
      activePane.selected.clear();
      return;
    }
    if (k === "F2") return (e.preventDefault(), askRename());
    if (k === "F5") return (e.preventDefault(), askTransfer("copy"));
    if (k === "F6") return (e.preventDefault(), askTransfer("move"));
    if (k === "F7") return (e.preventDefault(), askMkdir());
    if (k === "F8" || k === "Delete") return (e.preventDefault(), askDelete(e.shiftKey));

    // Quick filter: printable single character.
    if (k.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
      activePane.filter += k;
      activePane.cursor = 0;
      e.preventDefault();
    }
  }

  function onModalKeydown(e) {
    if (e.key === "Enter") {
      e.preventDefault();
      confirmModal();
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancelModal();
    }
  }

  $effect(() => {
    if (modal && (modal.type === "mkdir" || modal.type === "rename") && modalInput) {
      modalInput.focus();
      modalInput.select();
    }
  });

  function pct() {
    if (!progress.total) return 0;
    return Math.min(100, Math.round((progress.copied / progress.total) * 100));
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="panes">
  {#each panes as pane, i (pane.id)}
    <Pane
      {pane}
      {drives}
      active={active === i}
      onActivate={() => (active = i)}
      onSetCursor={(idx) => (pane.cursor = idx)}
      onOpen={(row) => open(pane, row)}
      onSort={(key) => setSort(pane, key)}
      onDrive={(path) => {
        active = i;
        navigate(pane, path);
      }}
    />
  {/each}
</div>

<div class="statusbar">
  <button onclick={askRename}><b>F2</b> 이름변경</button>
  <button onclick={() => askTransfer("copy")}><b>F5</b> 복사</button>
  <button onclick={() => askTransfer("move")}><b>F6</b> 이동</button>
  <button onclick={askMkdir}><b>F7</b> 새폴더</button>
  <button onclick={() => askDelete(false)}><b>F8</b> 삭제</button>
</div>

{#if modal}
  <div class="overlay" onmousedown={cancelModal} role="presentation">
    <div class="modal" onmousedown={(e) => e.stopPropagation()} role="dialog" tabindex="-1">
      {#if modal.type === "mkdir"}
        <h3>새 폴더 만들기</h3>
        <input
          bind:this={modalInput}
          bind:value={modal.value}
          onkeydown={onModalKeydown}
          placeholder="폴더 이름"
        />
        <div class="actions">
          <button onclick={cancelModal}>취소</button>
          <button class="primary" onclick={confirmModal}>만들기</button>
        </div>
      {:else if modal.type === "rename"}
        <h3>이름 변경</h3>
        <input
          bind:this={modalInput}
          bind:value={modal.value}
          onkeydown={onModalKeydown}
        />
        <div class="actions">
          <button onclick={cancelModal}>취소</button>
          <button class="primary" onclick={confirmModal}>변경</button>
        </div>
      {:else if modal.type === "confirmDelete"}
        <h3>삭제 확인</h3>
        <p>{modal.targets.length}개 항목을 {modal.permanent ? "영구 삭제" : "휴지통으로 이동"}합니다.</p>
        <div class="actions">
          <button onclick={cancelModal}>취소</button>
          <button class="danger" onclick={confirmModal}>삭제</button>
        </div>
      {:else if modal.type === "confirmCopy" || modal.type === "confirmMove"}
        <h3>{modal.type === "confirmCopy" ? "복사" : "이동"} 확인</h3>
        <p>{modal.targets.length}개 항목을 다음 위치로 {modal.type === "confirmCopy" ? "복사" : "이동"}:</p>
        <p><b>{modal.dest}</b></p>
        <div class="actions">
          <button onclick={cancelModal}>취소</button>
          <button class="primary" onclick={confirmModal}>확인</button>
        </div>
      {:else if modal.type === "error"}
        <h3>오류</h3>
        <p class="error-text">{modal.message}</p>
        <div class="actions">
          <button class="primary" onclick={cancelModal}>확인</button>
        </div>
      {/if}
    </div>
  </div>
{/if}

{#if progress.active}
  <div class="overlay" role="presentation">
    <div class="modal" role="dialog" tabindex="-1">
      <h3>작업 진행 중…</h3>
      <p>{progress.current}</p>
      <div class="progress-track">
        <div class="progress-fill" style={`width:${pct()}%`}></div>
      </div>
      <p>{pct()}%</p>
    </div>
  </div>
{/if}
