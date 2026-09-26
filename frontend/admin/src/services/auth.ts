import type { AuthenticationErrorCode, LoginCredentials } from "../types/auth";

interface AuthenticationSession {
  accessToken: string;
  tokenType: string;
  accessTokenExpiresAt: number;
}

type AuthenticationListener = () => void;

export type AuthStatus =
  "unknown" | "authenticated" | "unauthenticated" | "authentication-error";

export type AuthOperation =
  | "bootstrapIdle"
  | "bootstrapPending"
  | "bootstrapRetryPending"
  | "refreshIdle"
  | "refreshing";

export interface AuthenticationSnapshot {
  authStatus: AuthStatus;
  operation: AuthOperation;
}

interface AccessTokenResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
}

const listeners = new Set<AuthenticationListener>();
const apiOrigin = (import.meta.env.VITE_API_ORIGIN ?? "")
  .trim()
  .replace(/\/$/, "");

let authenticationSession: AuthenticationSession | null = null;
let authenticationSnapshot: AuthenticationSnapshot = {
  authStatus: "unknown",
  operation: "bootstrapIdle",
};
let loginRequest: Promise<void> | null = null;
let logoutRequest: Promise<void> | null = null;
let bootstrapRequest: Promise<void> | null = null;
let protectedRefreshRequest: Promise<void> | null = null;
let bootstrapStarted = false;

export class AuthenticationError extends Error {
  readonly code: AuthenticationErrorCode;
  readonly status: number;

  constructor(code: AuthenticationErrorCode, status: number) {
    super(code);
    this.name = "AuthenticationError";
    this.code = code;
    this.status = status;
  }
}

function notifyListeners(): void {
  listeners.forEach((listener) => listener());
}

function setAuthenticationSnapshot(
  authStatus: AuthStatus,
  operation: AuthOperation,
): void {
  if (
    authenticationSnapshot.authStatus === authStatus &&
    authenticationSnapshot.operation === operation
  ) {
    return;
  }

  authenticationSnapshot = { authStatus, operation };
  notifyListeners();
}

function clearAuthenticationSession(): void {
  authenticationSession = null;
}

function buildApiUrl(pathname: string): string {
  if (import.meta.env.PROD && window.location.protocol !== "https:") {
    throw new AuthenticationError("UNKNOWN", 0);
  }

  if (apiOrigin.length === 0) {
    return pathname;
  }

  let origin: URL;

  try {
    origin = new URL(apiOrigin);
  } catch {
    throw new AuthenticationError("UNKNOWN", 0);
  }

  if (import.meta.env.PROD && origin.protocol !== "https:") {
    throw new AuthenticationError("UNKNOWN", 0);
  }

  return new URL(
    pathname,
    `${origin.toString().replace(/\/$/, "")}/`,
  ).toString();
}

function isNonEmptyString(value: unknown): value is string {
  return typeof value === "string" && value.length > 0;
}

function isFiniteNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

function parseAccessTokenResponse(value: unknown): AccessTokenResponse {
  if (
    typeof value !== "object" ||
    value === null ||
    !isNonEmptyString((value as Record<string, unknown>).access_token) ||
    !isNonEmptyString((value as Record<string, unknown>).token_type) ||
    !isFiniteNumber((value as Record<string, unknown>).expires_in)
  ) {
    throw new AuthenticationError("INVALID_RESPONSE", 200);
  }

  return value as AccessTokenResponse;
}

async function readProblemCode(
  response: Response,
): Promise<AuthenticationErrorCode | null> {
  const contentType = response.headers.get("content-type") ?? "";

  if (!contentType.toLowerCase().includes("application/problem+json")) {
    return null;
  }

  try {
    const body: unknown = await response.json();

    if (
      typeof body === "object" &&
      body !== null &&
      typeof (body as Record<string, unknown>).code === "string"
    ) {
      const code = (body as Record<string, unknown>).code;

      if (
        code === "INVALID_REQUEST" ||
        code === "INVALID_CREDENTIALS" ||
        code === "AUTHENTICATION_RATE_LIMITED" ||
        code === "UNAUTHORIZED"
      ) {
        return code;
      }
    }
  } catch {
    return null;
  }

  return null;
}

function createLoginError(
  status: number,
  code: AuthenticationErrorCode | null,
): AuthenticationError {
  if (status === 400 && code === "INVALID_REQUEST") {
    return new AuthenticationError("INVALID_REQUEST", status);
  }

  if (status === 401 && code === "INVALID_CREDENTIALS") {
    return new AuthenticationError("INVALID_CREDENTIALS", status);
  }

  if (status === 429 && code === "AUTHENTICATION_RATE_LIMITED") {
    return new AuthenticationError("AUTHENTICATION_RATE_LIMITED", status);
  }

  return new AuthenticationError("UNKNOWN", status);
}

function buildAuthenticatedRequestInit(
  init: RequestInit | undefined,
  authorization: string,
): RequestInit {
  const headers = new Headers(init?.headers);
  headers.set("Authorization", authorization);

  return {
    ...init,
    headers,
  };
}

function createAbortError(): DOMException {
  return new DOMException("The protected request was aborted.", "AbortError");
}

async function loginInternal(credentials: LoginCredentials): Promise<void> {
  const response = await fetch(buildApiUrl("/auth/login"), {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json",
    },
    body: JSON.stringify(credentials),
    cache: "no-store",
    credentials: "include",
  });

  if (!response.ok) {
    throw createLoginError(response.status, await readProblemCode(response));
  }

  const tokenResponse = parseAccessTokenResponse(await response.json());
  const issuedAt = Date.now();

  authenticationSession = {
    accessToken: tokenResponse.access_token,
    tokenType: tokenResponse.token_type,
    accessTokenExpiresAt: issuedAt + tokenResponse.expires_in * 1000,
  };

  setAuthenticationSnapshot("authenticated", "refreshIdle");
}

async function bootstrapInternal(): Promise<void> {
  setAuthenticationSnapshot("unknown", "bootstrapPending");

  try {
    const response = await fetch(buildApiUrl("/auth/refresh"), {
      method: "POST",
      credentials: "include",
      cache: "no-store",
      headers: {
        Accept: "application/json",
      },
    });

    if (response.status === 401) {
      clearAuthenticationSession();
      setAuthenticationSnapshot("unauthenticated", "bootstrapIdle");
      return;
    }

    if (!response.ok) {
      throw new AuthenticationError("UNKNOWN", response.status);
    }

    const tokenResponse = parseAccessTokenResponse(await response.json());
    const issuedAt = Date.now();

    clearAuthenticationSession();
    authenticationSession = {
      accessToken: tokenResponse.access_token,
      tokenType: tokenResponse.token_type,
      accessTokenExpiresAt: issuedAt + tokenResponse.expires_in * 1000,
    };

    setAuthenticationSnapshot("authenticated", "refreshIdle");
  } catch {
    clearAuthenticationSession();
    setAuthenticationSnapshot("authentication-error", "bootstrapIdle");
  }
}

async function performProtectedRefresh(): Promise<void> {
  setAuthenticationSnapshot(
    authenticationSession === null ? "unknown" : "authenticated",
    "refreshing",
  );

  try {
    const response = await fetch(buildApiUrl("/auth/refresh"), {
      method: "POST",
      credentials: "include",
      cache: "no-store",
      headers: {
        Accept: "application/json",
      },
    });

    if (response.status === 401) {
      clearAuthenticationSession();
      setAuthenticationSnapshot("unauthenticated", "refreshIdle");
      return;
    }

    if (!response.ok) {
      throw new AuthenticationError("UNKNOWN", response.status);
    }

    const tokenResponse = parseAccessTokenResponse(await response.json());
    const issuedAt = Date.now();

    authenticationSession = {
      accessToken: tokenResponse.access_token,
      tokenType: tokenResponse.token_type,
      accessTokenExpiresAt: issuedAt + tokenResponse.expires_in * 1000,
    };

    setAuthenticationSnapshot("authenticated", "refreshIdle");
  } catch (error) {
    clearAuthenticationSession();
    setAuthenticationSnapshot("authentication-error", "refreshIdle");

    if (error instanceof AuthenticationError) {
      throw error;
    }

    throw new AuthenticationError("UNKNOWN", 0);
  }
}

async function refreshForProtectedRequest(): Promise<void> {
  if (protectedRefreshRequest !== null) {
    return protectedRefreshRequest;
  }

  protectedRefreshRequest = performProtectedRefresh().finally(() => {
    protectedRefreshRequest = null;
  });

  return protectedRefreshRequest;
}

async function fetchWithAuthentication(
  input: RequestInfo | URL,
  init: RequestInit = {},
): Promise<Response> {
  const authorizationBeforeRequest = getAuthorizationHeader();

  if (authorizationBeforeRequest === null) {
    throw new AuthenticationError("UNAUTHORIZED", 401);
  }

  if (init.signal?.aborted) {
    throw createAbortError();
  }

  const response = await fetch(
    input,
    buildAuthenticatedRequestInit(init, authorizationBeforeRequest),
  );

  if (response.status !== 401) {
    return response;
  }

  await response.body?.cancel();

  if (init.signal?.aborted) {
    throw createAbortError();
  }

  const authorizationAfterResponse = getAuthorizationHeader();

  if (
    authorizationAfterResponse === null ||
    authorizationAfterResponse === authorizationBeforeRequest
  ) {
    await refreshForProtectedRequest();
  }

  if (init.signal?.aborted) {
    throw createAbortError();
  }

  const refreshedAuthorization = getAuthorizationHeader();

  if (refreshedAuthorization === null) {
    throw new AuthenticationError("UNAUTHORIZED", 401);
  }

  return fetch(
    input,
    buildAuthenticatedRequestInit(init, refreshedAuthorization),
  );
}

async function logoutInternal(): Promise<void> {
  const response = await fetch(buildApiUrl("/auth/logout"), {
    method: "POST",
    headers: {
      Accept: "application/json",
    },
    cache: "no-store",
    credentials: "include",
  });

  if (response.status === 204) {
    clearAuthenticationSession();
    setAuthenticationSnapshot("unauthenticated", "refreshIdle");
    return;
  }

  if (response.status === 401) {
    clearAuthenticationSession();
    setAuthenticationSnapshot("unauthenticated", "refreshIdle");
    return;
  }

  throw new AuthenticationError("UNKNOWN", response.status);
}

async function bootstrap(): Promise<void> {
  if (bootstrapStarted) {
    return bootstrapRequest ?? Promise.resolve();
  }

  bootstrapStarted = true;
  bootstrapRequest = bootstrapInternal().finally(() => {
    bootstrapRequest = null;
  });

  return bootstrapRequest;
}

async function retryBootstrap(): Promise<void> {
  if (bootstrapRequest !== null) {
    return bootstrapRequest;
  }

  bootstrapStarted = false;
  setAuthenticationSnapshot("unknown", "bootstrapRetryPending");
  return bootstrap();
}

function getAuthorizationHeader(): string | null {
  if (authenticationSession === null) {
    return null;
  }

  return `${authenticationSession.tokenType} ${authenticationSession.accessToken}`;
}

export const authService = {
  subscribe(listener: AuthenticationListener): () => void {
    listeners.add(listener);
    return () => listeners.delete(listener);
  },

  getSnapshot(): AuthenticationSnapshot {
    return authenticationSnapshot;
  },

  getAuthorizationHeader,

  fetchWithAuthentication,

  bootstrap,

  retryBootstrap,

  async login(credentials: LoginCredentials): Promise<void> {
    if (authenticationSession !== null) {
      return;
    }

    if (loginRequest !== null) {
      return loginRequest;
    }

    loginRequest = loginInternal(credentials).finally(() => {
      loginRequest = null;
    });

    return loginRequest;
  },

  async logout(): Promise<void> {
    if (logoutRequest !== null) {
      return logoutRequest;
    }

    logoutRequest = logoutInternal().finally(() => {
      logoutRequest = null;
    });

    return logoutRequest;
  },

  clearClientState(): void {
    if (authenticationSession === null) {
      return;
    }

    clearAuthenticationSession();
    setAuthenticationSnapshot("unauthenticated", "refreshIdle");
  },
};
