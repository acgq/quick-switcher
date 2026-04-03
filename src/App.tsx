import { useState, useEffect, useRef, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'

// Import settings component conditionally based on window label
import Settings from './Settings'

interface WindowInfo {
  id: number
  title: string
  process_name: string
}

// Detect if this is the settings window
const isSettingsWindow = window.location.search.includes('settings')

function App() {
  // Render settings page if this is settings window
  if (isSettingsWindow) {
    return <Settings />
  }

  // Main window switcher
  const [windows, setWindows] = useState<WindowInfo[]>([])
  const [filteredWindows, setFilteredWindows] = useState<WindowInfo[]>([])
  const [search, setSearch] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(0)
  const [loading, setLoading] = useState(false)
  const listRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  // Load windows from backend
  const loadWindows = useCallback(() => {
    setLoading(true)
    invoke<WindowInfo[]>('get_windows')
      .then(list => {
        setWindows(list)
        setFilteredWindows(list)
      })
      .catch(e => console.error('Failed to load windows:', e))
      .finally(() => setLoading(false))
  }, [])

  // Reset state and reload
  const resetAndReload = useCallback(() => {
    setSearch('')
    setSelectedIndex(0)
    loadWindows()
    // Focus input when window is shown
    inputRef.current?.focus()
  }, [loadWindows])

  useEffect(() => {
    loadWindows()
  }, [loadWindows])

  // Listen for window focus event - this fires when window is shown
  useEffect(() => {
    const handleFocus = () => {
      resetAndReload()
    }
    window.addEventListener('focus', handleFocus)
    return () => window.removeEventListener('focus', handleFocus)
  }, [resetAndReload])

  // Search when query changes
  useEffect(() => {
    if (search === '') {
      setFilteredWindows(windows)
    } else {
      searchWindows(windows, search)
    }
    setSelectedIndex(0)
  }, [search, windows])

  const searchWindows = useCallback(async (windowList: WindowInfo[], query: string) => {
    try {
      const filtered = await invoke<WindowInfo[]>('search_windows', {
        windows: windowList,
        query: query
      })
      setFilteredWindows(filtered)
    } catch (e) {
      console.error('Search failed:', e)
      // Fallback to simple filter
      const filtered = windowList.filter(w =>
        w.title.toLowerCase().includes(query.toLowerCase()) ||
        w.process_name.toLowerCase().includes(query.toLowerCase())
      )
      setFilteredWindows(filtered)
    }
  }, [])

  // Disable browser default shortcuts
  useEffect(() => {
    const blockBrowserShortcuts = (e: KeyboardEvent) => {
      // Block common browser shortcuts
      if (e.ctrlKey || e.metaKey) {
        const blockedKeys = ['p', 's', 'f', 'r', 'w', 'd', 't', 'n', 'l', 'o']
        if (blockedKeys.includes(e.key.toLowerCase())) {
          e.preventDefault()
          e.stopPropagation()
        }
        // Block Ctrl+Tab and Ctrl+Shift+Tab
        if (e.key === 'Tab') {
          e.preventDefault()
          e.stopPropagation()
        }
      }
      // Block F5 (refresh) and F12 (devtools - though Tauri handles this)
      if (e.key === 'F5' || e.key === 'F12') {
        e.preventDefault()
        e.stopPropagation()
      }
    }

    window.addEventListener('keydown', blockBrowserShortcuts, true) // use capture phase
    return () => window.removeEventListener('keydown', blockBrowserShortcuts, true)
  }, [])

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        setSelectedIndex(prev =>
          prev < filteredWindows.length - 1 ? prev + 1 : prev
        )
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        setSelectedIndex(prev => prev > 0 ? prev - 1 : prev)
      } else if (e.key === 'Enter') {
        e.preventDefault()
        if (filteredWindows[selectedIndex]) {
          switchToWindow(filteredWindows[selectedIndex].id)
        }
      } else if (e.key === 'Escape') {
        e.preventDefault()
        hideWindow()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [selectedIndex, filteredWindows])

  // Scroll selected item into view
  useEffect(() => {
    if (listRef.current) {
      const selectedElement = listRef.current.querySelector('.selected')
      if (selectedElement) {
        selectedElement.scrollIntoView({ block: 'nearest' })
      }
    }
  }, [selectedIndex])

  async function switchToWindow(id: number) {
    await invoke('switch_window', { windowId: id })
    await invoke('hide_window')
  }

  async function hideWindow() {
    await invoke('hide_window')
  }

  return (
    <div className="container">
      <input
        ref={inputRef}
        type="text"
        placeholder="Search windows... (supports pinyin)"
        value={search}
        onChange={e => setSearch(e.target.value)}
        autoFocus
      />
      <div className="window-list" ref={listRef}>
        {loading && <div className="loading">Loading...</div>}
        {!loading && filteredWindows.map((w, index) => (
          <div
            key={w.id}
            className={`window-item ${index === selectedIndex ? 'selected' : ''}`}
            onClick={() => switchToWindow(w.id)}
            onMouseEnter={() => setSelectedIndex(index)}
          >
            <span className="process">{w.process_name}</span>
            <span className="title">{w.title}</span>
          </div>
        ))}
        {!loading && filteredWindows.length === 0 && (
          <div className="no-results">No windows found</div>
        )}
      </div>
    </div>
  )
}

export default App