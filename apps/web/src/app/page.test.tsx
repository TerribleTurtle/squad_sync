import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import Home from './page';

describe('Home Page', () => {
  it('renders the main heading', () => {
    render(<Home />);
    const elements = screen.getAllByText('FluxReplay');
    expect(elements.length).toBeGreaterThan(0);
    expect(screen.getByText(/Sync Your Squad/i)).toBeInTheDocument();
  });
});
