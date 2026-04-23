import { useEffect, useState } from 'react';
import type { ErrorDetail } from '../types';
import { useNavigate, useParams } from 'react-router-dom';
import { getErrorDetails } from '../api/client';
import type { ErrorRecord } from '../types';

interface ErrorDetailProps {
  error?: ErrorDetail;
}

export function ErrorDetail({ error }: ErrorDetailProps) {
  const navigate = useNavigate();
  const { errorId } = useParams<{ errorId: string }>();
const [errorDetails, setErrorDetails] = useState<ErrorDetail | null>(error || null);
const setNewErrorDetails = (value: ErrorDetail | null) => setErrorDetails(value);
  const [loading, setLoading] = useState(!error);
  const [errorState, setErrorState] = useState<string | null>(null);

  useEffect(() => {
    if (!errorId || error) return;

    setLoading(true);
    setErrorState(null);
getErrorDetails('dummy-project', errorId)
      .then((data) => {
        const errorRecord: ErrorRecord = {
          id: data.id,
          hash: data.hash,
          message: data.message,
          count: data.count,
          first_seen_at: data.first_seen_at,
          last_seen_at: data.last_seen_at,
          status: 'active'
        };
        setNewErrorDetails({ ...data, ...errorRecord });
      })
      .catch((err) => setErrorState(err.message ?? 'Failed to load error details'))
      .finally(() => setLoading(false));
  }, [errorId, error]);

  const handleBack = () => {
    navigate('/dashboard');
  };

return (
  <div className="p-8">
    {loading ? (
      <div className="p-8">
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-2xl font-bold">Error Details</h2>
          <button
            onClick={handleBack}
            className="bg-gray-500 text-white px-4 py-2 rounded hover:bg-gray-600"
          >
            Back to List
          </button>
        </div>
        <div className="bg-white rounded-lg shadow p-6">
          <p className="text-gray-600">Loading error details...</p>
        </div>
      </div>
    ) : errorDetails ? (
      <>
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-2xl font-bold">Error Details</h2>
          <button
            onClick={handleBack}
            className="bg-gray-500 text-white px-4 py-2 rounded hover:bg-gray-600"
          >
            Back to List
          </button>
        </div>

        <div className="bg-white rounded-lg shadow p-6 mb-6">
          <div className="flex justify-between items-start mb-4">
            <div>
              <h3 className="text-lg font-semibold mb-2">Error Message</h3>
              <p className="text-gray-600">{errorDetails.message}</p>
            </div>
            <div className="text-right">
              <p className="text-sm text-gray-500">ID: {errorDetails.id}</p>
              <p className="text-sm text-gray-500">Hash: {errorDetails.hash}</p>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4 mb-4">
            <div>
              <h4 className="font-medium text-gray-600 mb-1">Count</h4>
              <p className="text-lg font-semibold text-blue-600">{errorDetails.count}</p>
            </div>
            <div>
              <h4 className="font-medium text-gray-600 mb-1">Status</h4>
              <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                errorDetails.status === 'active' ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
              }`}>
                {errorDetails.status}
              </span>
            </div>
            <div>
              <h4 className="font-medium text-gray-600 mb-1">First Seen</h4>
              <p>{new Date(errorDetails.first_seen_at).toLocaleString()}</p>
            </div>
            <div>
              <h4 className="font-medium text-gray-600 mb-1">Last Seen</h4>
              <p>{new Date(errorDetails.last_seen_at).toLocaleString()}</p>
            </div>
          </div>

          {errorDetails.stack && (
            <div className="mb-4">
              <h4 className="font-medium text-gray-600 mb-2">Stack Trace</h4>
              <pre className="bg-gray-100 p-3 rounded text-sm font-mono text-gray-800 overflow-x-auto">
                {errorDetails.stack}
              </pre>
            </div>
          )}

          {Object.keys(errorDetails.context).length > 0 && (
            <div>
              <h4 className="font-medium text-gray-600 mb-2">Context</h4>
              <pre className="bg-gray-100 p-3 rounded text-sm font-mono text-gray-800 overflow-x-auto">
                {JSON.stringify(errorDetails.context, null, 2)}
              </pre>
            </div>
          )}
        </div>
      </>
    ) : (
      <div className="p-8">
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-2xl font-bold">Error Details</h2>
          <button
            onClick={handleBack}
            className="bg-gray-500 text-white px-4 py-2 rounded hover:bg-gray-600"
          >
            Back to List
          </button>
        </div>
        <div className="bg-white rounded-lg shadow p-6">
          <p className="text-red-800">{errorState || 'Failed to load error details'}</p>
        </div>
      </div>
    )}
  </div>
);
}