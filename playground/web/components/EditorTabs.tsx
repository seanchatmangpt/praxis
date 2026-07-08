'use client';

import React from 'react';
import { EditorFile } from '@/lib/types';

interface EditorTabsProps {
  files: EditorFile[];
  activeIndex: number;
  onActiveChange: (index: number) => void;
  onAddFile?: () => void;
  onRemoveFile?: (index: number) => void;
}

export default function EditorTabs({
  files,
  activeIndex,
  onActiveChange,
  onAddFile,
  onRemoveFile,
}: EditorTabsProps) {
  return (
    <div style={{ display: 'flex', borderBottom: '1px solid #ddd', backgroundColor: '#f5f5f5' }}>
      {files.map((file, idx) => (
        <div
          key={idx}
          onClick={() => onActiveChange(idx)}
          style={{
            padding: '8px 12px',
            cursor: 'pointer',
            backgroundColor: activeIndex === idx ? '#fff' : '#f5f5f5',
            borderBottom: activeIndex === idx ? '2px solid #0070f3' : 'none',
            borderRight: '1px solid #ddd',
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
          }}
        >
          <span>{file.name}</span>
          {onRemoveFile && files.length > 1 && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                onRemoveFile(idx);
              }}
              style={{
                background: 'none',
                border: 'none',
                cursor: 'pointer',
                fontSize: '12px',
                color: '#999',
              }}
            >
              ✕
            </button>
          )}
        </div>
      ))}
      {onAddFile && (
        <button
          onClick={onAddFile}
          style={{
            padding: '8px 12px',
            background: 'none',
            border: 'none',
            cursor: 'pointer',
            color: '#0070f3',
            fontSize: '14px',
          }}
        >
          +
        </button>
      )}
    </div>
  );
}
