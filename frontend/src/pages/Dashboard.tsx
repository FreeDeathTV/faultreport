import { useEffect, useState } from 'react';
import { Layout } from '../components/Layout';
import { ErrorList } from '../components/ErrorList';
import { listErrors } from '../api/client';
import { useNavigate } from 'react-router-dom';
import type { ErrorRecord } from '../types';

export function Dashboard() {
  const [errors, setErrors] = useState<ErrorRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();

  useEffect(() => {
    listErrors('dummy-project')
      .then((response) => setErrors(response.errors))
      .catch((err) => setError(err.message ?? 'Failed to load'))
      .finally(() => setLoading(false));
  }, []);

  return (
    <Layout>
      <div className="p-8">
        <h2 className="text-2xl font-bold mb-6">Recent Errors</h2>
        {loading && <p>Loading...</p>}
        {!loading && error && (
          <div className="rounded border border-red-200 bg-red-50 text-red-800 p-3 mb-4">
            Could not load errors (is the backend running at {import.meta.env.VITE_API_URL || 'http://localhost:8000'}?).
          </div>
        )}
        {!loading && !error && (
  <ErrorList
    errors={errors}
    onRowClick={(errorId: string) => navigate(`/dashboard/${errorId}`)}
  />
)}
      </div>
    </Layout>
  );
}

