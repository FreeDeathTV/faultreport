import { render, screen } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import App from './App';

describe('App', () => {
  it('renders login page by default', () => {
    render(
      <BrowserRouter>
        <App />
      </BrowserRouter>
    );
    const linkElement = screen.getByText(/Login to FaultReport/i);
    expect(linkElement).toBeInTheDocument();
  });

  it('renders dashboard page', () => {
    render(
      <BrowserRouter>
        <App />
      </BrowserRouter>
    );
    const linkElement = screen.getByText(/Sign in with Firebase/i);
    linkElement.click();
    expect(screen.getByText(/Recent Errors/i)).toBeInTheDocument();
  });

  it('renders error detail page', () => {
    render(
      <BrowserRouter>
        <App />
      </BrowserRouter>
    );
    const linkElement = screen.getByText(/Sign in with Firebase/i);
    linkElement.click();
    const errorRow = screen.getByText(/Backend unreachable — showing mock data/i);
    errorRow.click();
    expect(screen.getByText(/Error Details/i)).toBeInTheDocument();
  });
});
