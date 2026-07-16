<script setup lang="ts">
import { ref, watch } from 'vue';
import { useSettingsStore } from '../../stores/settingsStore';
import { t } from '../../i18n';
import BaseCard from '../base/BaseCard.vue';
import BaseSelect from '../inputs/BaseSelect.vue';
import BaseToggle from '../inputs/BaseToggle.vue';

const settingsStore = useSettingsStore();

const startMinimized = ref(settingsStore.settings.start_minimized);
const theme = ref(settingsStore.settings.theme);

const themeOptions = [
  { value: 'dark', label: 'KAVUS DARK (Slate/Carbon)' },
  { value: 'light', label: 'LIGHT MODE (Classic)' }
];

watch(
  () => settingsStore.settings,
  (newVal) => {
    startMinimized.value = newVal.start_minimized;
    theme.value = newVal.theme;
  },
  { deep: true }
);

const handleSave = async () => {
  try {
    await settingsStore.saveSettings({
      ...settingsStore.settings,
      start_minimized: startMinimized.value,
      theme: theme.value,
    });
  } catch (err) {
    console.error('Failed to auto-save general settings:', err);
  }
};
</script>

<template>
  <BaseCard :title="t.settings.generalTitle" :loading="settingsStore.isLoading">
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <div class="space-y-0.5">
          <div class="text-xs text-on-surface font-semibold">{{ t.settings.startMinimized }}</div>
          <div class="text-[11px] text-on-surface-variant">{{ t.settings.startMinimizedDesc }}</div>
        </div>
        <BaseToggle
          v-model="startMinimized"
          @change="handleSave"
          :disabled="settingsStore.isLoading"
        />
      </div>

      <hr class="border-outline-variant" />

      <div class="flex items-center justify-between">
        <div class="space-y-0.5">
          <div class="text-xs text-on-surface font-semibold">{{ t.settings.theme }}</div>
          <div class="text-[11px] text-on-surface-variant">{{ t.settings.themeDesc }}</div>
        </div>
        <BaseSelect
          v-model="theme"
          @change="handleSave"
          :disabled="settingsStore.isLoading"
          :options="themeOptions"
        />
      </div>
    </div>
  </BaseCard>
</template>
