/**
 * Unit tests for Toolbar component.
 * Verifies button visibility, callback firing, and disabled state.
 */

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import Toolbar from '../Toolbar';

describe('Toolbar', () => {
  describe('test_all_buttons_present', () => {
    it('renders all buttons with correct labels', () => {
      const mockCallbacks = {
        onLoadCase: vi.fn(),
        onRunAllDialects: vi.fn(),
        onRunHooks: vi.fn(),
        onReplay: vi.fn(),
        onGenerateReport: vi.fn(),
      };

      render(
        <Toolbar
          onLoadCase={mockCallbacks.onLoadCase}
          onRunAllDialects={mockCallbacks.onRunAllDialects}
          onRunHooks={mockCallbacks.onRunHooks}
          onReplay={mockCallbacks.onReplay}
          onGenerateReport={mockCallbacks.onGenerateReport}
        />
      );

      // Verify all buttons are present
      expect(screen.getByText(/Load Case/i)).toBeTruthy();
      expect(screen.getByText(/Run All Dialects/i)).toBeTruthy();
      expect(screen.getByText(/Run Hooks/i)).toBeTruthy();
      expect(screen.getByText(/Replay/i)).toBeTruthy();
      expect(screen.getByText(/Report/i)).toBeTruthy();
    });

    it('renders buttons even without callbacks', () => {
      render(<Toolbar />);

      // Verify buttons render when no callbacks provided
      expect(screen.getByText(/Load Case/i)).toBeTruthy();
      expect(screen.getByText(/Run All Dialects/i)).toBeTruthy();
      expect(screen.getByText(/Run Hooks/i)).toBeTruthy();
      expect(screen.getByText(/Replay/i)).toBeTruthy();
      expect(screen.getByText(/Report/i)).toBeTruthy();
    });
  });

  describe('test_button_click_fires_callback', () => {
    it('fires onLoadCase when Load Case button clicked', () => {
      const onLoadCase = vi.fn();

      render(<Toolbar onLoadCase={onLoadCase} />);

      const button = screen.getByText(/Load Case/i);
      fireEvent.click(button);

      expect(onLoadCase).toHaveBeenCalledTimes(1);
    });

    it('fires onRunAllDialects when Run All Dialects button clicked', () => {
      const onRunAllDialects = vi.fn();

      render(<Toolbar onRunAllDialects={onRunAllDialects} />);

      const button = screen.getByText(/Run All Dialects/i);
      fireEvent.click(button);

      expect(onRunAllDialects).toHaveBeenCalledTimes(1);
    });

    it('fires onRunHooks when Run Hooks button clicked', () => {
      const onRunHooks = vi.fn();

      render(<Toolbar onRunHooks={onRunHooks} />);

      const button = screen.getByText(/Run Hooks/i);
      fireEvent.click(button);

      expect(onRunHooks).toHaveBeenCalledTimes(1);
    });

    it('fires onReplay when Replay button clicked', () => {
      const onReplay = vi.fn();

      render(<Toolbar onReplay={onReplay} />);

      const button = screen.getByText(/Replay/i);
      fireEvent.click(button);

      expect(onReplay).toHaveBeenCalledTimes(1);
    });

    it('fires onGenerateReport when Report button clicked', () => {
      const onGenerateReport = vi.fn();

      render(<Toolbar onGenerateReport={onGenerateReport} />);

      const button = screen.getByText(/Report/i);
      fireEvent.click(button);

      expect(onGenerateReport).toHaveBeenCalledTimes(1);
    });

    it('does not fire other callbacks when button clicked', () => {
      const mockCallbacks = {
        onLoadCase: vi.fn(),
        onRunAllDialects: vi.fn(),
        onRunHooks: vi.fn(),
        onReplay: vi.fn(),
        onGenerateReport: vi.fn(),
      };

      render(
        <Toolbar
          onLoadCase={mockCallbacks.onLoadCase}
          onRunAllDialects={mockCallbacks.onRunAllDialects}
          onRunHooks={mockCallbacks.onRunHooks}
          onReplay={mockCallbacks.onReplay}
          onGenerateReport={mockCallbacks.onGenerateReport}
        />
      );

      const button = screen.getByText(/Load Case/i);
      fireEvent.click(button);

      expect(mockCallbacks.onLoadCase).toHaveBeenCalledTimes(1);
      expect(mockCallbacks.onRunAllDialects).not.toHaveBeenCalled();
      expect(mockCallbacks.onRunHooks).not.toHaveBeenCalled();
      expect(mockCallbacks.onReplay).not.toHaveBeenCalled();
      expect(mockCallbacks.onGenerateReport).not.toHaveBeenCalled();
    });
  });

  describe('test_buttons_disabled_when_loading', () => {
    it('disables all buttons when isLoading is true', () => {
      render(<Toolbar isLoading={true} />);

      const buttons = screen.getAllByRole('button');
      buttons.forEach((button) => {
        expect((button as HTMLButtonElement).disabled).toBe(true);
      });
    });

    it('enables all buttons when isLoading is false', () => {
      render(<Toolbar isLoading={false} />);

      const buttons = screen.getAllByRole('button');
      buttons.forEach((button) => {
        expect((button as HTMLButtonElement).disabled).toBe(false);
      });
    });

    it('enables all buttons by default (isLoading not specified)', () => {
      render(<Toolbar />);

      const buttons = screen.getAllByRole('button');
      buttons.forEach((button) => {
        expect((button as HTMLButtonElement).disabled).toBe(false);
      });
    });

    it('shows loading indicator when isLoading is true', () => {
      render(<Toolbar isLoading={true} />);

      expect(screen.getByText(/Processing/i)).toBeTruthy();
    });

    it('does not show loading indicator when isLoading is false', () => {
      render(<Toolbar isLoading={false} />);

      expect(screen.queryByText(/Processing/i)).toBeFalsy();
    });

    it('clicking button when disabled does not fire callback', () => {
      const onLoadCase = vi.fn();

      render(<Toolbar onLoadCase={onLoadCase} isLoading={true} />);

      const button = screen.getByText(/Load Case/i);
      fireEvent.click(button);

      // Button is disabled, so callback should not be called
      expect(onLoadCase).not.toHaveBeenCalled();
    });
  });
});
