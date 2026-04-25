<template>
  <div class="reader" @wheel.prevent="onWheel" tabindex="0" ref="readerEl">
    <div class="toolbar">
      <button @click="goBack" class="tb">Back</button>
      <span class="pi">{{ curPage + 1 }} / {{ pages }}</span>
      <div class="modes">
        <button v-for="m in modeList" :key="m.v" @click="mode=m.v"
          :class="['tb', {act:mode===m.v}]" :title="m.t">{{ m.l }}</button>
      </div>
    </div>
    <div class="stage" ref="stageEl">
      <img v-if="imgSrc" :src="imgSrc" :style="imgSty" class="pg"
        @load="onLoad" @mousedown.prevent="mdStart" @mousemove="mdMove"
        @mouseup="mdEnd" @mouseleave="mdEnd" />
      <div v-else class="ld">Loading...</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";

const route = useRoute();
const router = useRouter();
const mid = Number(route.params.id);
const readerEl = ref<HTMLElement | null>(null);
const stageEl = ref<HTMLElement | null>(null);

const pages = ref(0);
const curPage = ref(0);
const imgSrc = ref("");
const natW = ref(0);
const natH = ref(0);

type Mode = "fit-h" | "fit-w" | "auto" | "orig";
const mode = ref<Mode>("auto");
const modeList = [
  { v: "fit-h" as Mode, l: "H", t: "Fit Height" },
  { v: "fit-w" as Mode, l: "W", t: "Fit Width" },
  { v: "auto"  as Mode, l: "A", t: "Auto Fit" },
  { v: "orig"  as Mode, l: "1", t: "Original Size (drag to pan)" },
];

// Drag state for original-size mode
const dragging = ref(false);
const dragX = ref(0);
const dragY = ref(0);
const offX = ref(0);
const offY = ref(0);

function mdStart(e: MouseEvent) {
  if (mode.value !== "orig") return;
  dragging.value = true;
  dragX.value = e.clientX;
  dragY.value = e.clientY;
}
function mdMove(e: MouseEvent) {
  if (!dragging.value) return;
  offX.value += e.clientX - dragX.value;
  offY.value += e.clientY - dragY.value;
  dragX.value = e.clientX;
  dragY.value = e.clientY;
}
function mdEnd() { dragging.value = false; }

const imgSty = computed(() => {
  const st: Record<string, string> = {};
  if (mode.value === "orig") {
    st.maxWidth = "none";
    st.maxHeight = "none";
    st.transform = "translate(" + offX.value + "px," + offY.value + "px)";
    st.cursor = "grab";
    return st;
  }
  const stage = stageEl.value;
  if (!stage || !natW.value || !natH.value) {
    st.objectFit = "contain";
    return st;
  }
  const sw = stage.clientWidth;
  const sh = stage.clientHeight;
  const imgRatio = natW.value / natH.value;
  if (mode.value === "fit-h") {
    st.height = sh + "px";
    st.width = "auto";
  } else if (mode.value === "fit-w") {
    st.width = sw + "px";
    st.height = "auto";
  } else {
    // auto: choose best fit
    const stageRatio = sw / sh;
    if (imgRatio > stageRatio) {
      st.width = sw + "px";
      st.height = "auto";
    } else {
      st.height = sh + "px";
      st.width = "auto";
    }
  }
  return st;
});

function onLoad(e: Event) {
  const img = e.target as HTMLImageElement;
  natW.value = img.naturalWidth;
  natH.value = img.naturalHeight;
}

// Wheel: accumulate delta, flip page at threshold, with 200ms cooldown
let wheelAcc = 0;
let lastWheelTime = 0;
function onWheel(e: WheelEvent) {
  const now = Date.now();
  if (now - lastWheelTime < 200) return;
  wheelAcc += e.deltaY;
  const threshold = 50;
  if (wheelAcc > threshold) { wheelAcc = 0; lastWheelTime = now; nextPage(); }
  else if (wheelAcc < -threshold) { wheelAcc = 0; lastWheelTime = now; prevPage(); }
}

function nextPage() {
  if (curPage.value < pages.value - 1) { curPage.value++; loadPage(); }
}
function prevPage() {
  if (curPage.value > 0) { curPage.value--; loadPage(); }
}

async function loadPage() {
  imgSrc.value = "";
  try {
    const path = await invoke<string>("get_page", { mangaId: mid, page: curPage.value });
    imgSrc.value = convertFileSrc(path);
  } catch (e) { console.error(e); }
  offX.value = 0;
  offY.value = 0;
}

function onKey(e: KeyboardEvent) {
  if (e.key === "ArrowRight" || e.key === "ArrowDown" || e.key === "PageDown") nextPage();
  else if (e.key === "ArrowLeft" || e.key === "ArrowUp" || e.key === "PageUp") prevPage();
  else if (e.key === "Escape") goBack();
}

function goBack() { router.push("/detail/" + mid); }

onMounted(async () => {
  try {
    const list = await invoke<{ index: number; name: string }[]>("list_pages", { mangaId: mid });
    pages.value = list.length;
    loadPage();
  } catch (e) { console.error(e); }
  readerEl.value?.focus();
  window.addEventListener("keydown", onKey);
});

onUnmounted(() => { window.removeEventListener("keydown", onKey); });
</script>

<style scoped>
.reader { height: 100%; display: flex; flex-direction: column; background: #111; outline: none; }
.toolbar {
  height: 36px; display: flex; align-items: center; gap: 8px;
  padding: 0 12px; background: var(--bg2); border-bottom: 1px solid var(--border); flex-shrink: 0;
}
.tb {
  background: var(--bg3); color: var(--text); border: 1px solid var(--border);
  padding: 2px 10px; border-radius: 3px; cursor: pointer; font-size: 12px;
}
.tb:hover { background: var(--accent2); }
.tb.act { background: var(--accent); border-color: var(--accent); }
.pi { color: var(--text); font-size: 13px; min-width: 60px; text-align: center; }
.modes { display: flex; gap: 4px; margin-left: auto; }
.stage {
  flex: 1; display: flex; align-items: center; justify-content: center;
  overflow: hidden; position: relative;
}
.pg { display: block; user-select: none; -webkit-user-drag: none; }
.ld { color: var(--text2); font-size: 18px; }
</style>