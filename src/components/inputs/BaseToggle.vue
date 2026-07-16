<script setup lang="ts">
defineProps<{
  modelValue: boolean;
  disabled?: boolean;
  id?: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'change', value: boolean): void;
}>();

const handleToggle = (event: Event) => {
  const target = event.target as HTMLInputElement;
  emit('update:modelValue', target.checked);
  emit('change', target.checked);
};
</script>

<template>
  <label class="relative inline-flex items-center cursor-pointer select-none">
    <input
      type="checkbox"
      :id="id"
      :checked="modelValue"
      :disabled="disabled"
      @change="handleToggle"
      class="sr-only peer"
    />
    <div
      class="relative w-9 h-5 bg-surface-dim hover:bg-surface-container-high border border-outline-variant peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-4 peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-on-surface-variant after:border-outline-variant after:border after:rounded-full after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-primary peer-checked:after:bg-on-primary peer-checked:border-primary disabled:opacity-50"
    ></div>
  </label>
</template>
