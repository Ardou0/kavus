import { setActivePinia, createPinia } from 'pinia';
import { describe, beforeEach, it, expect, vi } from 'vitest';
import { useSettingsStore } from '../settingsStore';

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: any[]) => mockInvoke(...args),
}));

// Mock plugin-autostart
vi.mock('@tauri-apps/plugin-autostart', () => ({
  enable: vi.fn(),
  disable: vi.fn(),
}));

describe('Settings Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockInvoke.mockReset();
  });

  // Asserts the fallback settings match AppSettings schema defaults when store is newly created
  it('should initialize with default states', () => {
    const store = useSettingsStore();
    expect(store.settings.enable_autocorrection).toBe(false);
    expect(store.settings.autocorrection_model).toBe('llama3-8b');
    expect(store.isLoading).toBe(false);
  });

  // Asserts that settings are loaded dynamically via Tauri command get_settings and populated into reactive ref
  it('should load settings from backend', async () => {
    const mockSettings = {
      start_minimized: false,
      log_level: 'debug',
      theme: 'light',
      default_project_path: '/dummy/path',
      show_sandbox_warning: false,
      enable_autocorrection: true,
      autocorrection_model: 'qwen-7b',
      cpu_threads: 6,
      gpu_layers: 20,
      context_size: 4096,
      gpu_device_name: 'RX 7700S',
      execution_backend: 'gpu',
      autocorrection_port: 18080,
    };
    mockInvoke.mockResolvedValueOnce(mockSettings);

    const store = useSettingsStore();
    await store.loadSettings();

    expect(mockInvoke).toHaveBeenCalledWith('get_settings');
    expect(store.settings.theme).toBe('light');
    expect(store.settings.autocorrection_model).toBe('qwen-7b');
    expect(store.settings.cpu_threads).toBe(6);
    expect(store.settings.gpu_layers).toBe(20);
  });

  it('should save settings to backend', async () => {
    const mockSettings = {
      start_minimized: false,
      log_level: 'debug',
      theme: 'dark',
      default_project_path: '/dummy/path',
      show_sandbox_warning: false,
      enable_autocorrection: true,
      autocorrection_model: 'qwen-7b',
      cpu_threads: 6,
      gpu_layers: 20,
      context_size: 4096,
      gpu_device_name: 'RX 7700S',
      execution_backend: 'gpu',
      autocorrection_port: 18080,
    };
    mockInvoke.mockResolvedValueOnce(null);

    const store = useSettingsStore();
    await store.saveSettings(mockSettings);

    expect(mockInvoke).toHaveBeenCalledWith('save_settings', { settings: mockSettings });
    expect(store.settings.theme).toBe('dark');
    expect(store.settings.autocorrection_model).toBe('qwen-7b');
  });
});
