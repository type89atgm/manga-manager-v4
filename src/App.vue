<template>
  <div id="app-root">
    <nav class="topbar">
      <div class="nav-left">
        <router-link to="/" class="nav-link">Library</router-link>
      </div>
      <div class="nav-right">
        <button @click="openSettings" class="btn-icon" title="Settings">&#9881;</button>
      </div>
    </nav>
    <main class="main-content">
      <router-view />
    </main>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
function openSettings() {
  router.push({ path: "/", query: { settings: "1" } });
}
</script>

<style scoped>
.topbar {
  height: 40px; background: var(--bg2); display: flex; align-items: center;
  justify-content: space-between; padding: 0 16px; border-bottom: 1px solid var(--border);
}
.nav-link { color: var(--text); text-decoration: none; font-weight: bold; margin-right: 16px; }
.nav-link.router-link-active { color: var(--accent); }
.btn-icon { background: none; border: none; color: var(--text2); font-size: 18px; cursor: pointer; }
.btn-icon:hover { color: var(--accent); }
.main-content { height: calc(100% - 40px); overflow: hidden; background: var(--bg); }
</style>