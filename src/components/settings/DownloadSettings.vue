<script setup lang="ts">
import { onMounted, computed, ref } from 'vue';
import { useDownloadStore } from '../../stores/downloadStore';
import { useModalStore } from '../../stores/modalStore';
import BaseCard from '../base/BaseCard.vue';

const downloadStore = useDownloadStore();
const modalStore = useModalStore();
const showAll = ref(false);

onMounted(async () => {
  await downloadStore.loadHistory();
  await downloadStore.setupListener();
});

const handleDeleteEntry = async (entry: any) => {
  const confirmed = await modalStore.confirm(
    `Are you sure you want to delete the model file for "${entry.model_name.toUpperCase()}"?\nThis will remove the file from your local storage and free up space.`,
    'Delete Model File'
  );
  if (confirmed) {
    await downloadStore.deleteModel(entry.id);
  }
};

// Sort log entries: most recent first
const sortedHistory = computed(() => {
  return [...downloadStore.history].sort((a, b) => b.timestamp - a.timestamp);
});

// Limit display to 5 unless expanded
const displayedHistory = computed(() => {
  if (showAll.value) {
    return sortedHistory.value;
  }
  return sortedHistory.value.slice(0, 5);
});

// Formatting helpers
const formatBytes = (bytes: number) => {
  if (!bytes) return '0.00 GB';
  const gb = bytes / 1024 / 1024 / 1024;
  return `${gb.toFixed(2)} GB`;
};

const formatDate = (timestampSec: number) => {
  if (!timestampSec) return 'Never';
  const date = new Date(timestampSec * 1000);
  return date.toLocaleString();
};

const activeDownloadProgressPercent = computed(() => {
  if (!downloadStore.activeDownload) return '0%';
  return `${downloadStore.activeDownload.progress.toFixed(1)}%`;
});

const activeDownloadProgressStyle = computed(() => ({
  width: activeDownloadProgressPercent.value
}));
</script>

<template>
  <div class="space-y-5">
    
    <!-- Active Download Card -->
    <BaseCard v-if="downloadStore.activeDownload" title="Active Download Progress">
      <div class="bg-surface-dim border border-outline-variant p-4 rounded-md space-y-3 font-mono">
        <div class="flex items-center justify-between text-xs">
          <span class="font-semibold text-primary uppercase">
            {{ downloadStore.activeDownload.model_name }}
          </span>
          <span class="text-on-surface-variant font-mono animate-pulse">
            {{ downloadStore.activeDownload.speed_mbps }} MB/s
          </span>
        </div>

        <!-- Progress Bar -->
        <div class="w-full bg-surface-container-high h-2 rounded overflow-hidden relative">
          <div
            class="bg-primary h-full transition-all duration-300 ease-out"
            :style="activeDownloadProgressStyle"
          ></div>
        </div>

        <div class="flex items-center justify-between text-[10px] text-on-surface-variant">
          <span>
            {{ formatBytes(downloadStore.activeDownload.downloaded_bytes) }} / 
            {{ formatBytes(downloadStore.activeDownload.total_bytes) }}
          </span>
          <span class="font-bold text-primary">
            {{ activeDownloadProgressPercent }}
          </span>
        </div>
      </div>
    </BaseCard>

    <!-- Downloads Log History -->
    <BaseCard>
      <template #header>
        <div class="flex items-center justify-between pb-2 w-full">
          <div>
            <h3 class="text-ui-header text-sm text-primary font-medium">Asset Logs & Download History</h3>
            <p class="text-[10px] text-on-surface-variant mt-0.5">View and manage local model files.</p>
          </div>
          <span class="text-[10px] text-on-surface-variant/70 font-mono">
            Total: {{ downloadStore.history.length }} record(s)
          </span>
        </div>
      </template>

      <div v-if="downloadStore.history.length === 0" class="text-xs text-on-surface-variant text-center py-8 font-mono">
        No asset download history records found.
      </div>

      <div v-else class="space-y-2.5">
        <div
          v-for="entry in displayedHistory"
          :key="entry.id"
          class="flex items-center justify-between border border-outline-variant rounded p-3 bg-surface-dim"
        >
          <div class="space-y-1 max-w-[65%]">
            <div class="flex items-center gap-2 flex-wrap">
              <span class="text-xs font-semibold text-on-surface uppercase">{{ entry.model_name }}</span>
              
              <!-- Status badges -->
              <span
                v-if="entry.status === 'completed'"
                class="bg-primary/20 text-primary border border-primary/40 px-1.5 py-0.5 rounded text-[8px] font-sans font-semibold uppercase leading-none"
              >
                Completed
              </span>
              <span
                v-else-if="entry.status === 'downloading'"
                class="bg-warning-container text-warning border border-warning/30 px-1.5 py-0.5 rounded text-[8px] font-sans font-semibold uppercase leading-none animate-pulse"
              >
                Downloading
              </span>
              <span
                v-else-if="entry.status === 'deleted'"
                class="bg-surface-container-high text-on-surface-variant/70 border border-outline-variant px-1.5 py-0.5 rounded text-[8px] font-sans font-semibold uppercase leading-none"
              >
                Deleted
              </span>
              <span
                v-else
                class="bg-surface-container-high text-on-surface-variant border border-outline-variant px-1.5 py-0.5 rounded text-[8px] font-sans font-semibold uppercase leading-none"
              >
                {{ entry.status }}
              </span>
            </div>
            <div class="text-[10px] text-on-surface-variant font-mono truncate" :title="entry.file_path">
              {{ entry.file_path }}
            </div>
            <div class="text-[9px] text-on-surface-variant/60 font-mono">
              Completed: {{ formatDate(entry.timestamp) }} | Size: {{ formatBytes(entry.size_bytes) }}
            </div>
          </div>

          <div class="flex items-center gap-1.5">
            <button
              v-if="entry.status === 'completed'"
              @click="handleDeleteEntry(entry)"
              :disabled="downloadStore.isLoading"
              class="hover:bg-error hover:border-error border border-transparent text-on-surface-variant hover:text-on-error text-xs font-semibold px-2.5 py-1.5 rounded transition-all cursor-pointer font-mono outline-none"
              title="Delete asset file"
            >
              Delete File
            </button>
          </div>
        </div>

        <!-- See More / Show Less Toggle Button -->
        <div v-if="sortedHistory.length > 5" class="flex justify-center pt-2 border-t border-outline-variant/30">
          <button
            @click="showAll = !showAll"
            class="text-primary hover:text-secondary font-mono text-xs hover:underline cursor-pointer outline-none bg-transparent border-none py-1.5 px-4 transition-all"
          >
            {{ showAll ? '[ Show Less ]' : `[ See More (${sortedHistory.length - 5} hidden) ]` }}
          </button>
        </div>
      </div>
    </BaseCard>

  </div>
</template>
