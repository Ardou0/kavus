<script setup lang="ts">
import BaseCard from '../base/BaseCard.vue';
</script>

<template>
  <BaseCard title="About KAVUS">
    <div class="space-y-4 text-xs leading-relaxed text-on-surface-variant">
      <div class="flex items-center gap-4 bg-surface-dim border border-outline-variant p-4 rounded">
        <!-- Kavus Logo representation -->
        <div class="w-12 h-12 bg-primary/10 border border-primary/30 rounded flex items-center justify-center text-primary font-mono text-xl font-bold">
          K
        </div>
        <div class="space-y-0.5 flex-1">
          <div class="text-sm font-semibold text-on-surface font-mono">KAVUS Desktop Client</div>
          <div class="text-xs">Version 0.1.0 (Stable Base)</div>
          <div class="text-[10px] text-on-surface-variant/60 font-mono">Build: Stable Carbon Series</div>
        </div>
        
        <!-- GitHub Link Button -->
        <a
          href="https://github.com/Ardou0/kavus"
          target="_blank"
          class="bg-surface hover:bg-surface-container-high border border-outline-variant hover:border-outline text-primary text-xs font-semibold px-3 py-1.5 rounded transition-all cursor-pointer font-mono flex items-center gap-1.5 outline-none select-none"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
          </svg>
          Repository
        </a>
      </div>

      <p>
        Kavus is a high-performance local workspace and development orchestrator designed for absolute privacy, speed, and safety. Operating entirely on-device, Kavus indexes projects and configuration details securely using a lightweight sandboxed model.
      </p>

      <div class="space-y-2">
        <h4 class="text-ui-header text-xs text-primary font-semibold">Technical Architecture</h4>
        <div class="grid grid-cols-2 gap-2 text-[11px] font-mono bg-surface-dim/50 border border-outline-variant/40 p-3 rounded">
          <div class="flex justify-between border-b border-outline-variant/20 pb-1">
            <span class="text-on-surface-variant/60">Core Framework</span>
            <span class="text-on-surface">Tauri v2.0</span>
          </div>
          <div class="flex justify-between border-b border-outline-variant/20 pb-1">
            <span class="text-on-surface-variant/60">Backend Engine</span>
            <span class="text-on-surface">Rust Stable</span>
          </div>
          <div class="flex justify-between border-b border-outline-variant/20 pb-1">
            <span class="text-on-surface-variant/60">Frontend Renderer</span>
            <span class="text-on-surface">Vue 3 (Composition)</span>
          </div>
          <div class="flex justify-between border-b border-outline-variant/20 pb-1">
            <span class="text-on-surface-variant/60">State Store</span>
            <span class="text-on-surface">Pinia v3</span>
          </div>
          <div class="flex justify-between md:col-span-2">
            <span class="text-on-surface-variant/60">Style Sheet Engine</span>
            <span class="text-on-surface">Tailwind CSS v4.0</span>
          </div>
        </div>
      </div>

      <div class="text-[10px] text-on-surface-variant/50 pt-2 border-t border-outline-variant">
        &copy; 2026 Kavus Development Group. All rights reserved. Open-source local orchestrator.
      </div>
    </div>
  </BaseCard>
</template>
