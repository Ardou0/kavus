<script setup lang="ts">
import { computed } from 'vue';

interface SelectOption {
  value: string | number;
  label: string;
}

const props = defineProps<{
  modelValue: string | number;
  options: (SelectOption | string)[];
  disabled?: boolean;
  id?: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string | number): void;
  (e: 'change', value: string | number): void;
}>();

const formattedOptions = computed<SelectOption[]>(() => {
  return props.options.map((opt) => {
    if (typeof opt === 'string') {
      return { value: opt, label: opt.toUpperCase() };
    }
    return opt;
  });
});

const handleChange = (event: Event) => {
  const target = event.target as HTMLSelectElement;
  emit('update:modelValue', target.value);
  emit('change', target.value);
};
</script>

<template>
  <div class="relative inline-block w-full sm:w-auto">
    <select
      :id="id"
      :value="modelValue"
      :disabled="disabled"
      @change="handleChange"
      class="w-full bg-surface-dim hover:bg-surface-container border border-outline-variant hover:border-outline rounded pl-3 pr-8 py-1.5 text-xs text-on-surface font-mono outline-none focus:border-primary transition-all cursor-pointer disabled:opacity-50 appearance-none select-none"
    >
      <option
        v-for="opt in formattedOptions"
        :key="opt.value"
        :value="opt.value"
        class="bg-surface-container-high text-on-surface py-1 font-mono text-xs"
      >
        {{ opt.label }}
      </option>
    </select>
    
    <!-- Custom Chevron Arrow -->
    <div class="absolute inset-y-0 right-0 flex items-center pr-2.5 pointer-events-none text-on-surface-variant">
      <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2.5" stroke="currentColor" class="w-3.5 h-3.5">
        <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
      </svg>
    </div>
  </div>
</template>
