<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue';
import { useSettingsStore } from '../../stores/settingsStore';
import { useDownloadStore } from '../../stores/downloadStore';
import { useModalStore } from '../../stores/modalStore';
import { invoke } from '@tauri-apps/api/core';
import BaseCard from '../base/BaseCard.vue';
import BaseSelect from '../inputs/BaseSelect.vue';
import BaseInput from '../inputs/BaseInput.vue';

const settingsStore = useSettingsStore();
const downloadStore = useDownloadStore();
const modalStore = useModalStore();

const enableAutocorrection = ref(settingsStore.settings.enable_autocorrection);
const autocorrectionModel = ref(settingsStore.settings.autocorrection_model);
const cpuThreads = ref(settingsStore.settings.cpu_threads);
const gpuLayers = ref(settingsStore.settings.gpu_layers);
const contextSize = ref(settingsStore.settings.context_size);
const gpuDeviceName = ref(settingsStore.settings.gpu_device_name);
const executionBackend = ref(settingsStore.settings.execution_backend);
const autocorrectionPort = ref(settingsStore.settings.autocorrection_port);

const recommendedModel = ref('');
const systemHasGpu = ref(false);
const totalRamGb = ref(8);
const gpuDevicesList = ref<string[]>([]);
const systemDependencies = ref<any[]>([]);
const isCheckingDeps = ref(false);
const availableModels = ref<Array<{ id: string; name: string; size_bytes: number }>>([]);

const cpuCoresMax = ref(8);



const fetchDependenciesStatus = async () => {
  isCheckingDeps.value = true;
  try {
    systemDependencies.value = await invoke<any[]>('check_system_dependencies');
  } catch (err) {
    console.error('Failed to check system dependencies:', err);
  } finally {
    isCheckingDeps.value = false;
  }
};

const handleInstallDependency = async (dep: any) => {
  if (!dep.download_url) return;
  try {
    if (dep.download_url === 'trigger_llama_download') {
      await invoke('download_llama_engine');
      setTimeout(fetchDependenciesStatus, 1500);
    } else {
      await invoke('install_system_dependency', { downloadUrl: dep.download_url });
      setTimeout(fetchDependenciesStatus, 3000);
    }
  } catch (err) {
    console.error('Failed to trigger dependency installation:', err);
  }
};

const executableModelsList = ref<string[]>([]);
const isServerRunning = ref(false);

const checkActiveServerState = async () => {
  try {
    const profile = await invoke<{ is_server_running: boolean }>('check_hardware_performance');
    isServerRunning.value = profile.is_server_running;
  } catch (err) {
    console.error('Failed to check active Llama server status:', err);
  }
};

let serverStatePollTimer: any = null;

onMounted(async () => {
  await downloadStore.loadHistory();
  await downloadStore.setupListener();
  try {
    availableModels.value = await invoke<Array<{ id: string; name: string; size_bytes: number }>>('get_available_models');
  } catch (err) {
    console.error('Failed to load available models:', err);
  }
  await fetchDependenciesStatus();
  try {
    const profile = await invoke<{ total_ram_gb: number; has_gpu: boolean; recommended_model: string; executable_models: string[]; is_server_running: boolean; cpu_cores: number }>('check_hardware_performance');
    recommendedModel.value = profile.recommended_model;
    systemHasGpu.value = profile.has_gpu;
    totalRamGb.value = profile.total_ram_gb;
    executableModelsList.value = profile.executable_models || [];
    isServerRunning.value = profile.is_server_running;
    cpuCoresMax.value = profile.cpu_cores || 8;
    
    // Auto-detect and recommend default GPU layers if first run and GPU detected
    if (systemHasGpu.value && gpuLayers.value === 0 && settingsStore.settings.gpu_layers === 0) {
      gpuLayers.value = 32;
    }
  } catch (err) {
    console.error('Failed to load hardware performance profile:', err);
  }

  try {
    gpuDevicesList.value = await invoke<string[]>('get_gpu_devices');
    await handleSave();
  } catch (err) {
    console.error('Failed to retrieve GPU devices list:', err);
  }

  // Poll server state every 3 seconds to keep UI up to date
  serverStatePollTimer = setInterval(checkActiveServerState, 3000);
});

import { onUnmounted } from 'vue';
onUnmounted(() => {
  if (serverStatePollTimer) {
    clearInterval(serverStatePollTimer);
  }
});

const modelOptions = computed(() => {
  // Filter model options so only executable ones are shown
  const filtered = availableModels.value.filter(m => {
    // If executableModelsList is not populated yet (e.g. initial render), allow showing all
    if (executableModelsList.value.length === 0) return true;
    return executableModelsList.value.includes(m.id);
  });
  
  const sorted = [...filtered].sort((a, b) => a.size_bytes - b.size_bytes);
  return sorted.map(m => {
    const gb = m.size_bytes / 1024 / 1024 / 1024;
    const label = `${m.name} - ${gb.toFixed(1)} GB${recommendedModel.value === m.id ? ' (Recommended)' : ''}`;
    return { value: m.id, label };
  });
});

const getRecommendedModelName = () => {
  const rec = availableModels.value.find(m => m.id === recommendedModel.value);
  return rec ? rec.name : 'Qwen 2.5 Coder (7B)';
};

const contextOptions = [
  { value: 512, label: '512 (Fastest)' },
  { value: 1024, label: '1024 (Balanced)' },
  { value: 2048, label: '2048 (Default)' },
  { value: 4096, label: '4096 (High Context)' },
  { value: 8192, label: '8192 (Max Context)' }
];

const backendOptions = [
  { value: 'cpu', label: 'CPU ONLY (Runs purely on CPU cores)' },
  { value: 'gpu', label: 'GPU ONLY (Offloads all layers to graphics card)' },
  { value: 'hybrid', label: 'HYBRID (Offloads specified layers to GPU + uses CPU cores)' }
];

const gpuDeviceOptions = computed(() => {
  const options = [{ value: '', label: 'Default / First Available GPU' }];
  gpuDevicesList.value.forEach(gpu => {
    options.push({ value: gpu, label: gpu });
  });
  return options;
});

// Check if currently selected model is downloaded
const isModelDownloaded = computed(() => {
  return downloadStore.history.some(
    (e) => e.id === autocorrectionModel.value && e.status === 'completed'
  );
});

// Check if currently selected model is actively downloading
const isModelDownloading = computed(() => {
  return downloadStore.activeDownload?.id === autocorrectionModel.value;
});

const selectedModelInfo = computed(() => {
  return availableModels.value.find(m => m.id === autocorrectionModel.value);
});

watch(
  () => settingsStore.settings,
  (newVal) => {
    // Avoid resetting local reactive forms during an active saving/loading sequence (prevents race conditions)
    if (settingsStore.isLoading) {
      return;
    }
    if (enableAutocorrection.value !== newVal.enable_autocorrection) {
      enableAutocorrection.value = newVal.enable_autocorrection;
    }
    if (autocorrectionModel.value !== newVal.autocorrection_model) {
      autocorrectionModel.value = newVal.autocorrection_model;
    }
    if (cpuThreads.value !== newVal.cpu_threads) {
      cpuThreads.value = newVal.cpu_threads;
    }
    if (gpuLayers.value !== newVal.gpu_layers) {
      gpuLayers.value = newVal.gpu_layers;
    }
    if (contextSize.value !== newVal.context_size) {
      contextSize.value = newVal.context_size;
    }
    if (gpuDeviceName.value !== newVal.gpu_device_name) {
      gpuDeviceName.value = newVal.gpu_device_name;
    }
    if (executionBackend.value !== newVal.execution_backend) {
      executionBackend.value = newVal.execution_backend;
    }
    if (autocorrectionPort.value !== newVal.autocorrection_port) {
      autocorrectionPort.value = newVal.autocorrection_port;
    }
  },
  { deep: true }
);

const handleSave = async () => {
  try {
    await settingsStore.saveSettings({
      ...settingsStore.settings,
      enable_autocorrection: enableAutocorrection.value,
      autocorrection_model: autocorrectionModel.value,
      cpu_threads: Number(cpuThreads.value),
      gpu_layers: Number(gpuLayers.value),
      context_size: Number(contextSize.value),
      gpu_device_name: gpuDeviceName.value,
      execution_backend: executionBackend.value,
      autocorrection_port: Number(autocorrectionPort.value),
    });
  } catch (err) {
    console.error('Failed to auto-save auto-correction settings:', err);
  }
};

const triggerBlockedConfigAlert = async () => {
  await modalStore.confirm(
    "To modify the model, backend mode, or port, you must first turn off the 'Enable Auto-correction' toggle. Once your changes are saved, you can reactivate it.",
    "Configuration Locked"
  );
};

const handleToggleAutocorrection = async () => {
  if (enableAutocorrection.value && !isModelDownloaded.value) {
    enableAutocorrection.value = false;
    await modalStore.confirm(
      `Cannot enable auto-correction. The selected model "${autocorrectionModel.value.toUpperCase()}" is not downloaded yet. Please download it first.`,
      'Model Required'
    );
    return;
  }
  await handleSave();
  await checkActiveServerState();
};

const handleModelChange = async (value: string | number) => {
  if (isServerRunning.value) {
    await triggerBlockedConfigAlert();
    autocorrectionModel.value = settingsStore.settings.autocorrection_model;
    return;
  }
  const newVal = String(value);
  autocorrectionModel.value = newVal;
  
  if (enableAutocorrection.value && !downloadStore.history.some(e => e.id === newVal && e.status === 'completed')) {
    enableAutocorrection.value = false;
  }
  
  await handleSave();
};

const triggerModelDownload = async () => {
  if (downloadStore.activeDownload) return;
  
  const modelInfo = selectedModelInfo.value;
  if (!modelInfo) return;

  const gbSize = (modelInfo.size_bytes / 1024 / 1024 / 1024).toFixed(1);
  const confirmed = await modalStore.confirm(
    `Start downloading "${modelInfo.name}" model (approx. ${gbSize} GB)?`,
    'Confirm Download'
  );
  
  if (confirmed) {
    await downloadStore.startDownload(modelInfo.id);
  }
};

const formatBytes = (bytes: number) => {
  if (!bytes) return '0.00 GB';
  const gb = bytes / 1024 / 1024 / 1024;
  return `${gb.toFixed(2)} GB`;
};
</script>

<template>
  <BaseCard title="Code Auto-correction Preferences" :loading="settingsStore.isLoading">
    <div class="space-y-4">
      
      <!-- Toggle: Enable Auto-correction -->
      <div class="flex items-center justify-between">
        <div class="space-y-0.5">
          <div class="text-xs text-on-surface font-semibold">Enable Auto-correction</div>
          <div class="text-[11px] text-on-surface-variant">Analyze and suggest code corrections locally using Llama models.</div>
        </div>
        <BaseInput
          v-model="enableAutocorrection"
          type="checkbox"
          @change="handleToggleAutocorrection"
          :disabled="settingsStore.isLoading"
        />
      </div>

      <hr class="border-outline-variant" />

      <!-- Select: Correction Model -->
      <div class="flex items-start justify-between gap-5 flex-wrap md:flex-nowrap">
        <div class="space-y-0.5 max-w-md">
          <div class="text-xs text-on-surface font-semibold">Correction Model</div>
          <div class="text-[11px] text-on-surface-variant leading-relaxed">Select the local LLM model to run in the background.</div>
          <p class="text-[10px] text-on-surface-variant/70 italic leading-tight">
            Note: Selecting a new model will download it. To save device space, other models will be deleted once the download finishes.
          </p>
          
          <!-- Recommendation Profile Card -->
          <div class="mt-3 text-[10px] text-on-surface-variant bg-surface-dim/40 border border-outline-variant/30 p-2.5 rounded font-mono leading-relaxed max-w-sm">
            <div class="flex items-center gap-1.5 font-semibold text-primary uppercase text-[9px] mb-1">
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-3.5 h-3.5">
                <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12c0 1.268-.63 2.39-1.593 3.068a3.745 3.745 0 01-1.043 3.296 3.745 3.745 0 01-3.296 1.043A3.745 3.745 0 0112 21c-1.268 0-2.39-.63-3.068-1.593a3.746 3.746 0 01-3.296-1.043 3.745 3.745 0 01-1.043-3.296A3.745 3.745 0 013 12c0-1.268.63-2.39 1.593-3.068a3.745 3.745 0 011.043-3.296 3.746 3.746 0 013.296-1.043A3.746 3.746 0 0112 3c1.268 0 2.39.63 3.068 1.593a3.746 3.746 0 013.296 1.043 3.746 3.746 0 011.043 3.296A3.745 3.745 0 0121 12z" />
              </svg>
              Smart recommendation
            </div>
            Based on your system specs (<span class="text-on-surface font-semibold">{{ systemHasGpu ? 'GPU Enabled' : 'CPU Only' }}</span>, <span class="text-on-surface font-semibold">{{ Math.round(totalRamGb) }} GB RAM</span>), we recommend the <strong class="text-primary">{{ getRecommendedModelName() }}</strong> model.
          </div>
        </div>
        <div class="shrink-0 flex flex-col items-end gap-2 relative">
          <!-- Click interception overlay when server is active -->
          <div 
            v-if="isServerRunning" 
            @click.stop="triggerBlockedConfigAlert" 
            class="absolute inset-0 z-10 cursor-pointer"
          ></div>
          <BaseSelect
            :modelValue="autocorrectionModel"
            @change="handleModelChange"
            :disabled="settingsStore.isLoading || !!downloadStore.activeDownload || isServerRunning"
            :options="modelOptions"
          />
          
          <button
            v-if="!isModelDownloaded && !isModelDownloading"
            @click="triggerModelDownload"
            :disabled="settingsStore.isLoading || isServerRunning"
            class="bg-surface hover:bg-surface-container-high border border-outline-variant hover:border-outline text-primary text-[10px] font-semibold font-mono px-2.5 py-1 rounded transition-all cursor-pointer outline-none disabled:opacity-50"
          >
            Download Model
          </button>
          
          <span v-else-if="isModelDownloaded" class="text-[9px] text-primary/80 font-mono">
            ✓ Model Localized
          </span>
        </div>
      </div>

      <!-- Conditional Active Download Progress inside the card -->
      <template v-if="downloadStore.activeDownload">
        <hr class="border-outline-variant" />
        <div class="space-y-2.5 font-mono bg-surface-dim/40 border border-outline-variant/30 p-3 rounded">
          <div class="flex justify-between text-[10px]">
            <span class="text-primary font-semibold uppercase">Downloading {{ downloadStore.activeDownload!.model_name }}</span>
            <span class="text-on-surface-variant animate-pulse">{{ downloadStore.activeDownload!.speed_mbps }} MB/s</span>
          </div>
          <div class="w-full bg-surface-container-high h-1.5 rounded overflow-hidden">
            <div
              class="bg-primary h-full transition-all duration-300 ease-out"
              :style="{ width: `${downloadStore.activeDownload!.progress}%` }"
            ></div>
          </div>
          <div class="flex justify-between text-[9px] text-on-surface-variant">
            <span>
              {{ formatBytes(downloadStore.activeDownload!.downloaded_bytes) }} / 
              {{ formatBytes(downloadStore.activeDownload!.total_bytes) }}
            </span>
            <span class="text-primary font-bold">
              {{ downloadStore.activeDownload!.progress.toFixed(1) }}%
            </span>
          </div>
        </div>
      </template>

      <!-- Select: Execution Backend -->
      <hr class="border-outline-variant" />
      <div class="flex items-center justify-between">
        <div class="space-y-0.5">
          <div class="text-xs text-on-surface font-semibold">Execution Backend</div>
          <div class="text-[11px] text-on-surface-variant">Select CPU, GPU, or hybrid computation mode.</div>
        </div>
        <div class="relative">
          <!-- Click interception overlay when server is active -->
          <div 
            v-if="isServerRunning" 
            @click.stop="triggerBlockedConfigAlert" 
            class="absolute inset-0 z-10 cursor-pointer"
          ></div>
          <BaseSelect
            :modelValue="executionBackend"
            @change="(val) => { if (isServerRunning) { triggerBlockedConfigAlert(); } else { executionBackend = String(val); handleSave(); } }"
            :disabled="settingsStore.isLoading || isServerRunning"
            :options="backendOptions"
          />
        </div>
      </div>

      <!-- System Dependencies Health Check -->
      <div class="space-y-2 bg-surface-dim border border-outline-variant/60 p-3 rounded-md">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-1.5 text-xs font-semibold text-primary">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2.5" stroke="currentColor" class="w-4 h-4">
              <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12c0 1.268-.63 2.39-1.593 3.068a3.745 3.745 0 01-1.043 3.296 3.745 3.745 0 01-3.296 1.043A3.745 3.745 0 0112 21c-1.268 0-2.39-.63-3.068-1.593a3.746 3.746 0 01-3.296-1.043 3.745 3.745 0 01-1.043-3.296A3.745 3.745 0 013 12c0-1.268.63-2.39 1.593-3.068a3.745 3.745 0 011.043-3.296 3.746 3.746 0 013.296-1.043A3.746 3.746 0 0112 3c1.268 0 2.39.63 3.068 1.593a3.746 3.746 0 013.296 1.043 3.746 3.746 0 011.043 3.296A3.745 3.745 0 0121 12z" />
            </svg>
            <span>Local AI Runtime Dependencies</span>
          </div>
          <button 
            @click="fetchDependenciesStatus"
            :disabled="isCheckingDeps"
            class="text-[10px] text-on-surface-variant/80 hover:text-primary font-mono outline-none cursor-pointer flex items-center gap-1 bg-transparent border-none p-0"
          >
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-3 h-3" :class="{ 'animate-spin': isCheckingDeps }">
              <path stroke-linecap="round" stroke-linejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99" />
            </svg>
            Refresh
          </button>
        </div>

        <div class="space-y-2 mt-1">
          <div 
            v-for="dep in systemDependencies" 
            :key="dep.name"
            class="flex items-start justify-between border border-outline-variant/40 rounded p-2.5 bg-surface-container/40"
          >
            <div class="space-y-0.5 max-w-[70%] text-left">
              <div class="flex items-center gap-1.5 flex-wrap">
                <span class="text-[11px] font-semibold text-on-surface font-mono">{{ dep.name }}</span>
                
                <!-- Badge dynamic styling -->
                <span
                  v-if="dep.status === 'installed'"
                  class="bg-primary/20 text-primary border border-primary/30 px-1.5 py-0.5 rounded text-[8px] font-mono font-semibold uppercase leading-none"
                >
                  Ready
                </span>
                <span
                  v-else-if="dep.status === 'native'"
                  class="bg-success/20 text-success border border-success/30 px-1.5 py-0.5 rounded text-[8px] font-mono font-semibold uppercase leading-none"
                >
                  Native
                </span>
                <span
                  v-else-if="dep.status === 'outdated'"
                  class="bg-error/20 text-error border border-error/30 px-1.5 py-0.5 rounded text-[8px] font-mono font-semibold uppercase leading-none animate-pulse"
                >
                  Update Required
                </span>
                <span
                  v-else
                  class="bg-warning/20 text-warning border border-warning/30 px-1.5 py-0.5 rounded text-[8px] font-mono font-semibold uppercase leading-none animate-pulse"
                >
                  Missing
                </span>
              </div>
              <p class="text-[10px] text-on-surface-variant/80 leading-relaxed">{{ dep.details }}</p>
            </div>

            <!-- Action buttons -->
            <div class="shrink-0 flex items-center justify-end">
              <button
                v-if="(dep.status === 'missing' || dep.status === 'outdated') && dep.download_url"
                @click="handleInstallDependency(dep)"
                class="bg-surface hover:bg-surface-container-high border border-outline-variant hover:border-outline text-primary text-[10px] font-semibold font-mono px-2.5 py-1 rounded transition-all cursor-pointer outline-none"
              >
                Setup Dependency
              </button>
              
              <span v-else-if="dep.status === 'installed' || dep.status === 'native'" class="text-[10px] text-primary/80 font-mono">
                ✓ Active
              </span>
              
              <span v-else-if="dep.name.includes('Llama')" class="text-[9px] text-on-surface-variant/60 font-mono italic">
                Auto-installs with model
              </span>
            </div>
          </div>
        </div>
      </div>

      <hr class="border-outline-variant" />

      <!-- Select: GPU Target Device Name -->
      <div class="flex items-center justify-between" :class="{ 'opacity-50 pointer-events-none': !systemHasGpu || executionBackend === 'cpu' }">
        <div class="space-y-0.5">
          <div class="text-xs text-on-surface font-semibold">GPU Target Device</div>
          <div class="text-[11px] text-on-surface-variant flex items-center gap-1.5 font-sans">
            Select the GPU device to offload layers to.
          </div>
        </div>
        <div class="relative">
          <div 
            v-if="isServerRunning" 
            @click.stop="triggerBlockedConfigAlert" 
            class="absolute inset-0 z-10 cursor-pointer"
          ></div>
          <BaseSelect
            v-model="gpuDeviceName"
            @change="handleSave"
            :disabled="settingsStore.isLoading || !systemHasGpu || executionBackend === 'cpu' || isServerRunning"
            :options="gpuDeviceOptions"
          />
        </div>
      </div>

      <hr class="border-outline-variant" />

      <!-- Select: Context Size -->
      <div class="flex items-center justify-between">
        <div class="space-y-0.5">
          <div class="text-xs text-on-surface font-semibold">Context Size</div>
          <div class="text-[11px] text-on-surface-variant">Max token context window loaded by the model compiler.</div>
        </div>
        <div class="relative">
          <div 
            v-if="isServerRunning" 
            @click.stop="triggerBlockedConfigAlert" 
            class="absolute inset-0 z-10 cursor-pointer"
          ></div>
          <BaseSelect
            v-model="contextSize"
            @change="handleSave"
            :disabled="settingsStore.isLoading || isServerRunning"
            :options="contextOptions"
          />
        </div>
      </div>

      <hr class="border-outline-variant" />

      <!-- Input: Autocorrection Port -->
      <div class="flex items-center justify-between">
        <div class="space-y-0.5 max-w-[60%]">
          <div class="text-xs text-on-surface font-semibold">Server Network Port</div>
          <div class="text-[11px] text-on-surface-variant leading-relaxed">
            Network port on localhost (127.0.0.1) for the llama-server process. (Default: 18080)
          </div>
        </div>
        <div class="relative">
          <div 
            v-if="isServerRunning" 
            @click.stop="triggerBlockedConfigAlert" 
            class="absolute inset-0 z-10 cursor-pointer"
          ></div>
          <BaseInput
            v-model="autocorrectionPort"
            type="number"
            :min="1024"
            :max="65535"
            @input="() => { if (isServerRunning) { triggerBlockedConfigAlert(); autocorrectionPort = settingsStore.settings.autocorrection_port; } }"
            @change="() => { if (isServerRunning) { triggerBlockedConfigAlert(); autocorrectionPort = settingsStore.settings.autocorrection_port; } else { handleSave(); } }"
            :disabled="settingsStore.isLoading || isServerRunning"
            class="w-24"
          />
        </div>
      </div>
    </div>
  </BaseCard>
</template>
