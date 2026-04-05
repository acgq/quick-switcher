import { useState, useEffect, useRef, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import './App.css'

// Import settings component
import Settings from './Settings'

interface WindowInfo {
  id: string  // Use string to avoid JavaScript number precision issues
  title: string
  process_name: string
}

// Detect if this is the settings window
const isSettingsWindow = window.location.search.includes('settings')

function App() {
  // Main window state (always declared, but only used when !isSettingsWindow)
  const [windows, setWindows] = useState<WindowInfo[]>([])
  const [filteredWindows, setFilteredWindows] = useState<WindowInfo[]>([])
  const [search, setSearch] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(0)
  const [loading, setLoading] = useState(false)
  const listRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  // Load windows from backend (non-blocking)
  const loadWindows = useCallback(() => {
    if (isSettingsWindow) return
    setLoading(true)
    invoke<WindowInfo[]>('get_windows')
      .then(list => {
        setWindows(list)
        setFilteredWindows(list)
      })
      .catch(e => console.error('Failed to load windows:', e))
      .finally(() => setLoading(false))
  }, [])

  // Reset state immediately
  const resetState = useCallback(() => {
    if (isSettingsWindow) return
    setSearch('')
    setSelectedIndex(0)
  }, [])

  // Search function (defined early to be used in effects)
  const searchWindows = useCallback(async (windowList: WindowInfo[], query: string) => {
    if (isSettingsWindow) return
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

  useEffect(() => {
    if (!isSettingsWindow) {
      loadWindows()
    }
  }, [loadWindows])

  // Listen for Tauri window-shown event - reset and load data
  useEffect(() => {
    if (isSettingsWindow) return
    let unlisten: (() => void) | null = null

    listen('window-shown', () => {
      resetState()
      loadWindows()
    }).then(fn => {
      unlisten = fn
    })

    return () => {
      if (unlisten) unlisten()
    }
  }, [resetState, loadWindows])

  // Listen for windows-updated event - update list only when data actually changed
  useEffect(() => {
    if (isSettingsWindow) return
    let unlisten: (() => void) | null = null

    listen<WindowInfo[]>('windows-updated', (event) => {
      const newWindows = event.payload
      setWindows(newWindows)
      // Re-apply current search filter (keep user's search state)
      if (search === '') {
        setFilteredWindows(newWindows)
      } else {
        searchWindows(newWindows, search)
      }
    }).then(fn => {
      unlisten = fn
    })

    return () => {
      if (unlisten) unlisten()
    }
  }, [search, searchWindows])

  // Adjust selectedIndex when filtered list changes (only if out of range)
  useEffect(() => {
    if (isSettingsWindow) return
    if (selectedIndex >= filteredWindows.length && filteredWindows.length > 0) {
      setSelectedIndex(filteredWindows.length - 1)
    }
  }, [filteredWindows, selectedIndex])

  // Reset selection when search query changes
  useEffect(() => {
    if (isSettingsWindow) return
    setSelectedIndex(0)
  }, [search])

  // Focus input when window gains focus
  useEffect(() => {
    if (isSettingsWindow) return
    const handleFocus = () => {
      inputRef.current?.focus()
    }
    window.addEventListener('focus', handleFocus)
    return () => window.removeEventListener('focus', handleFocus)
  }, [])

  // Search when query changes
  useEffect(() => {
    if (isSettingsWindow) return
    if (search === '') {
      setFilteredWindows(windows)
    } else {
      searchWindows(windows, search)
    }
  }, [search, windows, searchWindows])

  // Disable browser default shortcuts
  useEffect(() => {
    if (isSettingsWindow) return
    const blockBrowserShortcuts = (e: KeyboardEvent) => {
      // Block common browser shortcuts (except n and p which we use for navigation)
      if (e.ctrlKey || e.metaKey) {
        const blockedKeys = ['s', 'f', 'r', 'w', 'd', 't', 'l', 'o']
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
    if (isSettingsWindow) return
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+N: move down
      if (e.ctrlKey && e.key.toLowerCase() === 'n') {
        e.preventDefault()
        setSelectedIndex(prev =>
          prev < filteredWindows.length - 1 ? prev + 1 : prev
        )
        return
      }
      // Ctrl+P: move up
      if (e.ctrlKey && e.key.toLowerCase() === 'p') {
        e.preventDefault()
        setSelectedIndex(prev => prev > 0 ? prev - 1 : prev)
        return
      }

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
    if (isSettingsWindow) return
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

  // Render settings page if this is settings window
  if (isSettingsWindow) {
    return <Settings />
  }

  // Main window switcher
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