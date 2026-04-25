<template>
  <div class="detail" v-if="detail">
    <div class="detail-top">
      <button @click="goBack" class="btn">Back</button>
      <button @click="goRead" class="btn accent">Read</button>
    </div>
    <div class="detail-body">
      <div class="detail-left">
        <img v-if="detail.manga.cover_cache_path" :src="toSrc(detail.manga.cover_cache_path)" class="cover" />
      </div>
      <div class="detail-right">
        <h2>{{ detail.manga.eh_title || detail.manga.title }}</h2>
        <div v-if="detail.manga.eh_title_jpn" class="sub-title">{{ detail.manga.eh_title_jpn }}</div>
        <div class="meta-row"><label>Category:</label><span>{{ detail.manga.eh_category || '-' }}</span></div>
        <div class="meta-row"><label>Pages:</label><span>{{ detail.manga.page_count }}</span></div>
        <div class="meta-row"><label>Uploader:</label><span>{{ detail.manga.eh_uploader || '-' }}</span></div>
        <div class="meta-row">
          <label>Score:</label>
          <input type="number" v-model.number="score" min="0" max="10" step="0.5" class="score-input" />
          <button @click="saveScore" class="btn-sm">Save</button>
        </div>
        <div class="meta-row"><label>Tag Status:</label><span :class="'ts ' + detail.manga.tag_status">{{ detail.manga.tag_status }}</span></div>
        <div class="tags-section">
          <h3>Tags</h3>
          <div class="tag-groups">
            <div v-for="(tags, ns) in groupedTags" :key="ns" class="tag-group">
              <span class="tag-ns">{{ ns }}:</span>
              <span v-for="t in tags" :key="t" class="tag-item">{{ t }}</span>
            </div>
          </div>
          <div v-if="Object.keys(groupedTags).length===0" class="no-tags">No tags yet. Paste a gallery URL below.</div>
        </div>
        <div class="url-section">
          <input v-model="ehUrl" placeholder="Paste E-Hentai / ExHentai gallery URL" class="url-input" />
          <button @click="fetchByUrl" class="btn" :disabled="fetching">{{ fetching ? 'Fetching...' : 'Fetch Tags' }}</button>
        </div>
        <div class="url-section" style="margin-top:8px">
          <input v-model="gidInput" placeholder="Or enter GID" class="url-input" style="width:120px" />
          <input v-model="tokenInput" placeholder="Token" class="url-input" style="width:200px" />
          <button @click="fetchByGid" class="btn" :disabled="fetching">Fetch by GID</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";

interface Tag { id: number; namespace: string; tag: string; }
interface MangaDetail { manga: any; tags: Tag[]; read_progress: number | null; }

const route = useRoute();
const router = useRouter();
const id = Number(route.params.id);
const detail = ref<MangaDetail | null>(null);
const score = ref(0);
const ehUrl = ref("");
const gidInput = ref("");
const tokenInput = ref("");
const fetching = ref(false);

function toSrc(p: string | null): string {
  if (!p) return "";
  return convertFileSrc(p);
}

const groupedTags = computed(() => {
  if (!detail.value) return {};
  const groups: Record<string, string[]> = {};
  for (const t of detail.value.tags) {
    if (!groups[t.namespace]) groups[t.namespace] = [];
    groups[t.namespace].push(t.tag);
  }
  return groups;
});

onMounted(async () => {
  detail.value = await invoke<MangaDetail>("get_manga_detail", { mangaId: id });
  score.value = detail.value.manga.score;
});

async function saveScore() {
  await invoke("set_score", { mangaId: id, score: score.value });
  detail.value = await invoke<MangaDetail>("get_manga_detail", { mangaId: id });
}

async function fetchByUrl() {
  if (!ehUrl.value.trim()) return;
  fetching.value = true;
  try {
    await invoke("fetch_tags_by_url", { mangaId: id, url: ehUrl.value.trim() });
    detail.value = await invoke<MangaDetail>("get_manga_detail", { mangaId: id });
    alert("Tags fetched successfully!");
  } catch (e) {
    alert("Error: " + e);
  } finally {
    fetching.value = false;
  }
}

async function fetchByGid() {
  if (!gidInput.value.trim() || !tokenInput.value.trim()) return;
  fetching.value = true;
  try {
    const url = "https://exhentai.org/g/" + gidInput.value.trim() + "/" + tokenInput.value.trim() + "/";
    await invoke("fetch_tags_by_url", { mangaId: id, url: url });
    detail.value = await invoke<MangaDetail>("get_manga_detail", { mangaId: id });
    alert("Tags fetched successfully!");
  } catch (e) {
    alert("Error: " + e);
  } finally {
    fetching.value = false;
  }
}

function goBack() { router.push("/"); }
function goRead() { router.push("/reader/" + id); }
</script>

<style scoped>
.detail { height: 100%; display: flex; flex-direction: column; color: var(--text); }
.detail-top { display: flex; gap: 8px; padding: 8px 12px; background: var(--bg2); border-bottom: 1px solid var(--border); flex-shrink: 0; }
.btn { background: var(--bg3); color: var(--text); border: 1px solid var(--border); padding: 4px 12px; border-radius: 4px; cursor: pointer; }
.btn.accent { background: var(--accent); border-color: var(--accent); }
.btn:hover { opacity: 0.85; }
.btn:disabled { opacity: 0.5; }
.detail-body { flex: 1; overflow-y: auto; display: flex; padding: 16px; gap: 24px; }
.detail-left { flex-shrink: 0; }
.cover { max-height: 400px; max-width: 300px; border-radius: 4px; }
.detail-right { flex: 1; min-width: 0; }
h2 { margin-bottom: 4px; font-size: 18px; word-break: break-word; }
.sub-title { color: var(--text2); font-size: 14px; margin-bottom: 12px; }
.meta-row { display: flex; align-items: center; gap: 8px; margin: 6px 0; font-size: 14px; flex-wrap: wrap; }
.meta-row label { color: var(--text2); min-width: 80px; }
.score-input { width: 60px; background: var(--bg); color: var(--text); border: 1px solid var(--border); padding: 2px 4px; border-radius: 3px; }
.btn-sm { background: var(--bg3); color: var(--text); border: 1px solid var(--border); padding: 2px 8px; border-radius: 3px; cursor: pointer; font-size: 12px; }
.ts { padding: 2px 6px; border-radius: 3px; font-size: 12px; }
.ts.tagged { background: #2d5a27; color: #8f8; }
.ts.non-tag { background: #555; color: #aaa; }
.ts.tag-failed { background: #5a2727; color: #f88; }
.tags-section { margin-top: 16px; }
h3 { margin-bottom: 8px; font-size: 15px; }
.tag-group { display: flex; flex-wrap: wrap; gap: 4px; margin: 4px 0; align-items: center; }
.tag-ns { color: var(--accent); font-size: 12px; font-weight: bold; min-width: 60px; }
.tag-item { background: var(--bg3); padding: 2px 6px; border-radius: 3px; font-size: 11px; }
.no-tags { color: var(--text2); font-size: 13px; margin-top: 8px; }
.url-section { margin-top: 16px; display: flex; gap: 8px; flex-wrap: wrap; }
.url-input { flex: 1; background: var(--bg); color: var(--text); border: 1px solid var(--border); padding: 4px 8px; border-radius: 4px; font-size: 13px; min-width: 150px; }
</style>