import { useState, useEffect, useRef, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'

interface WindowInfo {
  id: number
  title: string
  process_name: string
}

function App() {
  const [windows, setWindows] = useState<WindowInfo[]>([])
  const [filteredWindows, setFilteredWindows] = useState<WindowInfo[]>([])
  const [search, setSearch] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(0)
  const [loading, setLoading] = useState(false)
  const listRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    loadWindows()
  }, [])

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

  async function loadWindows() {
    setLoading(true)
    try {
      const list = await invoke<WindowInfo[]>('get_windows')
      setWindows(list)
      setFilteredWindows(list)
    } catch (e) {
      console.error('Failed to load windows:', e)
    }
    setLoading(false)
  }

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