import { useEffect, useRef, useState, useCallback } from 'react'
import type { OrderBook } from './types'

export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error'

export function useOrderBookSocket(url: string | null) {
  const [orderBook, setOrderBook] = useState<OrderBook | null>(null)
  const [status, setStatus] = useState<ConnectionStatus>('disconnected')
  const [error, setError] = useState<string | null>(null)
  const [updateCount, setUpdateCount] = useState(0)
  const wsRef = useRef<WebSocket | null>(null)
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null)

  const connect = useCallback(() => {
    if (!url) return

    setStatus('connecting')
    setError(null)

    const ws = new WebSocket(url)
    wsRef.current = ws

    ws.onopen = () => {
      setStatus('connected')
      setError(null)
    }

    ws.onmessage = (event) => {
      try {
        const data: OrderBook = JSON.parse(event.data)
        setOrderBook(data)
        setUpdateCount((c) => c + 1)
      } catch (e) {
        console.error('Failed to parse order book:', e)
      }
    }

    ws.onerror = () => {
      setStatus('error')
      setError('WebSocket connection error')
    }

    ws.onclose = (event) => {
      setStatus('disconnected')
      wsRef.current = null

      // Auto-reconnect after 3 seconds unless it was a clean close
      if (!event.wasClean) {
        reconnectTimer.current = setTimeout(() => {
          connect()
        }, 3000)
      }
    }
  }, [url])

  const disconnect = useCallback(() => {
    if (reconnectTimer.current) {
      clearTimeout(reconnectTimer.current)
      reconnectTimer.current = null
    }
    if (wsRef.current) {
      wsRef.current.close(1000)
      wsRef.current = null
    }
    setStatus('disconnected')
  }, [])

  useEffect(() => {
    if (url) {
      connect()
    }
    return () => {
      disconnect()
    }
  }, [url, connect, disconnect])

  return { orderBook, status, error, updateCount, disconnect }
}
