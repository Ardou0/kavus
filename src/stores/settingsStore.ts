import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface AppSettings {
  start_minimized: boolean;
  log_level: string;
  theme: string;
}

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<AppSettings>({
    start_minimized: true,
    log_level: 'info',
    theme: 'dark',
  });

  const isLoading = ref(false);

  const loadSettings = async () => {
    isLoading.value = true;
    try {
      settings.value = await invoke<AppSettings>('get_settings');
      applyTheme(settings.value.theme);
    } catch (err) {
      console.error('Failed to load settings:', err);
    } finally {
      isLoading.value = false;
    }
  };

  const saveSettings = async (newSettings: AppSettings) => {
    isLoading.value = true;
    try {
      await invoke('save_settings', { settings: newSettings });
      settings.value = { ...newSettings };
      applyTheme(newSettings.theme);
    } catch (err) {
      console.error('Failed to save settings:', err);
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  const applyTheme = (themeName: string) => {
    const root = document.documentElement;
    if (themeName === 'light') {
      root.classList.add('light-theme');
    } else {
      root.classList.remove('light-theme');
    }
  };

  return {
    settings,
    isLoading,
    loadSettings,
    saveSettings,
  };
});
