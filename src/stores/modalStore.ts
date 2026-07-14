import { defineStore } from 'pinia';
import { ref } from 'vue';
import { t } from '../i18n';

export type ModalType = 'info' | 'confirm' | 'warning' | 'error';

export interface ModalOptions {
  title?: string;
  message: string;
  type?: ModalType;
  okLabel?: string;
  cancelLabel?: string;
}

export const useModalStore = defineStore('modal', () => {
  const isOpen = ref(false);
  const title = ref('');
  const message = ref('');
  const type = ref<ModalType>('info');
  const okLabel = ref('');
  const cancelLabel = ref('');

  let resolvePromise: ((value: boolean) => void) | null = null;

  const show = (options: ModalOptions): Promise<boolean> => {
    return new Promise((resolve) => {
      resolvePromise = resolve;
      title.value = options.title || getDefaultTitle(options.type);
      message.value = options.message;
      type.value = options.type || 'info';
      okLabel.value = options.okLabel || t.common.ok;
      cancelLabel.value = options.cancelLabel || t.common.cancel;
      isOpen.value = true;
    });
  };

  const confirm = (message: string, title?: string): Promise<boolean> => {
    return show({ type: 'confirm', message, title });
  };

  const info = (message: string, title?: string): Promise<boolean> => {
    return show({ type: 'info', message, title });
  };

  const warning = (message: string, title?: string): Promise<boolean> => {
    return show({ type: 'warning', message, title });
  };

  const error = (message: string, title?: string): Promise<boolean> => {
    return show({ type: 'error', message, title });
  };

  const accept = () => {
    isOpen.value = false;
    if (resolvePromise) {
      resolvePromise(true);
      resolvePromise = null;
    }
  };

  const cancel = () => {
    isOpen.value = false;
    if (resolvePromise) {
      resolvePromise(false);
      resolvePromise = null;
    }
  };

  const getDefaultTitle = (tType?: ModalType): string => {
    switch (tType) {
      case 'confirm':
        return t.common.confirm;
      case 'warning':
        return t.common.warning;
      case 'error':
        return t.common.error;
      default:
        return t.common.info;
    }
  };

  return {
    isOpen,
    title,
    message,
    type,
    okLabel,
    cancelLabel,
    show,
    confirm,
    info,
    warning,
    error,
    accept,
    cancel,
  };
});
