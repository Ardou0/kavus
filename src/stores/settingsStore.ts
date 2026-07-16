import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface AppSettings {
  start_minimized: boolean;
  log_level: string;
  theme: string;
  default_project_path: string;
  show_sandbox_warning: boolean;
  enable_autocorrection: boolean;
  autocorrection_model: string;
  cpu_threads: number;
  gpu_layers: number;
  context_size: number;
  gpu_device_name: string;
  execution_backend: string;
  autocorrection_port: number;
}

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<AppSettings>({
    start_minimized: true,
    log_level: 'info',
    theme: 'dark',
    default_project_path: '',
    show_sandbox_warning: true,
    enable_autocorrection: false,
    autocorrection_model: 'llama3-8b',
    cpu_threads: 4,
    gpu_layers: 0,
    context_size: 2048,
    gpu_device_name: '',
    execution_backend: 'cpu',
    autocorrection_port: 18080,
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

      try {
        const { enable, disable } = await import('@tauri-apps/plugin-autostart');
        if (newSettings.start_minimized) {
          await enable();
        } else {
          await disable();
        }
      } catch (autostartErr) {
        console.error('Failed to sync OS autostart status:', autostartErr);
      }
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
