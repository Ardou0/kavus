import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useAppStore = defineStore('app', () => {
  const workspaceName = ref('Ardou0/kavus');
  const appStatus = ref('SYSTEM READY');
  const appName = ref('KAVUS');

  const updateWorkspace = (name: string) => {
    workspaceName.value = name;
  };

  const updateStatus = (status: string) => {
    appStatus.value = status.toUpperCase();
  };

  return {
    workspaceName,
    appStatus,
    appName,
    updateWorkspace,
    updateStatus,
  };
});
