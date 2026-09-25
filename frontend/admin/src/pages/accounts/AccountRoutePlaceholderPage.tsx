import type { MouseEvent } from "react";
import { navigateTo, usePathname } from "../../services/navigation";
import { resolveAccountReturnTo } from "./accountRoutes";

type AccountRouteMode = "create" | "edit";

interface AccountRoutePlaceholderPageProps {
  mode: AccountRouteMode;
}

function getAccountId(pathname: string): string | null {
  const match = pathname.match(/^\/admin\/accounts\/([^/]+)\/edit$/);

  if (match === null) {
    return null;
  }

  try {
    return decodeURIComponent(match[1]);
  } catch {
    return match[1];
  }
}

export function AccountRoutePlaceholderPage({
  mode,
}: AccountRoutePlaceholderPageProps) {
  const pathname = usePathname();
  const returnTo = resolveAccountReturnTo(window.location.search);
  const accountId = mode === "edit" ? getAccountId(pathname) : null;

  const title =
    mode === "create"
      ? "Create Administrator Account"
      : "Edit Administrator Account";

  const description =
    mode === "create"
      ? "The create-account route is available inside the Admin Shell."
      : `The edit-account route is available for account ${accountId ?? "the requested account"}.`;

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
    navigateTo(returnTo);
  }

  return (
    <section className="account-route-page">
      <header className="account-route-page__header">
        <p className="eyebrow">Administration</p>
        <h1 className="account-route-page__title">{title}</h1>
        <p className="account-route-page__description">{description}</p>
      </header>

      <p className="account-route-page__note">
        Account form fields are outside the scope of the account-list route
        work.
      </p>

      <a
        className="secondary-button account-route-page__back"
        href={returnTo}
        onClick={handleBack}
      >
        Back to Administrator Accounts
      </a>
    </section>
  );
}
