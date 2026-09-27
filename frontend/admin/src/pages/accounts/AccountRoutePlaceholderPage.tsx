import type { MouseEvent } from "react";
import { navigateTo } from "../../services/navigation";

export function AccountRoutePlaceholderPage() {
  function handleBack(event: MouseEvent<HTMLAnchorElement>): void {
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
    navigateTo("/admin/accounts");
  }

  return (
    <section className="account-route-page">
      <header className="account-route-page__header">
        <p className="eyebrow">Administration</p>
        <h1 className="account-route-page__title">
          Create Administrator Account
        </h1>
        <p className="account-route-page__description">
          The create-account route is available inside the Admin Shell.
        </p>
      </header>

      <p className="account-route-page__note">
        Account creation form implementation remains outside this Account
        Management route change.
      </p>

      <a
        className="secondary-button account-route-page__back"
        href="/admin/accounts"
        onClick={handleBack}
      >
        Back to Administrator Accounts
      </a>
    </section>
  );
}
