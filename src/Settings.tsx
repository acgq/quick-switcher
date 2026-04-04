import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'
import './Settings.css'

interface ShortcutConfig {
  modifiers: string[]
  key: string
}

function Settings() {
  const [shortcut, setShortcut] = useState<ShortcutConfig>({ modifiers: [], key: '' })
  const [isRecording, setIsRecording] = useState(false)
  const [tempShortcut, setTempShortcut] = useState<string>('')
  const [saved, setSaved] = useState(false)
  const [error, setError] = useState('')
  const [autostart, setAutostart] = useState(false)
  const [autostartError, setAutostartError] = useState('')
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    loadShortcut()
    loadAutostart()
  }, [])

  // Keyboard capture when recording
  useEffect(() => {
    if (!isRecording) return

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault()
      e.stopPropagation()

      const modifiers: string[] = []
      if (e.altKey) modifiers.push('Alt')
      if (e.ctrlKey) modifiers.push('Ctrl')
      if (e.shiftKey) modifiers.push('Shift')
      if (e.metaKey) modifiers.push('Win')

      // Get the main key
      let key = ''
      if (e.code.startsWith('Key')) {
        key = e.code.replace('Key', '')
      } else if (e.code.startsWith('Digit')) {
        key = e.code.replace('Digit', '')
      } else if (e.code === 'Space') {
        key = 'Space'
      } else if (e.code.startsWith('F') && e.code.length <= 3) {
        key = e.code
      } else if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(e.code)) {
        key = e.code.replace('Arrow', '')
      } else if (['Escape', 'Enter', 'Tab', 'Backspace', 'Delete', 'Insert', 'Home', 'End', 'PageUp', 'PageDown'].includes(e.code)) {
        key = e.code
      }

      if (key && modifiers.length > 0) {
        setTempShortcut(formatShortcut({ modifiers, key }))
        setShortcut({ modifiers, key })
      }
    }

    window.addEventListener('keydown', handleKeyDown, true)
    return () => window.removeEventListener('keydown', handleKeyDown, true)
  }, [isRecording])

  async function loadShortcut() {
    try {
      const config = await invoke<ShortcutConfig>('get_shortcut')
      setShortcut(config)
      setTempShortcut(formatShortcut(config))
    } catch (e) {
      console.error('Failed to load shortcut:', e)
    }
  }

  async function loadAutostart() {
    try {
      console.log('loading autostart status...')
      const enabled = await isEnabled()
      console.log('autostart status:', enabled)
      setAutostart(enabled)
      setAutostartError('')
    } catch (e) {
      console.error('Failed to load autostart:', e)
      setAutostartError(`Load failed: ${e}`)
    }
  }

  async function saveShortcut() {
    if (shortcut.modifiers.length === 0 || !shortcut.key) {
      setError('Please set a valid shortcut')
      return
    }

    try {
      await invoke('set_shortcut', { config: shortcut })
      setSaved(true)
      setError('')
      setTimeout(() => setSaved(false), 2000)
    } catch (e) {
      setError(`Failed to save: ${e}`)
    }
  }

  async function toggleAutostart() {
    console.log('toggleAutostart called, current state:', autostart)
    const newValue = !autostart
    console.log('trying to set autostart to:', newValue)
    try {
      if (newValue) {
        console.log('calling enable()')
        await enable()
        console.log('enable() succeeded')
      } else {
        console.log('calling disable()')
        await disable()
        console.log('disable() succeeded')
      }
      setAutostart(newValue)
      setAutostartError('')
      console.log('state updated to:', newValue)
    } catch (e) {
      console.error('Failed to set autostart:', e)
      setAutostartError(`Toggle failed: ${e}`)
    }
  }

  function formatShortcut(config: ShortcutConfig): string {
    if (!config.key && config.modifiers.length === 0) return ''
    const parts = [...config.modifiers, config.key]
    return parts.join(' + ')
  }

  function startRecording() {
    setIsRecording(true)
    setTempShortcut('')
    inputRef.current?.focus()
  }

  function stopRecording() {
    setIsRecording(false)
    if (shortcut.key) {
      setTempShortcut(formatShortcut(shortcut))
    }
  }

  return (
    <div className="settings-container">
      <h1>Settings</h1>

      <div className="setting-row">
        <div className="setting-label">
          <span className="label-text">Launch at Startup</span>
          <span className="label-hint">Automatically start when you log in</span>
        </div>
        <div className="setting-value">
          <div
            className={`toggle-switch ${autostart ? 'active' : ''}`}
            onClick={toggleAutostart}
          >
            <span className="toggle-slider"></span>
          </div>
        </div>
      </div>

      {autostartError && <div className="error-message">{autostartError}</div>}

      <div className="setting-row">
        <div className="setting-label">
          <span className="label-text">Global Shortcut</span>
          <span className="label-hint">Press the key combination to toggle the window</span>
        </div>
        <div className="setting-value">
          <input
            ref={inputRef}
            type="text"
            className={`shortcut-input ${isRecording ? 'recording' : ''}`}
            value={isRecording ? tempShortcut || 'Press keys...' : formatShortcut(shortcut)}
            readOnly
            placeholder="Click to set shortcut"
            onClick={() => {
              if (!isRecording) startRecording()
            }}
            onBlur={() => {
              if (isRecording) stopRecording()
            }}
          />
          {isRecording && (
            <button className="cancel-btn" onClick={stopRecording}>
              Cancel
            </button>
          )}
        </div>
      </div>

      {error && <div className="error-message">{error}</div>}
      {saved && <div className="success-message">Shortcut saved!</div>}

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