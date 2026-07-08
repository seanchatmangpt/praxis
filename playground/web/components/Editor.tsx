'use client';

import React from 'react';

interface EditorProps {
  value: string;
  onChange: (value: string) => void;
  language?: string;
  readOnly?: boolean;
  theme?: 'light' | 'dark';
}

export function Editor({
  value,
  onChange,
  language = 'javascript',
  readOnly = false,
  theme = 'light',
}: EditorProps): React.ReactElement {
  return (
    <div
      style={{
        border: '1px solid #ccc',
        borderRadius: '4px',
        overflow: 'hidden',
        height: '100%',
        backgroundColor: theme === 'dark' ? '#1e1e1e' : '#fff',
      }}
    >
      <textarea
        value={value}
        onChange={(e) => onChange(e.target.value)}
        readOnly={readOnly}
        style={{
          width: '100%',
          height: '100%',
          padding: '1rem',
          fontFamily: 'monospace',
          fontSize: '14px',
          border: 'none',
          resize: 'none',
          backgroundColor: theme === 'dark' ? '#1e1e1e' : '#fff',
          color: theme === 'dark' ? '#d4d4d4' : '#000',
        }}
        placeholder={`Enter ${language} code here...`}
      />
    </div>
  );
}
