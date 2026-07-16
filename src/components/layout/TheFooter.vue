<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useSettingsStore } from '../../stores/settingsStore';
import { useDownloadStore } from '../../stores/downloadStore';

const settingsStore = useSettingsStore();
const downloadStore = useDownloadStore();
const router = useRouter();

const status = ref<'disabled' | 'starting' | 'running' | 'error'>('disabled');
let healthInterval: number | null = null;

const isModelConfigured = computed(() => {
  const modelId = settingsStore.settings.autocorrection_model;
  return downloadStore.history.some(e => e.id === modelId && e.status === 'completed');
});

const checkHealth = async () => {
  if (!settingsStore.settings.enable_autocorrection) {
    status.value = 'disabled';
    return;
  }
  
  if (!isModelConfigured.value) {
    status.value = 'disabled';
    return;
  }

  try {
    const port = settingsStore.settings.autocorrection_port || 18080;
    const res = await fetch(`http://127.0.0.1:${port}/health`);
    if (res.ok) {
      status.value = 'running';
    } else {
      status.value = 'error';
    }
  } catch {
    if (status.value !== 'starting') {
      status.value = 'error';
    }
  }
};

const goToAutocorrectionSettings = () => {
  router.push({ name: 'settings', query: { tab: 'autocorrection' } });
};

onMounted(async () => {
  await downloadStore.loadHistory();
  await checkHealth();
  healthInterval = window.setInterval(checkHealth, 5000);
});

onUnmounted(() => {
  if (healthInterval) clearInterval(healthInterval);
});
</script>

<template>
  <footer class="flex items-center justify-between h-6 bg-surface-dim border-t border-outline-variant px-3 select-none text-[10px] text-on-surface-variant font-mono">
    <!-- Right side: Autocorrection Status clickable redirection -->
    <div 
      @click="goToAutocorrectionSettings"
      class="flex items-center gap-1.5 cursor-pointer hover:text-primary transition-colors pr-1 group"
      title="Click to open Auto-Correction Settings"
    >
      <span 
        class="w-1.5 h-1.5 rounded-full transition-all duration-300"
        :class="{
          'bg-outline-variant': status === 'disabled',
          'bg-warning animate-pulse': status === 'starting',
          'bg-success': status === 'running',
          'bg-error': status === 'error'
        }"
      ></span>
      <span class="text-on-surface/90 group-hover:text-primary transition-colors">
        Auto-Correct: 
        <span v-if="status === 'disabled'" class="text-outline">Idle</span>
        <span v-else-if="status === 'starting'" class="text-warning">Starting...</span>
        <span v-else-if="status === 'running'" class="text-success font-bold">Active</span>
        <span v-else-if="status === 'error'" class="text-error font-bold">Error</span>
      </span>
    </div>
  </footer>
</template>
