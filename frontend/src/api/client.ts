const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:8000';
const PROJECT_ID = import.meta.env.VITE_PROJECT_ID || '00000000-0000-0000-0000-000000000000';
const API_KEY = import.meta.env.VITE_API_KEY; // Optional Bearer key for demo/backend auth
const USE_MOCK = import.meta.env.VITE_USE_MOCK === 'true';

interface ApiResponse {
  errors: any[];
  total: number;
  page: number;
  per_page: number;
}

interface ErrorDetail {
  id: string;
  hash: string;
  message: string;
  count: number;
  first_seen_at: string;
  last_seen_at: string;
  stack: string;
  context: Record<string, any>;
}

export async function listErrors(projectId: string = PROJECT_ID, page = 1): Promise<ApiResponse> {
  // Explicit mock mode
  if (USE_MOCK) {
    return mockErrors(page);
  }

  try {
    const res = await fetch(`${API_BASE}/api/projects/${projectId}/errors?page=${page}&per_page=20`, {
      headers: API_KEY ? { Authorization: `Bearer ${API_KEY}` } : undefined,
    });
    if (!res.ok) {
      throw new Error(`Failed to fetch errors (status ${res.status})`);
    }
    return res.json();
  } catch (err) {
    if (import.meta.env.DEV) {
      // Keep console noise in dev only
      console.warn('Error fetching errors, using mock data:', err);
    }
    return mockErrors(page);
  }
}

export async function getError(projectId: string = PROJECT_ID, errorId: string): Promise<ErrorDetail> {
  const res = await fetch(`${API_BASE}/api/projects/${projectId}/errors/${errorId}`, {
    headers: API_KEY ? { Authorization: `Bearer ${API_KEY}` } : undefined,
  });
  if (!res.ok) throw new Error('Failed to fetch error');
  return res.json();
}

export async function getErrorDetails(projectId: string = PROJECT_ID, errorId: string): Promise<ErrorDetail> {
  return getError(projectId, errorId);
}

function mockErrors(page: number): ApiResponse {
  const now = new Date().toISOString();
  return {
    errors: [{
      id: 'mock-error-1',
      hash: 'mockhash123',
      message: 'Backend unreachable — showing mock data',
      count: 1,
      first_seen_at: now,
      last_seen_at: now,
      stack: 'stack not available',
      context: {},
    }],
    total: 1,
    page,
    per_page: 20,
  };
}
