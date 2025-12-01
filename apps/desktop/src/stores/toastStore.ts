import { create } from 'zustand';

export type ToastType = 'success' | 'error';

interface Toast {
  message: string;
  type: ToastType;
}

interface ToastState {
  toast: Toast | null;
  showToast: (message: string, type: ToastType) => void;
  hideToast: () => void;
}

export const useToastStore = create<ToastState>((set) => ({
  toast: null,
  showToast: (message, type) => {
    set({ toast: { message, type } });
    setTimeout(() => {
      set({ toast: null });
    }, 3000);
  },
  hideToast: () => set({ toast: null }),
}));
