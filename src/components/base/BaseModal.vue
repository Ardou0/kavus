<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useModalStore } from '../../stores/modalStore';

const modalStore = useModalStore();

const handleKeyDown = (event: KeyboardEvent) => {
  if (modalStore.isOpen && event.key === 'Escape') {
    modalStore.cancel();
  }
};

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});
</script>

<template>
  <Transition name="modal-fade">
    <div
      v-if="modalStore.isOpen"
      class="fixed inset-0 flex items-center justify-center z-[9999] p-4 bg-black/60 backdrop-blur-sm"
      @click.self="modalStore.cancel"
    >
      <div
        class="w-full max-w-sm bg-surface-container-high border rounded shadow-2xl p-4 select-none overflow-hidden"
        :class="{
          'border-primary/50': modalStore.type === 'info' || modalStore.type === 'confirm',
          'border-amber-500/50': modalStore.type === 'warning',
          'border-error/50': modalStore.type === 'error'
        }"
      >
        <div class="flex items-center gap-2.5 border-b border-outline-variant pb-2">
          <svg
            v-if="modalStore.type === 'info' || modalStore.type === 'confirm'"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="2"
            stroke="currentColor"
            class="w-5 h-5 text-primary"
          >
            <path stroke-linecap="round" stroke-linejoin="round" d="M11.25 11.25l.041-.02a.75.75 0 111.063.852l-.708 2.836a.75.75 0 001.063.852l.041-.021M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-9-3.75h.008v.008H12V8.25z" />
          </svg>
          <svg
            v-else-if="modalStore.type === 'warning'"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="2"
            stroke="currentColor"
            class="w-5 h-5 text-amber-500"
          >
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
          </svg>
          <svg
            v-else-if="modalStore.type === 'error'"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="2"
            stroke="currentColor"
            class="w-5 h-5 text-error"
          >
            <path stroke-linecap="round" stroke-linejoin="round" d="M9.75 9.75l4.5 4.5m0-4.5l-4.5 4.5M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span class="text-xs font-semibold uppercase tracking-wider font-mono text-on-surface">
            {{ modalStore.title }}
          </span>
        </div>

        <div class="py-3 text-xs leading-relaxed text-on-surface-variant font-mono whitespace-pre-wrap">
          {{ modalStore.message }}
        </div>

        <div class="flex items-center justify-end gap-2 pt-2 border-t border-outline-variant">
          <button
            v-if="modalStore.type !== 'info'"
            @click="modalStore.cancel"
            class="bg-surface hover:bg-surface-container border border-outline-variant hover:border-outline text-on-surface text-xs font-semibold px-3 py-1.5 rounded transition-all cursor-pointer font-mono outline-none"
          >
            {{ modalStore.cancelLabel }}
          </button>
          <button
            @click="modalStore.accept"
            class="text-xs font-semibold px-4 py-1.5 rounded transition-all cursor-pointer font-mono outline-none"
            :class="{
              'bg-primary hover:bg-secondary text-on-primary': modalStore.type === 'info' || modalStore.type === 'confirm',
              'bg-amber-500 hover:bg-amber-600 text-black': modalStore.type === 'warning',
              'bg-error hover:bg-error-container text-on-error': modalStore.type === 'error'
            }"
          >
            {{ modalStore.okLabel }}
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 0.15s ease;
}
.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}
.modal-fade-enter-active .w-full {
  transition: transform 0.15s cubic-bezier(0.16, 1, 0.3, 1);
}
.modal-fade-enter-from .w-full {
  transform: scale(0.96);
}
</style>
