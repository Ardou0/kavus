<script setup lang="ts">
import { ref, watch } from 'vue';
import { useSettingsStore } from '../../stores/settingsStore';
import { useProjectStore } from '../../stores/projectStore';
import { useModalStore } from '../../stores/modalStore';
import { t } from '../../i18n';
import BaseCard from '../base/BaseCard.vue';
import BaseSelect from '../inputs/BaseSelect.vue';

const settingsStore = useSettingsStore();
const projectStore = useProjectStore();
const modalStore = useModalStore();

const logLevel = ref(settingsStore.settings.log_level);

const logLevelOptions = [
  { value: 'debug', label: 'DEBUG' },
  { value: 'info', label: 'INFO' },
  { value: 'warn', label: 'WARNING' },
  { value: 'error', label: 'ERROR' }
];

watch(
  () => settingsStore.settings,
  (newVal) => {
    logLevel.value = newVal.log_level;
  },
  { deep: true }
);

const handleSave = async () => {
  try {
    await settingsStore.saveSettings({
      ...settingsStore.settings,
      log_level: logLevel.value,
    });
  } catch (err) {
    console.error('Failed to auto-save advanced settings:', err);
  }
};

const handleClearWorkspaces = async () => {
  const confirmed = await modalStore.confirm(
    'Are you sure you want to clear all registered workspaces?\nThis will unregister them from the local database, but it will NOT delete your directories or files.',
    'Clear Workspaces Database'
  );
  
  if (confirmed) {
    for (const project of [...projectStore.projects]) {
      try {
        await projectStore.unregisterProject(project.id);
      } catch (err) {
        console.error('Failed to unregister project during reset:', err);
      }
    }
    await modalStore.confirm('All registered workspaces have been successfully cleared.', 'Reset Completed');
  }
};
</script>

<template>
  <div class="space-y-5">
    <BaseCard :title="t.settings.advancedTitle || 'Advanced Settings'" :loading="settingsStore.isLoading">
      <div class="flex items-center justify-between">
        <div class="space-y-0.5">
          <div class="text-xs text-on-surface font-semibold">{{ t.settings.logLevel }}</div>
          <div class="text-[11px] text-on-surface-variant">{{ t.settings.logLevelDesc }}</div>
        </div>
        <BaseSelect
          v-model="logLevel"
          @change="handleSave"
          :disabled="settingsStore.isLoading"
          :options="logLevelOptions"
        />
      </div>
    </BaseCard>

    <!-- Maintenance Section -->
    <BaseCard title="Maintenance & Diagnostics" titleColor="text-destructive">
      <div class="flex items-center justify-between bg-destructive-container border border-destructive/20 p-3 rounded flex-wrap gap-3">
        <div class="space-y-0.5">
          <div class="text-xs text-on-destructive-container font-semibold">Unregister All Workspaces</div>
          <div class="text-[11px] text-on-surface-variant">Clear all workspace indexing records from the database index.</div>
        </div>
        <button
          @click="handleClearWorkspaces"
          class="hover:bg-error hover:border-error border border-transparent text-on-destructive-container hover:text-on-error text-xs font-semibold px-3 py-1.5 rounded transition-all cursor-pointer font-mono outline-none"
        >
          Reset Registry
        </button>
      </div>
    </BaseCard>
  </div>
</template>
