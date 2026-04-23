import { Layout } from '../components/Layout';
import { useNavigate } from 'react-router-dom';

export function Login() {
  const navigate = useNavigate();

  return (
    <Layout>
      <div className="max-w-md mx-auto mt-20 p-8 bg-white shadow-lg rounded-lg">
        <h1 className="text-2xl font-bold mb-6 text-center">Login to FaultReport</h1>
        <button
          className="w-full bg-blue-500 text-white py-2 px-4 rounded hover:bg-blue-600"
          onClick={() => navigate('/dashboard')}
        >
          Sign in with Firebase
        </button>
        <p className="mt-4 text-sm text-gray-600 text-center">
          MVP mock - click to dashboard
        </p>
      </div>
    </Layout>
  );
}

