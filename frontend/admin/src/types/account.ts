export type AccountStatus = "active" | "inactive";

export interface AdministratorAccount {
  id: string;
  email: string;
  displayName: string | null;
  status: AccountStatus;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
}

export interface AccountListResponse {
  items: AdministratorAccount[];
  page: number;
  pageSize: number;
  total: number;
}
