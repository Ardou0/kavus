import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface ProjectConfig {
  id: string;
  name: string;
  path: string;
  created_at: number;
  last_opened: number;
}

export interface FileSystemItem {
  name: string;
  path: string;
  is_dir: boolean;
  size_bytes?: number;
}

export const useProjectStore = defineStore('projects', () => {
  const projects = ref<ProjectConfig[]>([]);
  const currentProject = ref<ProjectConfig | null>(null);
  const currentFileTree = ref<FileSystemItem[]>([]);
  const isLoading = ref(false);

  const loadProjects = async () => {
    isLoading.value = true;
    try {
      const res = await invoke<{ projects: ProjectConfig[] }>('list_projects');
      projects.value = res.projects;
    } catch (err) {
      console.error('Failed to load projects index:', err);
    } finally {
      isLoading.value = false;
    }
  };

  const pickFolder = async (): Promise<string | null> => {
    try {
      return await invoke<string | null>('pick_project_folder');
    } catch (err) {
      console.error('Failed to pick folder:', err);
      return null;
    }
  };

  const registerProject = async (name: string, path: string) => {
    isLoading.value = true;
    try {
      const newProj = await invoke<ProjectConfig>('add_project', { name, path });
      projects.value.push(newProj);
      return newProj;
    } catch (err) {
      console.error('Failed to add project to index:', err);
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  const unregisterProject = async (id: string) => {
    isLoading.value = true;
    try {
      await invoke('remove_project', { id });
      projects.value = projects.value.filter((p) => p.id !== id);
      if (currentProject.value?.id === id) {
        currentProject.value = null;
        currentFileTree.value = [];
      }
    } catch (err) {
      console.error('Failed to remove project from index:', err);
    } finally {
      isLoading.value = false;
    }
  };

  const selectProject = async (project: ProjectConfig) => {
    try {
      const updatedProject = await invoke<ProjectConfig>('touch_project', { id: project.id });
      const idx = projects.value.findIndex((p) => p.id === project.id);
      if (idx !== -1) {
        projects.value[idx] = updatedProject;
      }
      currentProject.value = updatedProject;
      await loadFileTree(updatedProject.path);
    } catch (err) {
      console.error('Failed to touch project:', err);
      currentProject.value = project;
      await loadFileTree(project.path);
    }
  };

  const loadFileTree = async (path: string) => {
    try {
      currentFileTree.value = await invoke<FileSystemItem[]>('list_project_files', { path });
    } catch (err) {
      console.error('Failed to scan project files:', err);
    }
  };

  const createFolder = async (path: string) => {
    try {
      await invoke('create_project_directory', { path });
      if (currentProject.value) {
        const lastSeparator = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
        const parentPath = lastSeparator !== -1 ? path.substring(0, lastSeparator) : '';
        await loadFileTree(parentPath || currentProject.value.path);
      }
    } catch (err) {
      console.error('Failed to create folder:', err);
      throw err;
    }
  };

  const createFile = async (path: string, content = '') => {
    try {
      await invoke('create_project_file', { path, content });
      if (currentProject.value) {
        const lastSeparator = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
        const parentPath = lastSeparator !== -1 ? path.substring(0, lastSeparator) : '';
        await loadFileTree(parentPath || currentProject.value.path);
      }
    } catch (err) {
      console.error('Failed to create file:', err);
      throw err;
    }
  };

  const deleteItem = async (path: string) => {
    try {
      await invoke('delete_project_item', { path });
      if (currentProject.value) {
        const lastSeparator = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
        const parentPath = lastSeparator !== -1 ? path.substring(0, lastSeparator) : '';
        await loadFileTree(parentPath || currentProject.value.path);
      }
    } catch (err) {
      console.error('Failed to delete item:', err);
      throw err;
    }
  };

  const renameProject = async (id: string, newName: string) => {
    isLoading.value = true;
    try {
      const updatedProj = await invoke<ProjectConfig>('rename_project', { id, newName });
      const idx = projects.value.findIndex((p) => p.id === id);
      if (idx !== -1) {
        projects.value[idx] = updatedProj;
      }
      if (currentProject.value?.id === id) {
        currentProject.value = updatedProj;
      }
    } catch (err) {
      console.error('Failed to rename project:', err);
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  return {
    projects,
    currentProject,
    currentFileTree,
    isLoading,
    loadProjects,
    pickFolder,
    registerProject,
    unregisterProject,
    selectProject,
    renameProject,
    loadFileTree,
    createFolder,
    createFile,
    deleteItem,
  };
});
