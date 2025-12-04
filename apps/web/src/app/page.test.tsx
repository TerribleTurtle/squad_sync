import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import Home from './page';

describe('Home Page', () => {
  it('renders the main heading', () => {
    render(<Home />);
    expect(screen.getByText('FluxReplay')).toBeInTheDocument();
    expect(screen.getByText(/Sync Your Squad/i)).toBeInTheDocument();
  });
});
