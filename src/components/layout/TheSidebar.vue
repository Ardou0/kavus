<script setup lang="ts">
import { computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import SidebarButton from '../base/SidebarButton.vue';
import { appService } from '../../services/appService';

const route = useRoute();
const router = useRouter();

const activeView = computed(() => (route.name as string) || 'dashboard');

const navigateTo = (viewName: string) => {
  router.push({ name: viewName });
};

const triggerExit = async () => {
  await appService.exit();
};
</script>

<template>
  <aside class="flex flex-col justify-between w-12 bg-surface-dim border-r border-outline-variant py-2 select-none h-[calc(100vh-2.25rem-1.5rem)]">
    <!-- Top Section (Flows downwards) -->
    <div class="flex flex-col items-center gap-1.5">
      <!-- Dashboard -->
      <SidebarButton
        :active="activeView === 'dashboard'"
        tooltip="Dashboard"
        @click="navigateTo('dashboard')"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M3.75 6A2.25 2.25 0 0 1 6 3.75h2.25A2.25 2.25 0 0 1 10.5 6v2.25a2.25 2.25 0 0 1-2.25 2.25H6a2.25 2.25 0 0 1-2.25-2.25V6ZM3.75 15.75A2.25 2.25 0 0 1 6 13.5h2.25a2.25 2.25 0 0 1 2.25 2.25V18a2.25 2.25 0 0 1-2.25 2.25H6A2.25 2.25 0 0 1 3.75 18v-2.25ZM13.5 6a2.25 2.25 0 0 1 2.25-2.25H18A2.25 2.25 0 0 1 20.25 6v2.25A2.25 2.25 0 0 1 18 10.5h-2.25a2.25 2.25 0 0 1-2.25-2.25V6ZM13.5 15.75a2.25 2.25 0 0 1 2.25-2.25H18a2.25 2.25 0 0 1 2.25 2.25V18A2.25 2.25 0 0 1 18 20.25h-2.25A2.25 2.25 0 0 1 13.5 18v-2.25Z"
        />
      </SidebarButton>
    </div>

    <!-- Bottom Section (Flows upwards) -->
    <div class="flex flex-col items-center gap-1.5">
      <!-- Settings -->
      <SidebarButton
        :active="activeView === 'settings'"
        tooltip="Settings"
        @click="navigateTo('settings')"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.43l-1.003.828c-.293.241-.438.613-.43.992a7.723 7.723 0 0 1 0 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.954.26 1.43l-1.297 2.247a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.43l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 0 1 0-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128c.332-.183.582-.495.645-.869l.214-1.28Z"
        />
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
        />
      </SidebarButton>

      <SidebarButton
        tooltip="Exit Application"
        class="hover:!bg-error-container hover:!border-error hover:!text-white"
        @click="triggerExit"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M5.636 5.636a9 9 0 1 0 12.728 0M12 3v9"
        />
      </SidebarButton>
    </div>
  </aside>
</template>
