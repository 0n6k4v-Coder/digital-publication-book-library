export interface LoginCredentials {
  email: string;
  password: string;
}

export interface LoginTokenResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
  refresh_token: string;
  refresh_expires_in: number;
}

export type AuthenticationErrorCode =
  | "INVALID_REQUEST"
  | "INVALID_CREDENTIALS"
  | "AUTHENTICATION_RATE_LIMITED"
  | "UNAUTHORIZED"
  | "INVALID_RESPONSE"
  | "UNKNOWN";
