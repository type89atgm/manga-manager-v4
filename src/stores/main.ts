import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface MangaItem {
  id: number;
  title: string;
  folder: string;
  page_count: number;
  score: number;
  cover_cache_path: string | null;
  tag_status: string;
  eh_category: string | null;
}

export const useLibraryStore = defineStore("library", () => {
  const mangaList = ref<MangaItem[]>([]);
  const loading = ref(false);
  const searchKeyword = ref("");

  async function loadManga(folder?: string) {
    loading.value = true;
    try {
      mangaList.value = await invoke<MangaItem[]>("list_manga", { folder: folder || null });
    } finally {
      loading.value = false;
    }
  }

  async function searchManga(keyword: string) {
    loading.value = true;
    try {
      mangaList.value = await invoke<MangaItem[]>("search_manga", { keyword });
    } finally {
      loading.value = false;
    }
  }

  async function scanLibrary(paths: string[]) {
    loading.value = true;
    try {
      return await invoke<{ added: number; total: number }>("scan_library", { paths });
    } finally {
      loading.value = false;
    }
  }

  return { mangaList, loading, searchKeyword, loadManga, searchManga, scanLibrary };
});