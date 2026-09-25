import { useEffect, useId, useRef } from "react";
import type { RefObject, SyntheticEvent } from "react";

interface ConfirmDialogProps {
  open: boolean;
  title: string;
  description: string;
  confirmLabel: string;
  pendingLabel: string;
  isPending: boolean;
  returnFocusRef: RefObject<HTMLElement | null>;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({
  open,
  title,
  description,
  confirmLabel,
  pendingLabel,
  isPending,
  returnFocusRef,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const cancelButtonRef = useRef<HTMLButtonElement>(null);
  const titleId = useId();
  const descriptionId = useId();

  useEffect(() => {
    const dialog = dialogRef.current;

    if (dialog === null) {
      return;
    }

    if (open) {
      if (!dialog.open) {
        if (typeof dialog.showModal === "function") {
          dialog.showModal();
        } else {
          dialog.setAttribute("open", "");
        }
      }

      cancelButtonRef.current?.focus();
      return;
    }

    if (dialog.open && typeof dialog.close === "function") {
      dialog.close();
    } else {
      dialog.removeAttribute("open");
    }

    returnFocusRef.current?.focus();
  }, [open, returnFocusRef]);

  function handleCancel(event: SyntheticEvent<HTMLDialogElement>): void {
    event.preventDefault();

    if (!isPending) {
      onCancel();
    }
  }

  return (
    <dialog
      ref={dialogRef}
      className="account-dialog"
      aria-labelledby={titleId}
      aria-describedby={descriptionId}
      onCancel={handleCancel}
    >
      <div className="account-dialog__content">
        <header className="account-dialog__header">
          <h2 id={titleId}>{title}</h2>
          <p id={descriptionId}>{description}</p>
        </header>

        <div className="account-dialog__actions">
          <button
            ref={cancelButtonRef}
            className="secondary-button account-dialog__button"
            type="button"
            disabled={isPending}
            onClick={onCancel}
          >
            Cancel
          </button>

          <button
            className="primary-button account-dialog__button"
            type="button"
            disabled={isPending}
            onClick={onConfirm}
          >
            {isPending ? pendingLabel : confirmLabel}
          </button>
        </div>
      </div>
    </dialog>
  );
}
