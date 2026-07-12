<script setup lang="ts">
import { ref } from 'vue';
import { appService } from '../services/appService';
import { t } from '../i18n';

const name = ref('');
const greetMsg = ref('');

const triggerGreet = async () => {
  if (!name.value.trim()) return;
  try {
    greetMsg.value = await appService.greet(name.value);
  } catch (err) {
    console.error('Failed to invoke greet command:', err);
  }
};
</script>

<template>
  <div class="flex-1 p-4 overflow-y-auto space-y-4 bg-background">
    <!-- Interaction Panel: Rust Interop -->
    <div class="bg-surface-container border border-outline-variant p-4 rounded-md space-y-3">
      <h3 class="text-ui-header text-sm text-primary font-medium">
        {{ t.dashboard.ipcTitle }}
      </h3>
      <p class="text-ui-body text-xs text-on-surface-variant max-w-xl">
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
          class="bg-primary hover:bg-secondary text-on-primary text-xs font-semibold px-4 py-1.5 rounded transition-all cursor-pointer font-mono"
        >
          {{ t.dashboard.btnInvoke }}
        </button>
      </form>

      <div v-if="greetMsg" class="bg-surface-dim border border-outline-variant p-3 rounded text-xs font-mono text-on-surface-variant max-w-md">
        <span class="text-primary font-semibold">{{ t.dashboard.response }}:</span> {{ greetMsg }}
      </div>
    </div>
  </div>
</template>
