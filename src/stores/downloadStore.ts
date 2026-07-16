import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface DownloadLogEntry {
  id: string;
  model_name: string;
  file_path: string;
  status: string; // "downloading", "completed", "failed", "deleted"
  size_bytes: number;
  timestamp: number;
}

export interface ActiveDownload {
  id: string;
  model_name: string;
  progress: number;
  speed_mbps: number;
  downloaded_bytes: number;
  total_bytes: number;
  status: string;
}

export const useDownloadStore = defineStore('download', () => {
  const history = ref<DownloadLogEntry[]>([]);
  const activeDownload = ref<ActiveDownload | null>(null);
  const isLoading = ref(false);

  const loadHistory = async () => {
    isLoading.value = true;
    try {
      history.value = await invoke<DownloadLogEntry[]>('get_download_history');
    } catch (err) {
      console.error('Failed to load download history:', err);
    } finally {
      isLoading.value = false;
    }
  };

  const startDownload = async (modelName: string) => {
    try {
      activeDownload.value = {
        id: modelName,
        model_name: modelName,
        progress: 0,
        speed_mbps: 0,
        downloaded_bytes: 0,
        total_bytes: 4800000000,
        status: 'downloading',
      };
      await invoke('start_model_download', { modelName });
    } catch (err) {
      console.error('Failed to start model download:', err);
      activeDownload.value = null;
    }
  };

  const deleteModel = async (id: string) => {
    isLoading.value = true;
    try {
      history.value = await invoke<DownloadLogEntry[]>('delete_downloaded_model', { id });
    } catch (err) {
      console.error('Failed to unregister/delete model entry:', err);
    } finally {
      isLoading.value = false;
    }
  };

  let unlistenProgress: (() => void) | null = null;

  const setupListener = async () => {
    if (unlistenProgress) return;
    unlistenProgress = await listen<ActiveDownload>('download-progress', (event) => {
      const payload = event.payload;
      if (payload.status === 'completed' || payload.status === 'failed') {
        activeDownload.value = null;
        loadHistory();
      } else {
        activeDownload.value = payload;
      }
    });
  };

  return {
    history,
    activeDownload,
    isLoading,
    loadHistory,
    startDownload,
    deleteModel,
    setupListener,
  };
});
