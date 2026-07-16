<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { useDownloadStore } from '../stores/downloadStore';
import { useSettingsStore } from '../stores/settingsStore';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { appService } from '../services/appService';
import { t } from '../i18n';

const downloadStore = useDownloadStore();
const settingsStore = useSettingsStore();

const name = ref('');
const greetMsg = ref('');
const isGreeting = ref(false);

// Inference playground variables
const testPrompt = ref('Why is the sky blue? Explain in one short sentence.');
const inferenceResult = ref('');
const isInferring = ref(false);
const inferenceError = ref('');
const inferenceStatus = ref<'idle' | 'running' | 'success' | 'failed'>('idle');

let unlistenInference: (() => void) | null = null;

const activeModel = computed(() => settingsStore.settings.autocorrection_model);

const isModelDownloaded = computed(() => {
  return downloadStore.history.some(
    e => e.id === activeModel.value && e.status === 'completed'
  );
});

onMounted(async () => {
  isGreeting.value = false;
  await downloadStore.loadHistory();
  await settingsStore.loadSettings();

  unlistenInference = await listen<string>('inference-chunk', (event) => {
    inferenceResult.value += event.payload;
  });
});

onUnmounted(() => {
  if (unlistenInference) {
    unlistenInference();
  }
});

const triggerGreet = async () => {
  if (!name.value.trim() || isGreeting.value) return;
  isGreeting.value = true;
  try {
    greetMsg.value = await appService.greet(name.value);
  } catch (err) {
    console.error('Failed to invoke greet command:', err);
  } finally {
    isGreeting.value = false;
  }
};

const runInferenceTest = async () => {
  if (!activeModel.value || !testPrompt.value.trim() || isInferring.value) return;
  
  isInferring.value = true;
  inferenceStatus.value = 'running';
  inferenceResult.value = '';
  inferenceError.value = '';

  try {
    const result = await invoke<string>('run_model_inference', {
      modelName: activeModel.value,
      prompt: testPrompt.value
    });
    // Set the full response as a safety fallback, though it's already updated by stream chunks
    inferenceResult.value = result;
    inferenceStatus.value = 'success';
  } catch (err: any) {
    console.error('Inference diagnostic run failed:', err);
    inferenceError.value = String(err);
    inferenceStatus.value = 'failed';
  } finally {
    isInferring.value = false;
  }
};

const abortInference = async () => {
  try {
    await invoke('abort_model_inference');
  } catch (err) {
    console.error('Failed to abort inference:', err);
  }
};
</script>

<template>
  <div class="flex-1 p-4 overflow-y-auto space-y-5 bg-background">
    
    <!-- Interaction Panel: Rust Interop -->
    <div class="bg-surface-container border border-outline-variant p-4 rounded-md space-y-3">
      <h3 class="text-ui-header text-sm text-primary font-medium">
        {{ t.dashboard.ipcTitle }}
      </h3>
      <p class="text-ui-body text-xs text-on-surface-variant max-w-xl leading-relaxed">
        {{ t.dashboard.ipcDesc }}
      </p>

      <form @submit.prevent="triggerGreet" class="flex gap-2 max-w-md">
        <input
          v-model="name"
          type="text"
          :placeholder="t.dashboard.inputPlaceholder"
          class="flex-1 bg-surface-dim border border-outline-variant rounded px-3 py-1.5 text-xs text-on-surface placeholder:text-on-surface-variant/40 outline-none focus:border-primary transition-colors font-mono"
        />
        <button
          type="submit"
          :disabled="isGreeting"
          class="bg-primary hover:bg-secondary text-on-primary text-xs font-semibold px-4 py-1.5 rounded transition-all cursor-pointer font-mono disabled:opacity-50"
        >
          {{ isGreeting ? 'Calling...' : t.dashboard.btnInvoke }}
        </button>
      </form>

      <div v-if="greetMsg" class="bg-surface-dim border border-outline-variant p-3 rounded text-xs font-mono text-on-surface-variant max-w-md">
        <span class="text-primary font-semibold">{{ t.dashboard.response }}:</span> {{ greetMsg }}
      </div>
    </div>

    <!-- Interaction Panel: Local AI Inference Diagnostic Terminal -->
    <div class="bg-surface-container border border-outline-variant p-4 rounded-md space-y-4">
      <div class="space-y-0.5">
        <h3 class="text-ui-header text-sm text-primary font-medium">
          Local AI Inference Playground
        </h3>
        <p class="text-ui-body text-xs text-on-surface-variant leading-relaxed">
          Verify if Tauri can successfully spawn the local Llama inference process and generate response tokens.
        </p>
      </div>

      <div v-if="!isModelDownloaded" class="bg-surface-dim border border-outline-variant/60 rounded p-5 text-center space-y-2">
        <p class="text-xs text-on-surface-variant font-mono">
          [!] Active LLM model "{{ activeModel.toUpperCase() }}" is not downloaded yet.
        </p>
        <p class="text-[11px] text-on-surface-variant/70 leading-relaxed">
          Go to <strong class="text-primary">Settings > Autocorrection</strong> to download it first.
        </p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-12 gap-5" v-else>
        
        <!-- Controls area (Left) -->
        <div class="md:col-span-5 space-y-3.5">
          <div class="space-y-1 bg-surface-dim border border-outline-variant/40 p-2.5 rounded font-mono text-[10px]">
            <span class="text-on-surface-variant uppercase font-semibold">Active LLM Model</span>
            <div class="text-primary font-bold text-xs pt-0.5">{{ activeModel.toUpperCase() }}</div>
          </div>

          <div class="space-y-1">
            <label class="text-[11px] font-semibold text-on-surface uppercase font-mono">Test Prompt</label>
            <textarea
              v-model="testPrompt"
              :disabled="isInferring"
              rows="3"
              class="w-full text-xs bg-surface-dim text-on-surface border border-outline-variant rounded p-2.5 outline-none focus:border-primary transition-colors font-mono leading-relaxed"
            ></textarea>
          </div>

          <div class="flex gap-2">
            <button
              @click="runInferenceTest"
              :disabled="isInferring || !testPrompt.trim()"
              class="flex-1 bg-primary hover:bg-secondary text-on-primary text-xs font-semibold px-4 py-2 rounded transition-all cursor-pointer font-mono disabled:opacity-40"
            >
              {{ isInferring ? 'Querying llama-server...' : 'Run Diagnostics Test' }}
            </button>
            <button
              v-if="isInferring"
              @click="abortInference"
              class="bg-error hover:bg-red-600 text-on-error text-xs font-semibold px-4 py-2 rounded transition-all cursor-pointer font-mono shrink-0"
            >
              Stop
            </button>
          </div>
        </div>

        <!-- Terminal Output (Right) -->
        <div class="md:col-span-7 flex flex-col space-y-1.5">
          <div class="text-[11px] font-semibold text-on-surface uppercase font-mono flex items-center justify-between">
            <span>Inference Output Console</span>
            
            <!-- Status Pill -->
            <span
              class="text-[9px] font-mono uppercase px-1.5 py-0.5 rounded leading-none"
              :class="{
                'bg-surface-container-high text-on-surface-variant/70': inferenceStatus === 'idle',
                'bg-warning-container text-warning animate-pulse border border-warning/20': inferenceStatus === 'running',
                'bg-primary/20 text-primary border border-primary/30': inferenceStatus === 'success',
                'bg-error-container text-error border border-error/20': inferenceStatus === 'failed',
              }"
            >
              {{ inferenceStatus }}
            </span>
          </div>

          <div class="flex-1 bg-neutral-900 border border-neutral-800 rounded p-4 font-mono text-[11px] min-h-[160px] max-h-[250px] overflow-y-auto leading-relaxed shadow-inner select-text">
            <!-- Idle -->
            <div v-if="inferenceStatus === 'idle'" class="text-neutral-500 italic">
              $ Awaiting inference playground execution trigger...
            </div>

            <!-- Running -->
            <div v-else-if="inferenceStatus === 'running'" class="text-neutral-100 whitespace-pre-wrap leading-relaxed">
              <div class="text-neutral-600 font-bold">$ Querying llama-server stream...</div>
              <div class="text-neutral-100 whitespace-pre-wrap leading-relaxed pt-1">{{ inferenceResult }}</div>
            </div>

            <!-- Success -->
            <div v-else-if="inferenceStatus === 'success'" class="text-green-400 space-y-2">
              <div class="text-neutral-600 font-bold">$ Process exited successfully. Output:</div>
              <div class="text-neutral-100 whitespace-pre-wrap leading-relaxed">{{ inferenceResult }}</div>
            </div>

            <!-- Failed -->
            <div v-else-if="inferenceStatus === 'failed'" class="text-red-400 space-y-2">
              <div class="text-neutral-600 font-bold">$ Process returned error trace:</div>
              <div class="text-red-300 whitespace-pre-wrap leading-relaxed">{{ inferenceError }}</div>
            </div>
          </div>
        </div>

      </div>
    </div>

  </div>
</template>
