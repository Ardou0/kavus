<script setup lang="ts">
import { ref, watch } from 'vue';
import { useSettingsStore } from '../stores/settingsStore';
import { t } from '../i18n';

const settingsStore = useSettingsStore();

// Local form bindings initialized to store settings
const startMinimized = ref(settingsStore.settings.start_minimized);
const logLevel = ref(settingsStore.settings.log_level);
const theme = ref(settingsStore.settings.theme);

// Keep form synchronized if store updates (e.g. once async boot load finishes)
watch(
  () => settingsStore.settings,
  (newVal) => {
    startMinimized.value = newVal.start_minimized;
    logLevel.value = newVal.log_level;
    theme.value = newVal.theme;
  },
  { deep: true }
);

const handleSaveSettings = async () => {
  try {
    await settingsStore.saveSettings({
      start_minimized: startMinimized.value,
      log_level: logLevel.value,
      theme: theme.value,
    });
  } catch (err) {
    console.error('Failed to save settings:', err);
  }
};
</script>

<template>
  <div class="flex-1 p-4 overflow-y-auto space-y-4 bg-background">
    <!-- Header -->
    <div class="border-b border-outline-variant pb-2">
      <h2 class="text-ui-header text-lg text-primary tracking-tight font-semibold">
        {{ t.settings.title }}
      </h2>
      <p class="text-ui-body text-xs text-on-surface-variant">
        {{ t.settings.subtitle }}
      </p>
    </div>

    <!-- Settings Form -->
    <div class="max-w-2xl bg-surface-container border border-outline-variant p-5 rounded-md space-y-4">
      <h3 class="text-ui-header text-sm text-primary font-medium border-b border-outline-variant pb-1.5">
        {{ t.settings.generalTitle }}
      </h3>

      <div class="space-y-4">
        <!-- Toggle: Start Minimized -->
        <div class="flex items-center justify-between">
          <div class="space-y-0.5">
            <div class="text-xs text-on-surface font-semibold">{{ t.settings.startMinimized }}</div>
            <div class="text-[11px] text-on-surface-variant">{{ t.settings.startMinimizedDesc }}</div>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input type="checkbox" v-model="startMinimized" :disabled="settingsStore.isLoading" class="sr-only peer">
            <div class="w-9 h-5 bg-surface-dim peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-on-surface-variant after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-primary peer-checked:after:bg-on-primary"></div>
          </label>
        </div>

        <hr class="border-outline-variant" />

        <!-- Select: Log Level -->
        <div class="flex items-center justify-between">
          <div class="space-y-0.5">
            <div class="text-xs text-on-surface font-semibold">{{ t.settings.logLevel }}</div>
            <div class="text-[11px] text-on-surface-variant">{{ t.settings.logLevelDesc }}</div>
          </div>
          <select
            v-model="logLevel"
            :disabled="settingsStore.isLoading"
            class="bg-surface-dim border border-outline-variant rounded px-2.5 py-1 text-xs text-on-surface font-mono outline-none focus:border-primary transition-colors cursor-pointer disabled:opacity-50"
          >
            <option value="debug">DEBUG</option>
            <option value="info">INFO</option>
            <option value="warn">WARNING</option>
            <option value="error">ERROR</option>
          </select>
        </div>

        <hr class="border-outline-variant" />

        <!-- Select: Theme -->
        <div class="flex items-center justify-between">
          <div class="space-y-0.5">
            <div class="text-xs text-on-surface font-semibold">{{ t.settings.theme }}</div>
            <div class="text-[11px] text-on-surface-variant">{{ t.settings.themeDesc }}</div>
          </div>
          <select
            v-model="theme"
            :disabled="settingsStore.isLoading"
            class="bg-surface-dim border border-outline-variant rounded px-2.5 py-1 text-xs text-on-surface font-mono outline-none focus:border-primary transition-colors cursor-pointer disabled:opacity-50"
          >
            <option value="dark">KAVUS DARK (Slate/Carbon)</option>
            <option value="light">LIGHT MODE (Classic)</option>
          </select>
        </div>
      </div>

      <div class="flex justify-end pt-2">
        <button
          @click="handleSaveSettings"
          :disabled="settingsStore.isLoading"
          class="bg-primary hover:bg-secondary text-on-primary text-xs font-semibold px-4 py-2 rounded transition-all cursor-pointer font-mono disabled:opacity-50"
        >
          {{ settingsStore.isLoading ? t.settings.saving : t.settings.btnSave }}
        </button>
      </div>
    </div>
  </div>
</template>
