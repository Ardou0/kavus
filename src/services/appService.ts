import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../stores/settingsStore';
import { projectService } from './projectService';

export const appService = {
  async exit(): Promise<void> {
    try {
      await invoke('exit_application');
    } catch (err) {
      console.error('Failed to exit application:', err);
    }
  },

  async minimizeWindow(): Promise<void> {
    try {
      await invoke('minimize_application');
    } catch (err) {
      console.error('Failed to minimize window:', err);
    }
  },

  async startDrag(): Promise<void> {
    try {
      await invoke('start_drag');
    } catch (err) {
      console.error('Failed to start drag:', err);
    }
  },

  async toggleMaximizeWindow(): Promise<void> {
    try {
      await invoke('toggle_maximize_application');
    } catch (err) {
      console.error('Failed to toggle maximize window:', err);
    }
  },

  async closeWindow(): Promise<void> {
    try {
      await invoke('hide_to_tray');
    } catch (err) {
      console.error('Failed to close window:', err);
    }
  },

  async greet(name: string): Promise<string> {
    try {
      return await invoke<string>('greet', { name });
    } catch (err) {
      console.error('Failed to invoke greet IPC:', err);
      throw err;
    }
  },

  async initialize(): Promise<void> {
    const settingsStore = useSettingsStore();
    await settingsStore.loadSettings();
    await projectService.initialize();
  }
};
