import { createContext, useCallback, useContext, useMemo, useState } from 'react';
import type { JSX, ReactNode } from 'react';

/** Severity of a toast message; drives ARIA politeness and styling. */
export type ToastVariant = 'error' | 'info';

/** A single transient notification. */
export interface Toast {
  readonly id: number;
  readonly message: string;
  readonly variant: ToastVariant;
}

/**
 * Stable toast actions. These references never change across renders, so
 * effects may depend on them without re-running when the toast list changes
 * (avoids a hydrate/toast feedback loop in `App`).
 */
export interface ToastActions {
  /** Shows an assertive error toast (`role="alert"`). */
  error(message: string): void;
  /** Shows a polite informational toast. */
  info(message: string): void;
  /** Dismisses a toast by id. */
  dismiss(id: number): void;
}

const ToastActionsContext = createContext<ToastActions | null>(null);

let nextToastId = 0;

/**
 * Provides the toast actions context and renders the live region that
 * announces toasts to assistive technology. Error toasts use `role="alert"`
 * (assertive); info toasts use `role="status"` (polite). The region is always
 * present so screen readers register it before the first toast appears.
 *
 * @param props.children - The subtree that can raise toasts.
 */
export const ToastProvider = ({ children }: { children: ReactNode }): JSX.Element => {
  const [toasts, setToasts] = useState<readonly Toast[]>([]);

  const dismiss = useCallback((id: number): void => {
    setToasts((current) => current.filter((toast) => toast.id !== id));
  }, []);

  const push = useCallback((message: string, variant: ToastVariant): void => {
    nextToastId += 1;
    const toast: Toast = { id: nextToastId, message, variant };
    setToasts((current) => [...current, toast]);
  }, []);

  // Stable across renders: depends only on the stable `push`/`dismiss`.
  const actions = useMemo<ToastActions>(
    () => ({
      error: (message) => push(message, 'error'),
      info: (message) => push(message, 'info'),
      dismiss,
    }),
    [push, dismiss]
  );

  return (
    <ToastActionsContext.Provider value={actions}>
      {children}
      <div className="toast-region" role="region" aria-label="Notifications">
        {toasts.map((toast) => (
          <div
            key={toast.id}
            className="toast"
            role={toast.variant === 'error' ? 'alert' : 'status'}
          >
            {toast.message}
            <button
              type="button"
              onClick={() => dismiss(toast.id)}
              aria-label="Dismiss notification"
            >
              ×
            </button>
          </div>
        ))}
      </div>
    </ToastActionsContext.Provider>
  );
};

/**
 * Accesses the stable toast actions. Must be called within a
 * {@link ToastProvider}.
 * @returns The toast actions ({@link ToastActions}).
 * @throws {Error} If used outside a `ToastProvider`.
 */
export const useToast = (): ToastActions => {
  const context = useContext(ToastActionsContext);
  if (context === null) {
    throw new Error('useToast must be used within a ToastProvider');
  }
  return context;
};
