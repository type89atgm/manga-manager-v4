<template>
  <div class="library">
    <div class="toolbar">
      <button @click="selectFolder" class="btn">Add Folder</button>
      <input v-model="keyword" @keyup.enter="doSearch" placeholder="Search title or tag..." class="search-input" />
      <button @click="doSearch" class="btn">Search</button>
      <button @click="batchFetch" class="btn" :disabled="fetching">{{ fetching ? 'Fetching...' : 'Batch Fetch Tags' }}</button>
    </div>
    <div v-if="store.loading" class="status-msg">Loading...</div>
    <div v-else class="grid">
      <div v-for="m in store.mangaList" :key="m.id" class="card" @click="goDetail(m.id)">
        <div class="thumb-wrap">
          <img v-if="m.cover_cache_path" :src="toSrc(m.cover_cache_path)" class="thumb" />
          <div v-else class="thumb-ph">?</div>
        </div>
        <div class="card-info">
          <div class="card-title" :title="m.title">{{ m.title }}</div>
          <div class="card-meta">
            <span class="score">{{ m.score > 0 ? m.score.toFixed(1) : '-' }}</span>
            <span class="ts" :class="m.tag_status">{{ m.tag_status }}</span>
          </div>
        </div>
      </div>
    </div>
    <div v-if="!store.loading && store.mangaList.length===0" class="status-msg">No manga found. Click "Add Folder" to scan your library.</div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useLibraryStore } from "../stores/main";

const store = useLibraryStore();
const router = useRouter();
const keyword = ref("");
const fetching = ref(false);

function toSrc(p: string | null): string {
  if (!p) return "";
  return convertFileSrc(p);
}

onMounted(() => { store.loadManga(); });

async function selectFolder() {
  const selected = await open({ directory: true, multiple: true });
  if (selected && Array.isArray(selected)) {
    const paths = selected.map(String);
    const result = await store.scanLibrary(paths);
    alert("Added " + result.added + " / " + result.total + " archives scanned");
    store.loadManga();
  }
}

async function doSearch() {
  if (keyword.value.trim()) {
    store.searchManga(keyword.value.trim());
  } else {
    store.loadManga();
  }
}

async function batchFetch() {
  fetching.value = true;
  try {
    const r = await invoke<{ success: number[]; failed: number[]; ip_banned: boolean }>("batch_fetch_tags");
    let msg = "Success: " + r.success.length + ", Failed: " + r.failed.length;
    if (r.ip_banned) msg += "\n\nIP BANNED! Wait 5-10 minutes before retrying.";
    alert(msg);
    store.loadManga();
  } catch (e) {
    alert("Error: " + e);
  } finally {
    fetching.value = false;
  }
}

function goDetail(id: number) { router.push("/detail/" + id); }
</script>

<style scoped>
.library { height: 100%; display: flex; flex-direction: column; }
.toolbar {
  display: flex; gap: 8px; padding: 8px 12px;
  background: var(--bg2); border-bottom: 1px solid var(--border); align-items: center; flex-shrink: 0;
}
.btn {
  background: var(--bg3); color: var(--text); border: 1px solid var(--border);
  padding: 4px 12px; border-radius: 4px; cursor: pointer; font-size: 13px;
}
.btn:hover { background: var(--accent2); }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.search-input {
  background: var(--bg); color: var(--text); border: 1px solid var(--border);
  padding: 4px 8px; border-radius: 4px; width: 200px; font-size: 13px;
}
.grid {
  flex: 1; overflow-y: auto; padding: 12px;
  display: flex; flex-wrap: wrap; gap: 12px; align-content: flex-start;
}
.card {
  width: 180px; cursor: pointer; background: var(--bg2); border-radius: 6px;
  overflow: hidden; border: 1px solid var(--border); transition: transform 0.15s;
}
.card:hover { transform: scale(1.03); border-color: var(--accent); }
.thumb-wrap {
  width: 180px; height: 240px; background: var(--bg);
  display: flex; align-items: center; justify-content: center; overflow: hidden;
}
.thumb { max-width: 100%; max-height: 100%; object-fit: contain; }
.thumb-ph { color: var(--text2); font-size: 48px; }
.card-info { padding: 6px 8px; }
.card-title { font-size: 12px; color: var(--text); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.card-meta { display: flex; justify-content: space-between; margin-top: 4px; font-size: 11px; }
.score { color: #ffd700; }
.ts { padding: 1px 4px; border-radius: 3px; font-size: 10px; }
.ts.tagged { background: #2d5a27; color: #8f8; }
.ts.non-tag { background: #555; color: #aaa; }
.ts.tag-failed { background: #5a2727; color: #f88; }
.status-msg { flex: 1; display: flex; align-items: center; justify-content: center; color: var(--text2); font-size: 16px; }
</style>