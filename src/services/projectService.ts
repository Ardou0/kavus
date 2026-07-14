import { useProjectStore } from '../stores/projectStore';

export const projectService = {
  async initialize(): Promise<void> {
    const store = useProjectStore();
    await store.loadProjects();
    if (store.projects.length > 0) {
      const lastOpened = [...store.projects].sort((a, b) => b.last_opened - a.last_opened)[0];
      if (lastOpened) {
        await store.selectProject(lastOpened);
      }
    }
  }
};
