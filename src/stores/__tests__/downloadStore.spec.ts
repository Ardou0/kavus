import { setActivePinia, createPinia } from 'pinia';
import { describe, beforeEach, it, expect, vi } from 'vitest';
import { useDownloadStore } from '../downloadStore';

// Mock Tauri invoke & event listener
const mockInvoke = vi.fn();
const mockListen = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: any[]) => mockInvoke(...args),
}));
vi.mock('@tauri-apps/api/event', () => ({
  listen: (...args: any[]) => mockListen(...args),
}));

describe('Download Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockInvoke.mockReset();
    mockListen.mockReset();
  });

  // Asserts the download state triggers empty structures on mount
  it('should initialize with default states', () => {
    const store = useDownloadStore();
    expect(store.history).toEqual([]);
    expect(store.activeDownload).toBeNull();
    expect(store.isLoading).toBe(false);
  });

  // Asserts that model download logs are pulled via get_download_history IPC
  it('should load download history from backend', async () => {
    const mockLogs = [
      { id: 'qwen-7b', model_name: 'Qwen 7B', file_path: '/path', status: 'completed', size_bytes: 4500000000, timestamp: 123 }
    ];
    mockInvoke.mockResolvedValueOnce(mockLogs);

    const store = useDownloadStore();
    await store.loadHistory();

    expect(mockInvoke).toHaveBeenCalledWith('get_download_history');
    expect(store.history).toEqual(mockLogs);
  });

  // Asserts that startDownload triggers active progress flags and schedules backend download
  it('should start model download', async () => {
    mockInvoke.mockResolvedValueOnce(null);

    const store = useDownloadStore();
    await store.startDownload('qwen-7b');

    expect(store.activeDownload).not.toBeNull();
    expect(store.activeDownload?.id).toBe('qwen-7b');
    expect(mockInvoke).toHaveBeenCalledWith('start_model_download', { modelName: 'qwen-7b' });
  });

  // Asserts that deleteModel commands deletion of model weights and refreshes history logs
  it('should delete model from local disk', async () => {
    const emptyLogs: any[] = [];
    mockInvoke.mockResolvedValueOnce(emptyLogs);

    const store = useDownloadStore();
    await store.deleteModel('qwen-7b');

    expect(mockInvoke).toHaveBeenCalledWith('delete_downloaded_model', { id: 'qwen-7b' });
    expect(store.history).toEqual(emptyLogs);
  });
});
