import { setActivePinia, createPinia } from 'pinia';
import { describe, beforeEach, it, expect, vi } from 'vitest';
import { useModalStore } from '../modalStore';

vi.mock('../../i18n', () => ({
  t: {
    common: {
      ok: 'OK',
      cancel: 'Cancel',
      confirm: 'Confirm',
      warning: 'Warning',
      error: 'Error',
      info: 'Information',
    },
  },
}));

describe('Modal Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('should initialize with default states', () => {
    const store = useModalStore();
    expect(store.isOpen).toBe(false);
    expect(store.title).toBe('');
    expect(store.message).toBe('');
  });

  it('should open modal with show options', () => {
    const store = useModalStore();
    store.show({
      title: 'Custom Title',
      message: 'Hello World',
      type: 'warning',
    });

    expect(store.isOpen).toBe(true);
    expect(store.title).toBe('Custom Title');
    expect(store.message).toBe('Hello World');
    expect(store.type).toBe('warning');
  });

  it('should resolve true when accepted', async () => {
    const store = useModalStore();
    const promise = store.confirm('Are you sure?');
    
    expect(store.isOpen).toBe(true);
    
    store.accept();
    
    const result = await promise;
    expect(result).toBe(true);
    expect(store.isOpen).toBe(false);
  });

  it('should resolve false when cancelled', async () => {
    const store = useModalStore();
    const promise = store.confirm('Are you sure?');
    
    expect(store.isOpen).toBe(true);
    
    store.cancel();
    
    const result = await promise;
    expect(result).toBe(false);
    expect(store.isOpen).toBe(false);
  });
});
