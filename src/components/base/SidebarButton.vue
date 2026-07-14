<script setup lang="ts">
defineProps<{
  active?: boolean;
  tooltip?: string;
}>();
</script>

<template>
  <button
    class="relative group flex items-center justify-center w-9 h-9 m-1 rounded-md transition-all duration-200 cursor-pointer border outline-none"
    :class="[
      active
        ? 'bg-secondary-container border-primary text-primary'
        : 'bg-transparent border-transparent text-on-surface-variant hover:bg-surface-container hover:text-on-surface hover:border-outline-variant'
    ]"
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      stroke-width="1.5"
      stroke="currentColor"
      class="w-5 h-5"
    >
      <slot></slot>
    </svg>

    <!-- Tooltip (high density HUD style) -->
    <div
      v-if="tooltip"
      class="absolute left-12 scale-0 group-hover:scale-100 transition-all duration-150 origin-left z-50 ease-in-out bg-surface-container-highest border border-outline text-on-surface text-ui-label-xs px-2 py-1 rounded shadow-lg whitespace-nowrap pointer-events-none"
    >
      {{ tooltip }}
    </div>
  </button>
</template>
