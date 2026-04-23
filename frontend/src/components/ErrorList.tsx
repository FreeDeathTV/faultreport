import { ErrorRecord } from '../types';
import type { ErrorDetail } from '../types';

interface ErrorListProps {
  errors: ErrorRecord[];
  onRowClick?: (errorId: string) => void;
}

export function ErrorList({ errors, onRowClick }: ErrorListProps) {
  return (
    <table className="min-w-full divide-y divide-gray-200">
      <thead className="bg-gray-50">
        <tr>
          <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Message
          </th>
          <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Count
          </th>
          <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            First Seen
          </th>
          <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Last Seen
          </th>
        </tr>
      </thead>
      <tbody className="bg-white divide-y divide-gray-200">
        {errors.map((error) => (
          <tr
            key={error.id}
            className="hover:bg-gray-50 cursor-pointer"
            onClick={() => onRowClick?.(error.id)}
          >
            <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 truncate max-w-xs">
              {error.message}
            </td>
            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {error.count}
            </td>
            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {new Date(error.first_seen_at).toLocaleString()}
            </td>
            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {new Date(error.last_seen_at).toLocaleString()}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

