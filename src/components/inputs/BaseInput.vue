<script setup lang="ts">
interface Props {
  modelValue: string | number | boolean;
  type?: 'text' | 'number' | 'password' | 'email' | 'url' | 'checkbox';
  placeholder?: string;
  disabled?: boolean;
  min?: number;
  max?: number;
  id?: string;
}

const props = withDefaults(defineProps<Props>(), {
  type: 'text',
  placeholder: '',
  disabled: false
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: string | number | boolean): void;
  (e: 'change', value: string | number | boolean): void;
  (e: 'input', value: string | number | boolean): void;
}>();

const handleInput = (event: Event) => {
  const target = event.target as HTMLInputElement;
  let val: string | number | boolean = target.value;
  if (props.type === 'checkbox') {
    val = target.checked;
  } else if (props.type === 'number') {
    val = target.value === '' ? '' : Number(target.value);
  }
  emit('update:modelValue', val);
  emit('input', val);
};

const handleChange = (event: Event) => {
  const target = event.target as HTMLInputElement;
  let val: string | number | boolean = target.value;
  if (props.type === 'checkbox') {
    val = target.checked;
  } else if (props.type === 'number') {
    val = target.value === '' ? '' : Number(target.value);
  }
  emit('update:modelValue', val);
  emit('change', val);
};
</script>

<template>
  <div class="relative inline-block w-full sm:w-auto">
    <!-- Checkbox / Toggle Mode -->
    <label v-if="type === 'checkbox'" class="relative inline-flex items-center cursor-pointer select-none">
      <input
        type="checkbox"
        :id="id"
        :checked="Boolean(modelValue)"
        :disabled="disabled"
        @change="handleChange"
        class="sr-only peer"
      />
      <div
        class="relative w-9 h-5 bg-surface-dim hover:bg-surface-container-high border border-outline-variant peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-4 peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-on-surface-variant after:border-outline-variant after:border after:rounded-full after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-primary peer-checked:after:bg-on-primary peer-checked:border-primary disabled:opacity-50"
      ></div>
    </label>

    <!-- Standard Input Fields (text, number, etc.) -->
    <input
      v-else
      :id="id"
      :type="type"
      :value="modelValue as string | number"
      :placeholder="placeholder"
      :disabled="disabled"
      :min="min"
      :max="max"
      @input="handleInput"
      @change="handleChange"
      class="w-full bg-surface-dim hover:bg-surface-container border border-outline-variant hover:border-outline rounded px-2.5 py-1.5 text-xs text-on-surface font-mono outline-none focus:border-primary transition-all disabled:opacity-50"
    />
  </div>
</template>
