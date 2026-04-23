export interface ErrorRecord {
  id: string;
  hash: string;
  message: string;
  count: number;
  first_seen_at: string;
  last_seen_at: string;
  status: 'active' | 'archived';
}

export interface ApiResponse {
  errors: ErrorRecord[];
  total: number;
  page: number;
  per_page: number;
}

export interface ErrorDetail extends ErrorRecord {
  stack: string;
  context: Record<string, any>;
}

