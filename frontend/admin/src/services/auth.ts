import type {
  AuthenticationErrorCode,
  LoginCredentials,
  LoginTokenResponse,
} from "../types/auth";

interface AuthenticationSession {
  accessToken: string;
  tokenType: string;
  accessTokenExpiresAt: number;
  refreshToken: string;
  refreshTokenExpiresAt: number;
}

type AuthenticationListener = () => void;

const listeners = new Set<AuthenticationListener>();
const apiOrigin = (import.meta.env.VITE_API_ORIGIN ?? "")
  .trim()
  .replace(/\/$/, "");

let authenticationSession: AuthenticationSession | null = null;
let loginRequest: Promise<void> | null = null;
let logoutRequest: Promise<void> | null = null;

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

function buildApiUrl(pathname: string): string {
  if (import.meta.env.PROD && window.location.protocol !== "https:") {
    throw new AuthenticationError("UNKNOWN", 0);
  }

  if (apiOrigin.length === 0) {
    return pathname;
  }

  const origin = new URL(apiOrigin);

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

function parseLoginTokenResponse(value: unknown): LoginTokenResponse {
  if (
    typeof value !== "object" ||
    value === null ||
    !isNonEmptyString((value as Record<string, unknown>).access_token) ||
    !isNonEmptyString((value as Record<string, unknown>).token_type) ||
    !isFiniteNumber((value as Record<string, unknown>).expires_in) ||
    !isNonEmptyString((value as Record<string, unknown>).refresh_token) ||
    !isFiniteNumber((value as Record<string, unknown>).refresh_expires_in)
  ) {
    throw new AuthenticationError("INVALID_RESPONSE", 200);
  }

  return value as LoginTokenResponse;
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

async function loginInternal(credentials: LoginCredentials): Promise<void> {
  const response = await fetch(buildApiUrl("/auth/login"), {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json",
    },
    body: JSON.stringify(credentials),
    cache: "no-store",
  });

  if (!response.ok) {
    throw createLoginError(response.status, await readProblemCode(response));
  }

  const tokenResponse = parseLoginTokenResponse(await response.json());
  const issuedAt = Date.now();

  authenticationSession = {
    accessToken: tokenResponse.access_token,
    tokenType: tokenResponse.token_type,
    accessTokenExpiresAt: issuedAt + tokenResponse.expires_in * 1000,
    refreshToken: tokenResponse.refresh_token,
    refreshTokenExpiresAt: issuedAt + tokenResponse.refresh_expires_in * 1000,
  };

  notifyListeners();
}

async function logoutInternal(): Promise<void> {
  if (authenticationSession === null) {
    return;
  }

  const sessionAtRequestStart = authenticationSession;

  const response = await fetch(buildApiUrl("/auth/logout"), {
    method: "POST",
    headers: {
      Accept: "application/json",
      Authorization: `${sessionAtRequestStart.tokenType} ${sessionAtRequestStart.accessToken}`,
    },
    cache: "no-store",
  });

  if (response.status === 204) {
    authenticationSession = null;
    notifyListeners();
    return;
  }

  if (response.status === 401) {
    authenticationSession = null;
    notifyListeners();
    return;
  }

  throw new AuthenticationError("UNKNOWN", response.status);
}

export const authService = {
  subscribe(listener: AuthenticationListener): () => void {
    listeners.add(listener);
    return () => listeners.delete(listener);
  },

  getSnapshot(): boolean {
    return authenticationSession !== null;
  },

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
    if (authenticationSession === null) {
      return;
    }

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

    authenticationSession = null;
    notifyListeners();
  },
};
