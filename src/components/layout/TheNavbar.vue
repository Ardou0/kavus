<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useProjectStore } from '../../stores/projectStore';
import { useDownloadStore } from '../../stores/downloadStore';
import { useModalStore } from '../../stores/modalStore';
import { appService } from '../../services/appService';

withDefaults(
  defineProps<{
    appName?: string;
  }>(),
  {
    appName: 'KAVUS',
  }
);

const projectStore = useProjectStore();
const downloadStore = useDownloadStore();
const modalStore = useModalStore();
const router = useRouter();

const minimize = () => appService.minimizeWindow();
const toggleMaximize = () => appService.toggleMaximizeWindow();
const close = async () => {
  if (downloadStore.activeDownload) {
    const confirmed = await modalStore.confirm(
      'A model download is currently in progress.\nAre you sure you want to abort the download and close the application?',
      'Abort Download & Exit'
    );
    if (!confirmed) return;
  }
  appService.closeWindow();
};

const dropdownOpen = ref(false);
let lastClickTime = 0;

const currentProjectName = computed(() => {
  return projectStore.currentProject?.name || 'Select Workspace...';
});

// Sort projects by last_opened descending and take the top 5
const recentProjects = computed(() => {
  return [...projectStore.projects]
    .sort((a, b) => b.last_opened - a.last_opened)
    .slice(0, 5);
});

const toggleDropdown = () => {
  dropdownOpen.value = !dropdownOpen.value;
};

const handleSelectProject = async (project: any) => {
  dropdownOpen.value = false;
  await projectStore.selectProject(project);
};

const handleManageProjects = () => {
  dropdownOpen.value = false;
  router.push('/projects');
};

const handleClickOutside = (event: MouseEvent) => {
  const el = document.getElementById('project-selector-container');
  if (el && !el.contains(event.target as Node)) {
    dropdownOpen.value = false;
  }
};

// Check for double click manually on the titlebar area
const handleMouseDownTitlebar = (event: MouseEvent) => {
  // Ignore clicks on interactive controls
  if (
    event.button !== 0 ||
    (event.target as HTMLElement).closest('button') ||
    (event.target as HTMLElement).closest('#project-selector-container')
  ) {
    return;
  }

  const now = Date.now();
  const timeSinceLastClick = now - lastClickTime;

  if (timeSinceLastClick < 300) {
    appService.toggleMaximizeWindow();
    lastClickTime = 0;
  } else {
    lastClickTime = now;
    appService.startDrag();
  }
};

onMounted(() => {
  window.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  window.removeEventListener('click', handleClickOutside);
});
</script>

<template>
  <header
    @mousedown="handleMouseDownTitlebar"
    class="flex items-center justify-between h-9 bg-surface border-b border-outline-variant select-none px-3 cursor-default"
  >
    <!-- Left: App Name / Logo -->
    <div class="flex items-center gap-2 pointer-events-none">
      <span class="text-ui-header tracking-wider font-semibold text-primary">
        {{ appName }}
      </span>
    </div>

    <!-- Middle: Workspace Name / Selector -->
    <div id="project-selector-container" class="relative">
      <div
        @click="toggleDropdown"
        class="flex items-center gap-1.5 bg-surface-container hover:bg-surface-container-high hover:border-primary/50 px-3 py-0.5 rounded border border-outline-variant cursor-pointer transition-all select-none"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="2"
          stroke="currentColor"
          class="w-3.5 h-3.5 text-on-surface-variant"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M2.25 12.75V12A9 9 0 0 1 12 3v0a9 9 0 0 1 9 9v.75m-18 0A2.25 2.25 0 0 0 5.25 15h13.5A2.25 2.25 0 0 0 21 12.75m-18 0V12a9 9 0 0 1 9-9v0a9 9 0 0 1 9 9v.75m-18 0a2.25 2.25 0 0 0 2.25 2.25h13.5A2.25 2.25 0 0 0 21 12.75M9 19.5h6"
          />
        </svg>
        <span class="text-display-mono text-on-surface-variant font-medium text-xs">
          {{ currentProjectName }}
        </span>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="2.5"
          stroke="currentColor"
          class="w-3 h-3 text-on-surface-variant/70 transition-transform duration-200"
          :class="{ 'rotate-180': dropdownOpen }"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
        </svg>
      </div>

      <!-- Context Dropdown Menu -->
      <div
        v-if="dropdownOpen"
        class="absolute left-1/2 -translate-x-1/2 mt-1 w-64 bg-surface-container-high border border-outline-variant rounded shadow-2xl z-[999] py-1 select-none overflow-hidden animate-fade-in"
      >
        <div class="px-3 py-1.5 text-[9px] text-on-surface-variant/50 font-mono border-b border-outline-variant uppercase tracking-wider">
          Recent Workspaces
        </div>
        
        <!-- Recents List -->
        <div class="max-h-56 overflow-y-auto">
          <div
            v-for="project in recentProjects"
            :key="project.id"
            @click="handleSelectProject(project)"
            class="flex items-center justify-between px-3 py-2 hover:bg-surface-container-highest cursor-pointer group transition-colors"
          >
            <div class="flex flex-col min-w-0 pr-2">
              <span
                class="text-xs font-medium text-on-surface group-hover:text-primary transition-colors truncate"
              >
                {{ project.name }}
              </span>
              <span class="text-[9px] text-on-surface-variant/60 font-mono truncate">
                {{ project.path }}
              </span>
            </div>
            
            <!-- Checkmark for active -->
            <svg
              v-if="projectStore.currentProject?.id === project.id"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              stroke-width="2.5"
              stroke="currentColor"
              class="w-3.5 h-3.5 text-primary flex-shrink-0"
            >
              <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
            </svg>
          </div>
          
          <div
            v-if="recentProjects.length === 0"
            class="px-3 py-4 text-center text-xs text-on-surface-variant/50 font-mono"
          >
            No workspaces open.
          </div>
        </div>

        <div class="border-t border-outline-variant mt-1 pt-1">
          <div
            @click="handleManageProjects"
            class="flex items-center gap-2 px-3 py-2 hover:bg-surface-container-highest text-primary cursor-pointer text-xs font-medium transition-colors font-mono"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              stroke-width="2"
              stroke="currentColor"
              class="w-3.5 h-3.5"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75"
              />
            </svg>
            Manage Projects...
          </div>
        </div>
      </div>
    </div>

    <!-- Right: Frameless Window Controls -->
    <div class="flex items-center gap-1">
      <!-- Minimize -->
      <button
        @click="minimize"
        class="flex items-center justify-center w-6 h-6 rounded hover:bg-surface-container text-on-surface-variant hover:text-on-surface transition-colors cursor-pointer outline-none border border-transparent hover:border-outline-variant"
        title="Minimize"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="2"
          stroke="currentColor"
          class="w-3.5 h-3.5"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 12h-15" />
        </svg>
      </button>

      <!-- Maximize / Restore -->
      <button
        @click="toggleMaximize"
        class="flex items-center justify-center w-6 h-6 rounded hover:bg-surface-container text-on-surface-variant hover:text-on-surface transition-colors cursor-pointer outline-none border border-transparent hover:border-outline-variant"
        title="Maximize"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="2"
          stroke="currentColor"
          class="w-3 h-3"
        >
          <rect x="4" y="4" width="16" height="16" rx="1.5" />
        </svg>
      </button>

      <!-- Close -->
      <button
        @click="close"
        class="flex items-center justify-center w-6 h-6 rounded hover:bg-error-container text-on-surface-variant hover:text-white transition-colors cursor-pointer outline-none border border-transparent hover:border-error"
        title="Close"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="2"
          stroke="currentColor"
          class="w-3.5 h-3.5"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  </header>
</template>
