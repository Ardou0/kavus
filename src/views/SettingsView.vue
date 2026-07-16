<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import { useRoute } from 'vue-router';
import { t } from '../i18n';
import GeneralSettings from '../components/settings/GeneralSettings.vue';
import WorkspaceSettings from '../components/settings/WorkspaceSettings.vue';
import AutocorrectionSettings from '../components/settings/AutocorrectionSettings.vue';
import DownloadSettings from '../components/settings/DownloadSettings.vue';
import AdvancedSettings from '../components/settings/AdvancedSettings.vue';
import AboutSettings from '../components/settings/AboutSettings.vue';

const route = useRoute();

const sections = ref([
  { id: 'general', label: 'General' },
  { id: 'workspace', label: 'Workspaces' },
  { id: 'autocorrection', label: 'Autocorrection' },
  { id: 'downloads', label: 'Downloads' },
  { id: 'advanced', label: 'Advanced' },
  { id: 'about', label: 'About' },
]);

const activeTab = ref(
  route.query.tab && typeof route.query.tab === 'string' ? route.query.tab : 'general'
);
const layoutMode = ref<'auto' | 'horizontal' | 'vertical'>('auto');
const windowWidth = ref(window.innerWidth);

const activeComponent = computed(() => {
  switch (activeTab.value) {
    case 'general': return GeneralSettings;
    case 'workspace': return WorkspaceSettings;
    case 'autocorrection': return AutocorrectionSettings;
    case 'downloads': return DownloadSettings;
    case 'advanced': return AdvancedSettings;
    case 'about': return AboutSettings;
    default: return GeneralSettings;
  }
});

const isVertical = computed(() => {
  if (layoutMode.value === 'horizontal') return false;
  if (layoutMode.value === 'vertical') return true;
  return windowWidth.value < 768 || sections.value.length > 6;
});

// Horizontal Sliding Pill Position tracking
const navContainerRef = ref<HTMLElement | null>(null);
const activePillStyle = ref({ left: '0px', width: '0px' });

const updateActivePill = () => {
  if (isVertical.value || !navContainerRef.value) return;
  const activeBtn = navContainerRef.value.querySelector(`button[data-tab="${activeTab.value}"]`) as HTMLElement;
  if (activeBtn) {
    activePillStyle.value = {
      left: `${activeBtn.offsetLeft}px`,
      width: `${activeBtn.offsetWidth}px`,
    };
  }
};

watch([activeTab, isVertical], () => {
  nextTick(updateActivePill);
});

const handleResize = () => {
  windowWidth.value = window.innerWidth;
  updateActivePill();
};

onMounted(() => {
  window.addEventListener('resize', handleResize);
  // Deep-link directly to specified tab from route query parameters on mount
  if (route.query.tab && typeof route.query.tab === 'string') {
    activeTab.value = route.query.tab;
  }
  nextTick(updateActivePill);
});

// Reactively switch tabs when the URL query parameters update (e.g. via footer deep links)
watch(
  () => route.query.tab,
  (newTab) => {
    if (newTab && typeof newTab === 'string') {
      activeTab.value = newTab;
    }
  }
);

onUnmounted(() => {
  window.removeEventListener('resize', handleResize);
});

const toggleLayout = () => {
  if (layoutMode.value === 'auto') {
    layoutMode.value = 'horizontal';
  } else if (layoutMode.value === 'horizontal') {
    layoutMode.value = 'vertical';
  } else {
    layoutMode.value = 'auto';
  }
};
</script>

<template>
  <div class="flex-1 p-4 overflow-y-auto space-y-5 bg-background flex flex-col items-center">
    <div class="w-full max-w-3xl space-y-5">
      <!-- Header -->
      <div class="border-b border-outline-variant pb-2 flex items-center justify-between flex-wrap gap-3">
        <div>
          <h2 class="text-ui-header text-lg text-primary tracking-tight font-semibold">
            {{ t.settings.title }}
          </h2>
          <p class="text-ui-body text-xs text-on-surface-variant">
            {{ t.settings.subtitle }}
          </p>
        </div>

        <!-- Layout toggle button -->
        <button
          @click="toggleLayout"
          class="bg-surface hover:bg-surface-container border border-outline-variant hover:border-outline text-on-surface-variant hover:text-white text-xs font-semibold px-2.5 py-1.5 rounded transition-all cursor-pointer font-mono outline-none flex items-center gap-1.5"
          title="Cycle Layout Mode (Auto / Horizontal / Vertical)"
        >
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-3.5 h-3.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75" />
          </svg>
          Layout: {{ layoutMode.toUpperCase() }}
        </button>
      </div>

      <!-- Navigation Tab Bar (Horizontal Mode) -->
      <div v-if="!isVertical" class="flex justify-center">
        <div
          ref="navContainerRef"
          class="relative flex p-1 bg-surface-dim border border-outline-variant rounded-md w-fit items-center"
        >
          <!-- Sliding White Pill Background -->
          <div
            class="absolute top-1 bottom-1 rounded bg-primary transition-all duration-200 ease-out shadow-md"
            :style="{ left: activePillStyle.left, width: activePillStyle.width }"
          ></div>

          <!-- Section Buttons -->
          <button
            v-for="sec in sections"
            :key="sec.id"
            :data-tab="sec.id"
            @click="activeTab = sec.id"
            class="relative z-10 px-4 py-1.5 text-xs font-semibold font-mono uppercase tracking-wider rounded transition-colors duration-150 outline-none cursor-pointer"
            :class="activeTab === sec.id ? 'text-on-primary font-bold' : 'text-on-surface-variant hover:text-on-surface'"
          >
            {{ sec.label }}
          </button>
        </div>
      </div>

      <!-- Settings Content Workspace -->
      <div :class="isVertical ? 'flex flex-col md:flex-row gap-5 justify-center items-start' : 'w-full flex justify-center'">
        <!-- Sidebar Navigation (Vertical Mode) -->
        <div
          v-if="isVertical"
          class="flex flex-col gap-1 w-full md:w-48 bg-surface-container border border-outline-variant p-1.5 rounded-md h-fit shrink-0"
        >
          <button
            v-for="sec in sections"
            :key="sec.id"
            @click="activeTab = sec.id"
            class="text-left px-3 py-2 text-[11px] font-mono uppercase tracking-wider rounded transition-all outline-none cursor-pointer"
            :class="activeTab === sec.id ? 'bg-primary text-on-primary font-bold' : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-dim'"
          >
            {{ sec.label }}
          </button>
        </div>

        <!-- Active Content Card -->
        <div class="flex-1 w-full max-w-2xl">
          <KeepAlive>
            <component :is="activeComponent" />
          </KeepAlive>
        </div>
      </div>
    </div>
  </div>
</template>
