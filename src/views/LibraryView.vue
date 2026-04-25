<template>
  <div class="library">
    <div class="toolbar">
      <button @click="selectFolder" class="btn">添加文件夹</button>
      <input v-model="keyword" @keyup.enter="doSearch" placeholder="搜索标题或标签..." class="search-input" />
      <button @click="doSearch" class="btn">搜索</button>
      <button @click="batchFetch" class="btn" :disabled="fetching">{{ fetching ? '获取中...' : '批量获取标签' }}</button>
    </div>
    <div v-if="store.loading" class="status-msg">加载中...</div>
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
            <span class="ts" :class="m.tag_status">{{ tsLabel(m.tag_status) }}</span>
          </div>
        </div>
      </div>
    </div>
    <div v-if="!store.loading && store.mangaList.length===0" class="status-msg">
      没有找到漫画。拖拽文件夹到此处或点击"添加文件夹"开始扫描。
    </div>

    <!-- 拖拽覆盖层 -->
    <div v-if="isDragging" class="drop-overlay">
      <div class="drop-box">
        <div class="drop-icon">&#128194;</div>
        <div>拖放文件夹到此处自动扫描</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useLibraryStore, type MangaItem } from "../stores/main";

const store = useLibraryStore();
const router = useRouter();
const keyword = ref("");
const fetching = ref(false);
const isDragging = ref(false);
let unlisten: (() => void) | null = null;

function tsLabel(s: string): string {
  if (s === "tagged") return "已标记";
  if (s === "non-tag") return "未标记";
  if (s === "tag-failed") return "标记失败";
  return s;
}

function toSrc(p: string | null): string {
  if (!p) return "";
  return convertFileSrc(p);
}

onMounted(async () => {
  store.loadManga();

  // Tauri 2 drag-drop listener
  try {
    unlisten = await getCurrentWebview().onDragDropEvent(async (event) => {
      if (event.payload.type === "enter") {
        isDragging.value = true;
      } else if (event.payload.type === "leave") {
        isDragging.value = false;
      } else if (event.payload.type === "drop") {
        isDragging.value = false;
        const paths = event.payload.paths;
        if (paths.length > 0) {
          const result = await store.scanLibrary(paths);
          alert("扫描完成：新增 " + result.added + " 部 / 共扫描 " + result.total + " 个文件");
          store.loadManga();
        }
      }
    });
  } catch (e) {
    console.warn("拖拽监听未启用:", e);
  }
});

onUnmounted(() => {
  if (unlisten) unlisten();
});

async function selectFolder() {
  const selected = await open({ directory: true, multiple: true });
  if (selected && Array.isArray(selected)) {
    const paths = selected.map(String);
    const result = await store.scanLibrary(paths);
    alert("扫描完成：新增 " + result.added + " 部 / 共扫描 " + result.total + " 个文件");
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
    let msg = "成功: " + r.success.length + ", 失败: " + r.failed.length;
    if (r.ip_banned) msg += "\n\nIP 已被封禁！请等待 5-10 分钟后重试。";
    alert(msg);
    store.loadManga();
  } catch (e) {
    alert("错误: " + e);
  } finally {
    fetching.value = false;
  }
}

function goDetail(id: number) { router.push("/detail/" + id); }
</script>

<style scoped>
.library { height: 100%; display: flex; flex-direction: column; position: relative; }
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
.status-msg { flex: 1; display: flex; align-items: center; justify-content: center; color: var(--text2); font-size: 16px; padding: 40px; text-align: center; }

.drop-overlay {
  position: fixed; top: 0; left: 0; width: 100%; height: 100%;
  background: rgba(0,0,0,0.7); z-index: 9999;
  display: flex; align-items: center; justify-content: center;
}
.drop-box {
  background: var(--bg2); border: 3px dashed var(--accent); border-radius: 16px;
  padding: 60px 80px; text-align: center; color: var(--text);
}
.drop-icon { font-size: 64px; margin-bottom: 16px; }
</style>