/**
 * Unit tests for EditorTabs component.
 * Verifies tab switching, tab visibility, and active state indicators.
 */

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import EditorTabs from '../EditorTabs';
import type { EditorFile } from '@/lib/types';

describe('EditorTabs', () => {
  const createMockFiles = (count: number): EditorFile[] => {
    return Array.from({ length: count }, (_, i) => ({
      name: `file${i + 1}.ttl`,
      content: `triple${i + 1}`,
      language: 'turtle' as const,
    }));
  };

  describe('test_all_tabs_visible', () => {
    it('renders all tab labels in the DOM', () => {
      const files = createMockFiles(3);

      render(
        <EditorTabs
          files={files}
          activeIndex={0}
          onActiveChange={vi.fn()}
        />
      );

      expect(screen.getByText('file1.ttl')).toBeTruthy();
      expect(screen.getByText('file2.ttl')).toBeTruthy();
      expect(screen.getByText('file3.ttl')).toBeTruthy();
    });

    it('renders exactly 4 tabs when 4 files provided', () => {
      const files = createMockFiles(4);

      render(
        <EditorTabs
          files={files}
          activeIndex={0}
          onActiveChange={vi.fn()}
        />
      );

      files.forEach((file) => {
        expect(screen.getByText(file.name)).toBeTruthy();
      });

      const tabElements = screen.getAllByText(/file\d+\.ttl/);
      expect(tabElements).toHaveLength(4);
    });

    it('renders single tab when only one file provided', () => {
      const files = createMockFiles(1);

      render(
        <EditorTabs
          files={files}
          activeIndex={0}
          onActiveChange={vi.fn()}
        />
      );

      expect(screen.getByText('file1.ttl')).toBeTruthy();
      const tabElements = screen.getAllByText(/file\d+\.ttl/);
      expect(tabElements).toHaveLength(1);
    });
  });

  describe('test_tab_switch_changes_content', () => {
    it('calls onActiveChange with correct index when tab clicked', () => {
      const files = createMockFiles(2);
      const onActiveChange = vi.fn();

      render(
        <EditorTabs
          files={files}
          activeIndex={0}
          onActiveChange={onActiveChange}
        />
      );

      const secondTab = screen.getByText('file2.ttl').closest('div');
      fireEvent.click(secondTab!);

      expect(onActiveChange).toHaveBeenCalledWith(1);
    });

    it('calling onActiveChange with first tab index', () => {
      const files = createMockFiles(3);
      const onActiveChange = vi.fn();

      render(
        <EditorTabs
          files={files}
          activeIndex={1}
          onActiveChange={onActiveChange}
        />
      );

      const firstTab = screen.getByText('file1.ttl').closest('div');
      fireEvent.click(firstTab!);

      expect(onActiveChange).toHaveBeenCalledWith(0);
    });

    it('clicking third tab calls onActiveChange with index 2', () => {
      const files = createMockFiles(4);
      const onActiveChange = vi.fn();

      render(
        <EditorTabs
          files={files}
          activeIndex={0}
          onActiveChange={onActiveChange}
        />
      );

      const thirdTab = screen.getByText('file3.ttl').closest('div');
      fireEvent.click(thirdTab!);

      expect(onActiveChange).toHaveBeenCalledWith(2);
    });

    it('clicking same tab again calls onActiveChange', () => {
      const files = createMockFiles(2);
      const onActiveChange = vi.fn();

      render(
        <EditorTabs
          files={files}
          activeIndex={0}
          onActiveChange={onActiveChange}
        />
      );

      const firstTab = screen.getByText('file1.ttl').closest('div');
      fireEvent.click(firstTab!);

      expect(onActiveChange).toHaveBeenCalledWith(0);
    });

    it('multiple tab switches call callback each time', () => {
      const files = createMockFiles(3);
      const onActiveChange = vi.fn();

      render(
        <EditorTabs
          files={files}
          activeIndex={0}
          onActiveChange={onActiveChange}
        />
      );

      fireEvent.click(screen.getByText('file2.ttl').closest('div')!);
      fireEvent.click(screen.getByText('file3.ttl').closest('div')!);
      fireEvent.click(screen.getByText('file1.ttl').closest('div')!);

      expect(onActiveChange).toHaveBeenCalledTimes(3);
      expect(onActiveChange).toHaveBeenNthCalledWith(1, 1);
      expect(onActiveChange).toHaveBeenNthCalledWith(2, 2);
      expect(onActiveChange).toHaveBeenNthCalledWith(3, 0);
    });
  });

  describe('test_active_tab_indicator', () => {
    it('active tab has blue underline indicator', () => {
      const files = createMockFiles(2);

      const { rerender } = render(
        <EditorTabs
          files={files}
          activeIndex={0}
          onActiveChange={vi.fn()}
        />
      );

      // First render: file1 is active
      let firstTabElement = screen.getByText('file1.ttl').closest('div') as HTMLElement;
      // DOM converts hex colors to RGB, so check for RGB format
      expect(firstTabElement.style.borderBottom).toContain('2px solid');
      expect(firstTabElement.style.borderBottom).toContain('rgb(0, 112, 243)');

      // Rerender with second tab active
      rerender(
        <EditorTabs
          files={files}
          activeIndex={1}
          onActiveChange={vi.fn()}
        />
      );

      // Now file2 should have the indicator
      let secondTabElement = screen.getByText('file2.ttl').closest('div') as HTMLElement;
      expect(secondTabElement.style.borderBottom).toContain('2px solid');
      expect(secondTabElement.style.borderBottom).toContain('rgb(0, 112, 243)');

      // And file1 should not have the blue underline
      firstTabElement = screen.getByText('file1.ttl').closest('div') as HTMLElement;
      // When borderBottom is 'none', it may show as empty string in style
      expect(firstTabElement.style.borderBottom).not.toContain('rgb(0, 112, 243)');
    });

    it('only one tab has active indicator at a time', () => {
      const files = createMockFiles(3);

      const { rerender } = render(
        <EditorTabs
          files={files}
          activeIndex={0}
          onActiveChange={vi.fn()}
        />
      );

      // Count tabs with active indicator (check for RGB blue color)
      let activeTabs = Array.from(document.querySelectorAll('div')).filter((el) =>
        el.style.borderBottom && el.style.borderBottom.includes('rgb(0, 112, 243)')
      );
      expect(activeTabs).toHaveLength(1);

      // Switch to second tab
      rerender(
        <EditorTabs
          files={files}
          activeIndex={1}
          onActiveChange={vi.fn()}
        />
      );

      activeTabs = Array.from(document.querySelectorAll('div')).filter((el) =>
        el.style.borderBottom && el.style.borderBottom.includes('rgb(0, 112, 243)')
      );
      expect(activeTabs).toHaveLength(1);
    });

    it('active tab has white background and inactive tabs have gray', () => {
      const files = createMockFiles(2);

      render(
        <EditorTabs
          files={files}
          activeIndex={0}
          onActiveChange={vi.fn()}
        />
      );

      const firstTab = screen.getByText('file1.ttl').closest('div') as HTMLElement;
      const secondTab = screen.getByText('file2.ttl').closest('div') as HTMLElement;

      // Active tab (file1) should have white background (DOM converts to rgb)
      expect(firstTab.style.backgroundColor).toContain('rgb(255, 255, 255)');

      // Inactive tab (file2) should have gray background (DOM converts to rgb)
      expect(secondTab.style.backgroundColor).toContain('rgb(245, 245, 245)');
    });

    it('changing activeIndex prop updates visual indicator', () => {
      const files = createMockFiles(3);

      const { rerender } = render(
        <EditorTabs
          files={files}
          activeIndex={0}
          onActiveChange={vi.fn()}
        />
      );

      expect(screen.getByText('file1.ttl').closest('div')?.style.backgroundColor).toContain('rgb(255, 255, 255)');

      rerender(
        <EditorTabs
          files={files}
          activeIndex={2}
          onActiveChange={vi.fn()}
        />
      );

      expect(screen.getByText('file3.ttl').closest('div')?.style.backgroundColor).toContain('rgb(255, 255, 255)');
      expect(screen.getByText('file1.ttl').closest('div')?.style.backgroundColor).toContain('rgb(245, 245, 245)');
    });
  });
});
