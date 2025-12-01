import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import App from './App';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('App', () => {
  it('renders the main interface', () => {
    render(<App />);
    expect(screen.getByText('SquadSync')).toBeInTheDocument();
    expect(screen.getByText('Enable Replay Buffer')).toBeInTheDocument();
  });
});
