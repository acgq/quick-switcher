import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './Settings.css'

interface ShortcutConfig {
  modifiers: string[]
  key: string
}

const MODIFIER_OPTIONS = ['Alt', 'Ctrl', 'Shift', 'Win']
const KEY_OPTIONS = ['Space', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'F1', 'F2', 'F3', 'F4', 'F5', 'F6', 'F7', 'F8', 'F9', 'F10', 'F11', 'F12']

function Settings() {
  const [shortcut, setShortcut] = useState<ShortcutConfig>({ modifiers: ['Alt', 'Ctrl'], key: 'Space' })
  const [selectedModifierIndex, setSelectedModifierIndex] = useState(0)
  const [selectedKeyIndex, setSelectedKeyIndex] = useState(0)
  const [editingModifiers, setEditingModifiers] = useState(false)
  const [editingKey, setEditingKey] = useState(false)
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    loadShortcut()
  }, [])

  // Keyboard navigation for modifier selection
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!editingModifiers && !editingKey) {
        if (e.key === 'ArrowUp') {
          e.preventDefault()
          if (editingModifiers) {
            setSelectedModifierIndex(prev => prev > 0 ? prev - 1 : prev)
          } else if (editingKey) {
            setSelectedKeyIndex(prev => prev > 0 ? prev - 1 : prev)
          }
        } else if (e.key === 'ArrowDown') {
          e.preventDefault()
          if (editingModifiers) {
            setSelectedModifierIndex(prev => prev < MODIFIER_OPTIONS.length - 1 ? prev + 1 : prev)
          } else if (editingKey) {
            setSelectedKeyIndex(prev => prev < KEY_OPTIONS.length - 1 ? prev + 1 : prev)
          }
        } else if (e.key === 'Enter') {
          e.preventDefault()
          if (!editingModifiers && !editingKey) {
            setEditingModifiers(true)
          } else if (editingModifiers) {
            toggleModifier(MODIFIER_OPTIONS[selectedModifierIndex])
            setEditingModifiers(false)
          } else if (editingKey) {
            setShortcut(prev => ({ ...prev, key: KEY_OPTIONS[selectedKeyIndex] }))
            setEditingKey(false)
          }
        } else if (e.key === 'Escape') {
          e.preventDefault()
          if (editingModifiers) {
            setEditingModifiers(false)
          } else if (editingKey) {
            setEditingKey(false)
          } else {
            invoke('close_settings')
          }
        } else if (e.key === 'Tab') {
          e.preventDefault()
          if (!editingModifiers && !editingKey) {
            if (e.shiftKey) {
              setEditingKey(true)
              setEditingModifiers(false)
            } else {
              setEditingModifiers(true)
              setEditingKey(false)
            }
          } else if (editingModifiers) {
            setEditingModifiers(false)
            setEditingKey(true)
            setSelectedKeyIndex(0)
          } else if (editingKey) {
            setEditingKey(false)
            setEditingModifiers(true)
            setSelectedModifierIndex(0)
          }
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [editingModifiers, editingKey, selectedModifierIndex, selectedKeyIndex])

  async function loadShortcut() {
    try {
      const config = await invoke<ShortcutConfig>('get_shortcut')
      setShortcut(config)
      setSelectedKeyIndex(KEY_OPTIONS.indexOf(config.key))
    } catch (e) {
      console.error('Failed to load shortcut:', e)
    }
  }

  async function saveShortcut() {
    try {
      await invoke('set_shortcut', { config: shortcut })
      setSaved(true)
      setTimeout(() => setSaved(false), 2000)
    } catch (e) {
      console.error('Failed to save shortcut:', e)
    }
  }

  function toggleModifier(mod: string) {
    setShortcut(prev => {
      const modifiers = prev.modifiers.includes(mod)
        ? prev.modifiers.filter(m => m !== mod)
        : [...prev.modifiers, mod]
      return { ...prev, modifiers }
    })
  }

  function formatShortcut(config: ShortcutConfig): string {
    const modStr = config.modifiers.join(' + ')
    return `${modStr} + ${config.key}`
  }

  return (
    <div className="settings-container">
      <h2>Settings</h2>

      <div className="setting-item">
        <label>Global Shortcut</label>
        <div className="shortcut-display">
          <span className="shortcut-preview">{formatShortcut(shortcut)}</span>
        </div>
      </div>

      <div className="setting-item">
        <label>Modifiers</label>
        <div className="modifier-list">
          {MODIFIER_OPTIONS.map((mod, index) => (
            <button
              key={mod}
              className={`modifier-btn ${shortcut.modifiers.includes(mod) ? 'active' : ''} ${editingModifiers && index === selectedModifierIndex ? 'selected' : ''}`}
              onClick={() => toggleModifier(mod)}
            >
              {mod}
            </button>
          ))}
        </div>
        <p className="hint">Press Enter to edit, Tab to switch between sections</p>
      </div>

      <div className="setting-item">
        <label>Key</label>
        <div className="key-list">
          {KEY_OPTIONS.slice(0, 20).map((key, index) => (
            <button
              key={key}
              className={`key-btn ${shortcut.key === key ? 'active' : ''} ${editingKey && index === selectedKeyIndex ? 'selected' : ''}`}
              onClick={() => setShortcut(prev => ({ ...prev, key }))}
            >
              {key}
            </button>
          ))}
        </div>
        <div className="key-list">
          {KEY_OPTIONS.slice(20).map((key, index) => (
            <button
              key={key}
              className={`key-btn ${shortcut.key === key ? 'active' : ''} ${editingKey && (index + 20) === selectedKeyIndex ? 'selected' : ''}`}
              onClick={() => setShortcut(prev => ({ ...prev, key }))}
            >
              {key}
            </button>
          ))}
        </div>
      </div>

      <div className="actions">
        <button className="save-btn" onClick={saveShortcut}>
          {saved ? 'Saved!' : 'Save'}
        </button>
        <button className="close-btn" onClick={() => invoke('close_settings')}>
          Close
        </button>
      </div>
    </div>
  )
}

export default Settings