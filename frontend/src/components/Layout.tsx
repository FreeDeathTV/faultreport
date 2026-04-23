import { ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const navigate = useNavigate();

  const handleHome = () => {
    navigate('/dashboard');
  };

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white shadow">
        <div className="max-w-7xl mx-auto py-6 px-4 flex justify-between items-center">
          <h1 className="text-3xl font-bold text-gray-900">FaultReport</h1>
          <button
            onClick={handleHome}
            className="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600"
          >
            Dashboard
          </button>
        </div>
      </header>
      <main>{children}</main>
    </div>
  );
}

