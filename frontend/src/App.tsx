import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { Dashboard } from './pages/Dashboard';
import { Login } from './pages/Login';
import { ErrorDetail } from './components/ErrorDetail';
// import { Layout } from './components/Layout';

function App() {
  return (
    <Router future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
      <Routes>
        <Route path="/login" element={<Login />} />
        <Route path="/dashboard" element={<Dashboard />} />
        <Route path="/dashboard/:errorId" element={<ErrorDetail />} />
        <Route path="/" element={<Login />} />
      </Routes>
    </Router>
  );
}

export default App;

