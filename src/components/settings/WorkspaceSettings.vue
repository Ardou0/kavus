<script setup lang="ts">
import { ref, watch } from 'vue';
import { useSettingsStore } from '../../stores/settingsStore';
import { useProjectStore } from '../../stores/projectStore';
import BaseCard from '../base/BaseCard.vue';
import BaseInput from '../inputs/BaseInput.vue';

const settingsStore = useSettingsStore();
const projectStore = useProjectStore();

const defaultPath = ref(settingsStore.settings.default_project_path);
const showWarning = ref(settingsStore.settings.show_sandbox_warning);

watch(
  () => settingsStore.settings,
  (newVal) => {
    defaultPath.value = newVal.default_project_path;
    showWarning.value = newVal.show_sandbox_warning;
  },
  { deep: true }
);

const handleSave = async () => {
  try {
    await settingsStore.saveSettings({
      ...settingsStore.settings,
      default_project_path: defaultPath.value.trim(),
      show_sandbox_warning: showWarning.value,
    });
  } catch (err) {
    console.error('Failed to auto-save workspace settings:', err);
  }
};

const handlePickFolder = async () => {
  const selectedPath = await projectStore.pickFolder();
  if (selectedPath) {
    defaultPath.value = selectedPath;
    await handleSave();
  }
};
</script>

<template>
  <BaseCard title="Workspace Preferences" :loading="settingsStore.isLoading">
    <div class="space-y-4">
      <!-- Default Project Path -->
      <div class="flex flex-col gap-1.5">
        <div class="text-xs text-on-surface font-semibold">Default Projects Directory</div>
        <div class="text-[11px] text-on-surface-variant mb-1">Standard root path pre-selected when creating new workspaces.</div>
        <div class="flex gap-1.5 items-center">
          <BaseInput
            v-model="defaultPath"
            type="text"
            placeholder="/home/user/projects"
            @change="handleSave"
            @keyup.enter="handleSave"
            :disabled="settingsStore.isLoading"
            class="flex-1"
          />
          <button
            type="button"
            @click="handlePickFolder"
            :disabled="settingsStore.isLoading"
            class="bg-surface hover:bg-surface-container border border-outline-variant hover:border-outline text-on-surface text-xs font-semibold px-3 py-1.5 rounded transition-all cursor-pointer flex items-center justify-center outline-none"
            title="Browse Directory"
          >
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-4 h-4">
              <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12h19.5M2.25 12v5.25A2.25 2.25 0 004.5 19.5h15a2.25 2.25 0 002.25-2.25V12M2.25 12V7.5A2.25 2.25 0 014.5 5.25h3.375c.621 0 1.189.252 1.603.662l1.603 1.603a.75.75 0 00.53.22h6.914A2.25 2.25 0 0121.75 10v2M2.25 12h19.5" />
            </svg>
          </button>
        </div>
      </div>

      <hr class="border-outline-variant" />

      <!-- Toggle Notice warning -->
      <div class="flex items-center justify-between">
        <div class="space-y-0.5">
          <div class="text-xs text-on-surface font-semibold">Show Sandbox Warning Banner</div>
          <div class="text-[11px] text-on-surface-variant">Display details about the hidden .kavus/ folders on registration.</div>
        </div>
        <BaseInput
          v-model="showWarning"
          type="checkbox"
          @change="handleSave"
          :disabled="settingsStore.isLoading"
        />
      </div>
    </div>
  </BaseCard>
</template>
