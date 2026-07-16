<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useProjectStore } from '../stores/projectStore';
import { useSettingsStore } from '../stores/settingsStore';
import { useModalStore } from '../stores/modalStore';
import { t } from '../i18n';
import BaseCard from '../components/base/BaseCard.vue';

const projectStore = useProjectStore();
const settingsStore = useSettingsStore();
const modalStore = useModalStore();
const router = useRouter();

const projectName = ref('');
const projectPath = ref('');
const errorMessage = ref('');
const isSubmitting = ref(false);

const isRenameModalOpen = ref(false);
const projectToRename = ref<any>(null);
const newNameInput = ref('');
const renameError = ref('');

const vFocus = {
  mounted: (el: HTMLInputElement) => el.focus()
};

onMounted(async () => {
  await projectStore.loadProjects();
  await settingsStore.loadSettings();
  if (settingsStore.settings.default_project_path) {
    projectPath.value = settingsStore.settings.default_project_path;
  }
});

const handlePickFolder = async () => {
  errorMessage.value = '';
  const selectedPath = await projectStore.pickFolder();
  if (selectedPath) {
    projectPath.value = selectedPath;
  }
};

const handleCreateProject = async () => {
  errorMessage.value = '';
  const name = projectName.value.trim();
  const path = projectPath.value.trim();

  if (!name) {
    errorMessage.value = t.errors.projectNameRequired;
    return;
  }
  if (!path) {
    errorMessage.value = t.errors.directoryPathRequired;
    return;
  }

  isSubmitting.value = true;
  try {
    const newProj = await projectStore.registerProject(name, path);
    projectName.value = '';
    projectPath.value = '';
    await projectStore.selectProject(newProj);
    router.push('/dashboard');
  } catch (err: any) {
    errorMessage.value = err?.toString() || t.errors.failedRegister;
  } finally {
    isSubmitting.value = false;
  }
};

const handleSelectProject = async (project: any) => {
  await projectStore.selectProject(project);
  router.push('/dashboard');
};

const handleRemoveProject = async (project: any) => {
  const confirmed = await modalStore.confirm(
    `Are you sure you want to unregister the workspace "${project.name}"?\nThis action only removes the project from the registry and does not delete your local files.`,
    'Unregister Workspace'
  );
  if (confirmed) {
    await projectStore.unregisterProject(project.id);
  }
};

const openRenameModal = (project: any) => {
  projectToRename.value = project;
  newNameInput.value = project.name;
  renameError.value = '';
  isRenameModalOpen.value = true;
};

const closeRenameModal = () => {
  isRenameModalOpen.value = false;
  projectToRename.value = null;
  newNameInput.value = '';
  renameError.value = '';
};

const submitRename = async () => {
  renameError.value = '';
  const newName = newNameInput.value.trim();
  if (!newName) {
    renameError.value = 'Workspace name cannot be empty';
    return;
  }
  if (!projectToRename.value) return;

  try {
    await projectStore.renameProject(projectToRename.value.id, newName);
    closeRenameModal();
  } catch (err: any) {
    renameError.value = err?.toString() || 'Failed to rename workspace';
  }
};

const formatDate = (timestampSec: number) => {
  if (!timestampSec) return 'Never';
  const date = new Date(timestampSec * 1000);
  return date.toLocaleString();
};
</script>

<template>
  <div class="flex-1 p-4 overflow-y-auto space-y-5 bg-background flex flex-col items-center">
    <div class="w-full max-w-6xl space-y-5">
      <!-- Header -->
      <div class="border-b border-outline-variant pb-2">
        <h2 class="text-ui-header text-lg text-primary tracking-tight font-semibold">
          {{ t.projects.title }}
        </h2>
        <p class="text-ui-body text-xs text-on-surface-variant">
          {{ t.projects.subtitle }}
        </p>
      </div>

      <!-- Centered, Wrapping Layout container -->
      <div class="flex flex-wrap gap-5 justify-center items-start w-full">
        
        <!-- Block 1: Register Workspace (Fixed Width 350px) -->
        <div class="w-full md:w-[350px] shrink-0">
          <BaseCard :title="t.projects.registerTitle">
            <form @submit.prevent="handleCreateProject" class="space-y-4">
              <div class="flex flex-col gap-1">
                <label class="text-[10px] text-on-surface-variant uppercase tracking-wider font-mono">{{ t.projects.projectName }}</label>
                <input
                  v-model="projectName"
                  type="text"
                  :placeholder="t.projects.projectNamePlaceholder"
                  :disabled="isSubmitting"
                  class="bg-surface-dim border border-outline-variant rounded px-3 py-1.5 text-xs text-on-surface outline-none focus:border-primary disabled:opacity-50 transition-colors"
                />
              </div>

              <div class="flex flex-col gap-1">
                <label class="text-[10px] text-on-surface-variant uppercase tracking-wider font-mono">{{ t.projects.directoryPath }}</label>
                <div class="flex gap-1.5">
                  <input
                    v-model="projectPath"
                    type="text"
                    :placeholder="t.projects.directoryPathPlaceholder"
                    :disabled="isSubmitting"
                    class="flex-1 bg-surface-dim border border-outline-variant rounded px-3 py-1.5 text-xs text-on-surface outline-none focus:border-primary disabled:opacity-50 transition-colors font-mono"
                  />
                  <button
                    type="button"
                    @click="handlePickFolder"
                    :disabled="isSubmitting"
                    class="bg-surface hover:bg-surface-container border border-outline-variant hover:border-outline text-on-surface text-xs font-semibold px-3 rounded transition-all cursor-pointer flex items-center justify-center outline-none"
                    title="Browse Directory"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-4 h-4">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12h19.5M2.25 12v5.25A2.25 2.25 0 004.5 19.5h15a2.25 2.25 0 002.25-2.25V12M2.25 12V7.5A2.25 2.25 0 014.5 5.25h3.375c.621 0 1.189.252 1.603.662l1.603 1.603a.75.75 0 00.53.22h6.914A2.25 2.25 0 0121.75 10v2M2.25 12h19.5" />
                    </svg>
                  </button>
                </div>
                <span class="text-[9px] text-on-surface-variant/60 font-mono">
                  {{ t.projects.directoryNote }}
                </span>
              </div>

              <!-- Persistent Sandbox Warning Toggle Link -->
              <div v-if="settingsStore.settings.show_sandbox_warning" class="bg-warning-container border border-warning/30 p-3 rounded-md text-on-warning-container text-[11px] leading-relaxed space-y-1">
                <div class="flex items-center gap-1.5 font-semibold text-warning">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-4 h-4 text-warning">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
                  </svg>
                  {{ t.warnings.sandboxTitle }}
                </div>
                <p>
                  {{ t.warnings.sandboxDesc }}
                </p>
              </div>

              <div v-if="errorMessage" class="bg-rose-950/40 border border-rose-500/50 p-2.5 rounded text-rose-400 text-xs font-mono">
                {{ errorMessage }}
              </div>

              <button
                type="submit"
                :disabled="isSubmitting"
                class="w-full bg-primary hover:bg-secondary text-on-primary text-xs font-semibold py-2 rounded transition-all cursor-pointer font-mono disabled:opacity-50"
              >
                {{ isSubmitting ? t.projects.submittingRegister : t.projects.submitRegister }}
              </button>
            </form>
          </BaseCard>
        </div>

        <!-- Block 2: Workspaces List (Flexible Width, Max 800px) -->
        <div class="flex-1 min-w-[320px] max-w-[800px]">
          <BaseCard :title="`${t.projects.listTitle} (${projectStore.projects.length})`">
            <div v-if="projectStore.projects.length === 0" class="text-xs text-on-surface-variant text-center py-8 font-mono">
              {{ t.projects.noProjects }}
            </div>

            <div v-else class="space-y-2.5">
              <div
                v-for="project in projectStore.projects"
                :key="project.id"
                class="flex items-center justify-between border border-outline-variant rounded p-3 transition-colors bg-surface-dim"
                :class="projectStore.currentProject?.id === project.id ? 'border-primary/60 bg-surface-container/20' : 'hover:border-outline'"
              >
                <div class="space-y-1 max-w-[55%]">
                  <div class="flex items-center gap-2 flex-wrap">
                    <span class="text-xs font-semibold text-on-surface">{{ project.name }}</span>
                    <div
                      v-if="projectStore.currentProject?.id === project.id"
                      class="flex items-center justify-center bg-primary/20 text-primary border border-primary/40 px-1.5 py-0.5 rounded text-[9px] font-sans font-semibold uppercase leading-none"
                    >
                      {{ t.projects.activeBadge }}
                    </div>
                  </div>
                  <div class="text-[10px] text-on-surface-variant font-mono truncate" :title="project.path">
                    {{ project.path }}
                  </div>
                  <div class="text-[9px] text-on-surface-variant/60 font-mono">
                    Created: {{ formatDate(project.created_at) }} | Last Opened: {{ formatDate(project.last_opened) }}
                  </div>
                </div>

                <div class="flex items-center gap-1.5 flex-wrap justify-end">
                  <button
                    @click="handleSelectProject(project)"
                    class="bg-surface hover:bg-surface-container-high border border-outline-variant hover:border-outline text-primary text-xs font-semibold px-3 py-1.5 rounded transition-all cursor-pointer font-mono outline-none"
                  >
                    {{ t.projects.btnOpen }}
                  </button>
                  <button
                    @click="openRenameModal(project)"
                    class="bg-surface hover:bg-surface-container-high border border-outline-variant hover:border-outline text-on-surface-variant hover:text-on-surface text-xs font-semibold px-2.5 py-1.5 rounded transition-all cursor-pointer font-mono outline-none"
                  >
                    Rename
                  </button>
                  <button
                    @click="handleRemoveProject(project)"
                    class="hover:bg-error hover:border-error border border-transparent text-on-surface-variant hover:text-on-error text-xs font-semibold px-2.5 py-1.5 rounded transition-all cursor-pointer font-mono outline-none"
                    :title="t.projects.removeTooltip"
                  >
                    {{ t.projects.btnRemove }}
                  </button>
                </div>
              </div>
            </div>
          </BaseCard>
        </div>

      </div>
    </div>

    <!-- Local Workspace Rename Modal -->
    <Transition name="modal-fade">
      <div
        v-if="isRenameModalOpen"
        class="fixed inset-0 flex items-center justify-center z-[9999] p-4 bg-black/60 backdrop-blur-sm"
        @click.self="closeRenameModal"
      >
        <div class="w-full max-w-sm bg-surface-container-high border border-primary/50 rounded shadow-2xl p-4 select-none overflow-hidden">
          <div class="flex items-center gap-2.5 border-b border-outline-variant pb-2">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-5 h-5 text-primary">
              <path stroke-linecap="round" stroke-linejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L6.832 19.82a4.5 4.5 0 01-1.897 1.13l-2.685.8.8-2.685a4.5 4.5 0 011.13-1.897L16.863 4.487zm0 0L19.5 7.125" />
            </svg>
            <span class="text-xs font-semibold uppercase tracking-wider font-mono text-on-surface">
              Rename Workspace
            </span>
          </div>

          <div class="py-4 space-y-3">
            <div class="flex flex-col gap-1.5">
              <label class="text-[10px] text-on-surface-variant uppercase tracking-wider font-mono">New Workspace Name</label>
              <input
                v-model="newNameInput"
                type="text"
                @keyup.enter="submitRename"
                @keyup.esc="closeRenameModal"
                class="bg-surface-dim border border-outline-variant rounded px-3 py-2 text-xs text-on-surface outline-none focus:border-primary transition-colors font-mono w-full"
                v-focus
              />
            </div>
            <div v-if="renameError" class="text-rose-400 text-[10px] font-mono">
              {{ renameError }}
            </div>
          </div>

          <div class="flex items-center justify-end gap-2 pt-2 border-t border-outline-variant">
            <button
              @click="closeRenameModal"
              class="bg-surface hover:bg-surface-container border border-outline-variant hover:border-outline text-on-surface-variant hover:text-white text-xs font-semibold px-3 py-1.5 rounded transition-all cursor-pointer font-mono outline-none"
            >
              Cancel
            </button>
            <button
              @click="submitRename"
              class="bg-primary hover:bg-secondary text-on-primary text-xs font-semibold px-4 py-1.5 rounded transition-all cursor-pointer font-mono outline-none"
            >
              Save
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
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
