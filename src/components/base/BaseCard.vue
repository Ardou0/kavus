<script setup lang="ts">
defineProps<{
  title?: string;
  titleColor?: string;
  subtitle?: string;
  loading?: boolean;
  loadingText?: string;
}>();
</script>

<template>
  <div class="bg-surface-container border border-outline-variant p-5 rounded-md space-y-4 shadow-sm w-full">
    <!-- Header Area -->
    <div v-if="title || $slots.header || loading" class="flex items-center justify-between border-b border-outline-variant pb-2 flex-wrap gap-2">
      <slot name="header">
        <div>
          <h3 :class="[titleColor || 'text-primary', 'text-ui-header text-sm font-medium']">
            {{ title }}
          </h3>
          <p v-if="subtitle" class="text-[10px] text-on-surface-variant mt-0.5 leading-tight">
            {{ subtitle }}
          </p>
        </div>
      </slot>
      <span v-if="loading" class="text-[9px] text-primary/80 font-mono animate-pulse">
        {{ loadingText || 'Saving...' }}
      </span>
    </div>

    <!-- Content Area -->
    <div class="space-y-4">
      <slot />
    </div>

    <!-- Footer Area -->
    <div v-if="$slots.footer" class="flex justify-end pt-2 border-t border-outline-variant/30">
      <slot name="footer" />
    </div>
  </div>
</template>
