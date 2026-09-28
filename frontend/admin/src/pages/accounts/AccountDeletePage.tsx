import { useEffect, useId, useRef, useState } from "react";
import type { MouseEvent, SyntheticEvent } from "react";
import { createPortal } from "react-dom";
import { AccountsError, accountsService } from "../../services/accounts";
import { navigateTo, usePathname } from "../../services/navigation";
import type { AdministratorAccount } from "../../types/account";
import { resolveAccountReturnTo } from "./accountRoutes";
import "./account-delete.css";

type DeleteAction = "soft" | "hard";

type LoadState =
  | { kind: "invalid"; accountId: string }
  | { kind: "loading" }
  | { kind: "ready"; account: AdministratorAccount }
  | { kind: "not-found"; accountId: string }
  | { kind: "error"; error: AccountsError };

interface MutationError {
  action: DeleteAction;
  message: string;
}

interface AccountDeleteConfirmationDialogProps {
  open: boolean;
  action: DeleteAction | null;
  targetLabel: string;
  accountId: string;
  isPending: boolean;
  errorMessage: string | null;
  onCancel: () => void;
  onConfirm: () => void;
}

const ACCOUNT_ID_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

function getAccountId(pathname: string): string | null {
  const match = pathname.match(/^\/admin\/accounts\/([^/]+)\/delete$/);

  if (match === null) {
    return null;
  }

  try {
    return decodeURIComponent(match[1]);
  } catch {
    return match[1];
  }
}

function getAccountLabel(account: AdministratorAccount): string {
  return account.displayName ?? account.email;
}

function getStatusLabel(account: AdministratorAccount): string {
  if (account.deletedAt !== null) {
    return "Soft-deleted";
  }

  return account.status === "active" ? "Active" : "Inactive";
}

function formatDate(value: string): string {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}

function getLoadErrorMessage(error: AccountsError): string {
  if (error.problemCode === "INVALID_ACCOUNT_ID") {
    return "The account identifier is invalid.";
  }

  switch (error.code) {
    case "FORBIDDEN":
      return "You do not have permission to view this administrator account.";
    case "NETWORK":
      return "The administrator account could not be loaded. Check your connection and try again.";
    case "SERVER":
      return "The account service is temporarily unavailable. Please try again.";
    case "VALIDATION":
      return "The account request was rejected. Please try again.";
    case "INVALID_RESPONSE":
      return "The account service returned an invalid response.";
    case "UNAUTHORIZED":
      return "Your session is no longer valid.";
    default:
      return "The administrator account could not be loaded. Please try again.";
  }
}

function getMutationErrorMessage(
  action: DeleteAction,
  error: AccountsError,
): string {
  switch (error.problemCode) {
    case "ACCOUNT_ALREADY_DELETED":
      return "That administrator account has already been soft-deleted. No second soft deletion was performed.";
    case "LAST_ACTIVE_ADMINISTRATOR":
      return "The last active administrator cannot be deleted.";
    case "ACCOUNT_NOT_FOUND":
      return "That administrator account could not be found. It may already have been removed.";
    default:
      break;
  }

  if (error.code === "FORBIDDEN") {
    return action === "soft"
      ? "You do not have permission to soft-delete this administrator account."
      : "You do not have permission to permanently delete this administrator account.";
  }

  switch (error.code) {
    case "CONFLICT":
      return "The administrator account changed before the deletion completed. Review the current account state and try again.";
    case "NOT_FOUND":
      return "The administrator account could not be found. It may already have been removed.";
    case "NETWORK":
      return "We could not reach the account service. No deletion result was received.";
    case "SERVER":
      return "The account service failed while processing the deletion. Please try again.";
    case "VALIDATION":
      return "The server rejected the deletion request. Please try again.";
    case "INVALID_RESPONSE":
      return "The account service returned an invalid deletion response.";
    case "UNAUTHORIZED":
      return "Your session is no longer valid.";
    default:
      return "The administrator account could not be deleted. Please try again.";
  }
}

function makeBodyInertExcept(excluded: HTMLElement): () => void {
  const previousState = Array.from(document.body.children)
    .filter((child): child is HTMLElement => child !== excluded)
    .map((child) => ({ child, inert: child.inert }));

  for (const { child } of previousState) {
    child.inert = true;
  }

  return () => {
    for (const { child, inert } of previousState) {
      child.inert = inert;
    }
  };
}

function getFocusableElements(dialog: HTMLElement): HTMLElement[] {
  return Array.from(
    dialog.querySelectorAll<HTMLElement>(
      [
        "button:not([disabled])",
        "[href]",
        "input:not([disabled])",
        "select:not([disabled])",
        "textarea:not([disabled])",
        '[tabindex]:not([tabindex="-1"])',
      ].join(","),
    ),
  );
}

function AccountDeleteConfirmationDialog({
  open,
  action,
  targetLabel,
  accountId,
  isPending,
  errorMessage,
  onCancel,
  onConfirm,
}: AccountDeleteConfirmationDialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const cancelButtonRef = useRef<HTMLButtonElement>(null);
  const errorRef = useRef<HTMLParagraphElement>(null);
  const onCancelRef = useRef(onCancel);
  const isPendingRef = useRef(isPending);
  const titleId = useId();
  const descriptionId = useId();
  const pendingStatusId = useId();

  onCancelRef.current = onCancel;
  isPendingRef.current = isPending;

  useEffect(() => {
    const dialog = dialogRef.current;

    if (dialog === null || !open) {
      return;
    }

    let fallbackCleanup: (() => void) | null = null;

    if (!dialog.open) {
      if (typeof dialog.showModal === "function") {
        dialog.showModal();
      } else {
        dialog.setAttribute("open", "");
        fallbackCleanup = makeBodyInertExcept(dialog);
      }
    }

    cancelButtonRef.current?.focus();

    function handleKeyDown(event: globalThis.KeyboardEvent): void {
      if (event.key === "Escape") {
        event.preventDefault();

        if (!isPendingRef.current) {
          onCancelRef.current();
        }

        return;
      }

      if (event.key !== "Tab") {
        return;
      }

      const focusable = getFocusableElements(dialog);

      if (focusable.length === 0) {
        event.preventDefault();
        return;
      }

      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      const active = document.activeElement;

      if (event.shiftKey && active === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && active === last) {
        event.preventDefault();
        first.focus();
      }
    }

    dialog.addEventListener("keydown", handleKeyDown);

    return () => {
      dialog.removeEventListener("keydown", handleKeyDown);
      fallbackCleanup?.();

      if (dialog.open && typeof dialog.close === "function") {
        dialog.close();
      } else {
        dialog.removeAttribute("open");
      }
    };
  }, [open]);

  useEffect(() => {
    if (errorMessage !== null) {
      errorRef.current?.focus();
    }
  }, [errorMessage]);

  function handleNativeCancel(event: SyntheticEvent<HTMLDialogElement>): void {
    event.preventDefault();

    if (!isPendingRef.current) {
      onCancelRef.current();
    }
  }

  if (action === null) {
    return null;
  }

  const isSoftDelete = action === "soft";
  const title = isSoftDelete
    ? "Delete administrator account?"
    : "Permanently delete administrator account?";
  const description = isSoftDelete
    ? `This will soft-delete ${targetLabel} (Account ID ${accountId}). The account can be restored later.`
    : `This permanently deletes ${targetLabel} (Account ID ${accountId}). This action cannot be undone.`;

  return createPortal(
    <dialog
      ref={dialogRef}
      className="account-delete-dialog"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby={titleId}
      aria-describedby={descriptionId}
      onCancel={handleNativeCancel}
    >
      <div className="account-delete-dialog__content">
        <header className="account-delete-dialog__header">
          <p className="eyebrow">Confirmation required</p>
          <h2 id={titleId}>{title}</h2>
          <p id={descriptionId}>{description}</p>
        </header>

        {errorMessage !== null ? (
          <p
            ref={errorRef}
            className="account-delete-dialog__error"
            role="alert"
            tabIndex={-1}
          >
            {errorMessage}
          </p>
        ) : null}

        {isPending ? (
          <p
            id={pendingStatusId}
            className="account-delete-dialog__status"
            role="status"
            aria-live="polite"
            aria-atomic="true"
          >
            {isSoftDelete
              ? "Soft-deleting administrator account…"
              : "Permanently deleting administrator account…"}
          </p>
        ) : null}

        <div className="account-delete-dialog__actions">
          <button
            ref={cancelButtonRef}
            className="secondary-button account-delete-dialog__button"
            type="button"
            disabled={isPending}
            onClick={() => onCancelRef.current()}
          >
            Cancel
          </button>

          <button
            className="account-delete-dialog__button account-delete-dialog__button--danger"
            type="button"
            disabled={isPending}
            aria-describedby={isPending ? pendingStatusId : undefined}
            onClick={onConfirm}
          >
            {isPending
              ? "Deleting…"
              : isSoftDelete
                ? "Delete Account"
                : "Permanently Delete"}
          </button>
        </div>
      </div>
    </dialog>,
    document.body,
  );
}

export function AccountDeletePage() {
  const pathname = usePathname();
  const accountId = getAccountId(pathname);
  const returnTo = resolveAccountReturnTo(window.location.search);

  const [loadState, setLoadState] = useState<LoadState>(() => {
    if (accountId === null) {
      return { kind: "invalid", accountId: "" };
    }

    if (!ACCOUNT_ID_PATTERN.test(accountId)) {
      return { kind: "invalid", accountId };
    }

    return { kind: "loading" };
  });

  const [dialogAction, setDialogAction] = useState<DeleteAction | null>(null);
  const [pendingAction, setPendingAction] = useState<DeleteAction | null>(null);
  const [mutationError, setMutationError] = useState<MutationError | null>(
    null,
  );
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const softDeleteTriggerRef = useRef<HTMLButtonElement>(null);
  const hardDeleteTriggerRef = useRef<HTMLButtonElement>(null);
  const activeTriggerRef = useRef<HTMLElement | null>(null);
  const previousDialogOpenRef = useRef(false);
  const mutationInFlightRef = useRef(false);
  const loadErrorRef = useRef<HTMLDivElement>(null);

  const titleId = useId();
  const summaryTitleId = useId();
  const actionsTitleId = useId();
  const successStatusId = useId();

  useEffect(() => {
    if (accountId === null || !ACCOUNT_ID_PATTERN.test(accountId)) {
      setLoadState({ kind: "invalid", accountId: accountId ?? "" });
      return;
    }

    const controller = new AbortController();

    setLoadState({ kind: "loading" });
    setMutationError(null);
    setSuccessMessage(null);

    accountsService
      .get(accountId, { signal: controller.signal })
      .then((account) => {
        if (controller.signal.aborted) {
          return;
        }

        setLoadState({ kind: "ready", account });
      })
      .catch((error: unknown) => {
        if (controller.signal.aborted) {
          return;
        }

        if (error instanceof AccountsError && error.code === "UNAUTHORIZED") {
          return;
        }

        const normalized =
          error instanceof AccountsError
            ? error
            : new AccountsError("UNKNOWN", 0);

        if (
          normalized.problemCode === "INVALID_ACCOUNT_ID" ||
          (normalized.code === "VALIDATION" &&
            normalized.problemCode === "INVALID_ACCOUNT_ID")
        ) {
          setLoadState({ kind: "invalid", accountId });
          return;
        }

        if (
          normalized.code === "NOT_FOUND" &&
          (normalized.problemCode === null ||
            normalized.problemCode === "ACCOUNT_NOT_FOUND")
        ) {
          setLoadState({ kind: "not-found", accountId });
          return;
        }

        setLoadState({ kind: "error", error: normalized });
      });

    return () => controller.abort();
  }, [accountId]);

  useEffect(() => {
    if (loadState.kind === "error") {
      loadErrorRef.current?.focus();
    }
  }, [loadState]);

  useEffect(() => {
    if (successMessage === null) {
      return;
    }

    navigateTo(returnTo, { replace: true });
  }, [returnTo, successMessage]);

  useEffect(() => {
    const isDialogOpen = dialogAction !== null;

    if (previousDialogOpenRef.current && !isDialogOpen) {
      activeTriggerRef.current?.focus();
    }

    previousDialogOpenRef.current = isDialogOpen;
  }, [dialogAction]);

  function openConfirmation(
    action: DeleteAction,
    event: MouseEvent<HTMLButtonElement>,
  ): void {
    if (mutationInFlightRef.current) {
      return;
    }

    activeTriggerRef.current = event.currentTarget;
    setMutationError(null);
    setDialogAction(action);
  }

  function cancelConfirmation(): void {
    if (pendingAction !== null) {
      return;
    }

    setMutationError(null);
    setDialogAction(null);
  }

  async function confirmDeletion(): Promise<void> {
    const action = dialogAction;

    if (
      action === null ||
      mutationInFlightRef.current ||
      accountId === null ||
      !ACCOUNT_ID_PATTERN.test(accountId)
    ) {
      return;
    }

    mutationInFlightRef.current = true;
    setPendingAction(action);
    setMutationError(null);
    setSuccessMessage(null);

    try {
      if (action === "soft") {
        await accountsService.softDelete(accountId);
      } else {
        await accountsService.hardDelete(accountId);
      }

      setDialogAction(null);
      setPendingAction(null);
      mutationInFlightRef.current = false;

      setSuccessMessage(
        action === "soft"
          ? "Administrator account soft-deleted successfully."
          : "Administrator account permanently deleted successfully.",
      );
    } catch (error: unknown) {
      if (error instanceof AccountsError && error.code === "UNAUTHORIZED") {
        mutationInFlightRef.current = false;
        setPendingAction(null);
        return;
      }

      const normalized =
        error instanceof AccountsError
          ? error
          : new AccountsError("UNKNOWN", 0);

      setPendingAction(null);
      mutationInFlightRef.current = false;

      setMutationError({
        action,
        message: getMutationErrorMessage(action, normalized),
      });
    }
  }

  const targetAccount = loadState.kind === "ready" ? loadState.account : null;

  const targetLabel =
    targetAccount !== null
      ? getAccountLabel(targetAccount)
      : `Account ${accountId}`;

  const isSoftDeleteAvailable =
    loadState.kind === "ready" && loadState.account.deletedAt === null;

  const isHardDeleteAvailable =
    accountId !== null &&
    (loadState.kind === "ready" || loadState.kind === "not-found");

  const isBusy = pendingAction !== null;

  return (
    <section
      className="account-delete-page"
      aria-labelledby={titleId}
      aria-busy={isBusy || loadState.kind === "loading"}
    >
      <a
        className="secondary-button account-delete-page__back"
        href={returnTo}
        onClick={(event) => {
          if (
            event.defaultPrevented ||
            event.button !== 0 ||
            event.metaKey ||
            event.ctrlKey ||
            event.shiftKey ||
            event.altKey
          ) {
            return;
          }

          event.preventDefault();
          navigateTo(returnTo);
        }}
      >
        Back to Administrator Accounts
      </a>

      <header className="account-delete-page__header">
        <p className="eyebrow">Administration</p>

        <h1 id={titleId}>Delete Administrator Account</h1>

        <p>
          Review the authoritative account details before choosing a soft or
          permanent deletion operation.
        </p>
      </header>

      {loadState.kind === "loading" ? (
        <p
          className="account-delete-page__status"
          role="status"
          aria-live="polite"
        >
          Loading administrator account…
        </p>
      ) : null}

      {loadState.kind === "invalid" ? (
        <div
          className="form-alert"
          role="alert"
          tabIndex={-1}
          ref={loadErrorRef}
        >
          <span className="form-alert__icon" aria-hidden="true">
            !
          </span>

          <div>
            <p className="form-alert__title">Invalid account identifier</p>

            <p className="form-alert__message">
              The delete route does not contain a valid Account ID. No
              destructive operation is available.
            </p>
          </div>
        </div>
      ) : null}

      {loadState.kind === "error" ? (
        <div
          className="form-alert"
          role="alert"
          tabIndex={-1}
          ref={loadErrorRef}
        >
          <span className="form-alert__icon" aria-hidden="true">
            !
          </span>

          <div>
            <p className="form-alert__title">
              Unable to load administrator account
            </p>

            <p className="form-alert__message">
              {getLoadErrorMessage(loadState.error)}
            </p>
          </div>
        </div>
      ) : null}

      {loadState.kind === "not-found" ? (
        <div className="account-delete-page__unavailable">
          <h2>Account unavailable</h2>

          <p>
            The standard account lookup did not return this Account. The
            permanent-delete operation remains available so a soft-deleted
            target can be purged explicitly by the backend authorization policy.
          </p>

          <dl className="account-delete-summary">
            <div>
              <dt>Account ID</dt>
              <dd>{loadState.accountId}</dd>
            </div>
          </dl>
        </div>
      ) : null}

      {targetAccount !== null ? (
        <section
          className="account-delete-page__summary"
          aria-labelledby={summaryTitleId}
        >
          <div className="account-delete-page__summary-header">
            <p className="eyebrow">Authoritative Account</p>

            <h2 id={summaryTitleId}>{getAccountLabel(targetAccount)}</h2>
          </div>

          <dl className="account-delete-summary">
            <div>
              <dt>Email</dt>
              <dd>{targetAccount.email}</dd>
            </div>

            {targetAccount.displayName !== null ? (
              <div>
                <dt>Display name</dt>
                <dd>{targetAccount.displayName}</dd>
              </div>
            ) : null}

            <div>
              <dt>Account ID</dt>
              <dd>{targetAccount.id}</dd>
            </div>

            <div>
              <dt>Status</dt>
              <dd>{getStatusLabel(targetAccount)}</dd>
            </div>

            <div>
              <dt>Created</dt>
              <dd>
                <time dateTime={targetAccount.createdAt}>
                  {formatDate(targetAccount.createdAt)}
                </time>
              </dd>
            </div>

            <div>
              <dt>Updated</dt>
              <dd>
                <time dateTime={targetAccount.updatedAt}>
                  {formatDate(targetAccount.updatedAt)}
                </time>
              </dd>
            </div>

            {targetAccount.deletedAt !== null ? (
              <div>
                <dt>Deleted</dt>
                <dd>
                  <time dateTime={targetAccount.deletedAt}>
                    {formatDate(targetAccount.deletedAt)}
                  </time>
                </dd>
              </div>
            ) : null}
          </dl>
        </section>
      ) : null}

      {successMessage !== null ? (
        <p
          id={successStatusId}
          className="account-delete-page__success"
          role="status"
          aria-live="polite"
          aria-atomic="true"
        >
          {successMessage}
        </p>
      ) : null}

      {loadState.kind === "ready" || loadState.kind === "not-found" ? (
        <section
          className="account-delete-page__actions"
          aria-labelledby={actionsTitleId}
        >
          <div>
            <p className="eyebrow">Destructive actions</p>
            <h2 id={actionsTitleId}>Choose the deletion operation</h2>
          </div>

          <div className="account-delete-page__action-list">
            {isSoftDeleteAvailable ? (
              <button
                ref={softDeleteTriggerRef}
                className="account-delete-page__action"
                type="button"
                disabled={isBusy}
                onClick={(event) => openConfirmation("soft", event)}
              >
                Soft-delete account
              </button>
            ) : null}

            {isHardDeleteAvailable ? (
              <button
                ref={hardDeleteTriggerRef}
                className="account-delete-page__action account-delete-page__action--danger"
                type="button"
                disabled={isBusy}
                onClick={(event) => openConfirmation("hard", event)}
              >
                Permanently delete account
              </button>
            ) : null}
          </div>
        </section>
      ) : null}

      <AccountDeleteConfirmationDialog
        open={dialogAction !== null}
        action={dialogAction}
        targetLabel={targetLabel}
        accountId={accountId ?? "unknown"}
        isPending={pendingAction !== null}
        errorMessage={mutationError?.message ?? null}
        onCancel={cancelConfirmation}
        onConfirm={() => void confirmDeletion()}
      />
    </section>
  );
}
