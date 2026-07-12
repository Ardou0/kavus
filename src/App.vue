<script setup lang="ts">
import { onMounted } from 'vue';
import TheNavbar from './components/layout/TheNavbar.vue';
import TheSidebar from './components/layout/TheSidebar.vue';
import TheFooter from './components/layout/TheFooter.vue';
import BaseModal from './components/base/BaseModal.vue';
import { appService } from './services/appService';
import { useAppStore } from './stores/appStore';

const appStore = useAppStore();

onMounted(async () => {
  await appService.initialize();
});
</script>

<template>
  <div class="flex flex-col h-screen w-screen bg-background text-on-surface overflow-hidden">
    <!-- Top Navbar -->
    <TheNavbar :app-name="appStore.appName" />

    <!-- Middle Content Area (Sidebar + View Panel) -->
    <div class="flex flex-1 overflow-hidden">
      <!-- Sidebar -->
      <TheSidebar />

      <!-- Main Router Outlet Area -->
      <main class="flex-1 flex flex-col overflow-hidden bg-background border-outline-variant">
        <RouterView v-slot="{ Component }">
          <KeepAlive>
            <component :is="Component" />
          </KeepAlive>
        </RouterView>
      </main>
    </div>

    <!-- Bottom Status Bar -->
    <TheFooter />

    <!-- Global Modal Dialog System -->
    <BaseModal />
  </div>
</template>

<style>
/* Prevent default text selection across the UI for an app-like feel */
html, body {
  margin: 0;
  padding: 0;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  user-select: none;
  background-color: var(--color-background);
}
</style>