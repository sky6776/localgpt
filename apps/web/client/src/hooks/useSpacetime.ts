import { useEffect, useState, useCallback } from 'react'
import { SpacetimeDBClient } from '@spacetimedb/sdk'

// Types (will be replaced by generated bindings)
interface Player {
  identity: string
  name: string
  x: number
  y: number
  z: number
  rotationY: number
  online: boolean
}

interface WorldEntity {
  id: number
  entityType: string
  x: number
  y: number
  z: number
  rotationY: number
  scale: number
  metadata: string
}

interface ChatMessage {
  id: number
  senderIdentity: string
  senderName: string
  message: string
  timestamp: bigint
}

interface SpacetimeState {
  connected: boolean
  identity: string | null
  players: Player[]
  entities: WorldEntity[]
  messages: ChatMessage[]
  sendMessage: (message: string) => void
  movePlayer: (x: number, y: number, z: number, rotationY: number) => void
  setName: (name: string) => void
}

const SPACETIMEDB_URL = import.meta.env.VITE_SPACETIMEDB_URL || 'ws://localhost:3000'
const MODULE_NAME = import.meta.env.VITE_MODULE_NAME || 'localgpt-world'

export function useSpacetime(): SpacetimeState {
  const [connected, setConnected] = useState(false)
  const [identity, setIdentity] = useState<string | null>(null)
  const [players, setPlayers] = useState<Player[]>([])
  const [entities, setEntities] = useState<WorldEntity[]>([])
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [client, setClient] = useState<SpacetimeDBClient | null>(null)

  useEffect(() => {
    const client = new SpacetimeDBClient(SPACETIMEDB_URL, MODULE_NAME)

    client.on('connected', (token: string) => {
      setConnected(true)
      setIdentity(token)
    })

    client.on('disconnected', () => {
      setConnected(false)
    })

    // Subscribe to tables
    client.subscribe(['SELECT * FROM player', 'SELECT * FROM world_entity', 'SELECT * FROM chat_message'])

    // Handle table updates
    client.on('rowUpdate', (table: string, row: Record<string, unknown>) => {
      if (table === 'player') {
        setPlayers(prev => {
          const existing = prev.findIndex(p => p.identity === row.identity)
          const player = row as Player
          if (existing >= 0) {
            const updated = [...prev]
            updated[existing] = player
            return updated
          }
          return [...prev, player]
        })
      } else if (table === 'world_entity') {
        setEntities(prev => {
          const existing = prev.findIndex(e => e.id === row.id)
          const entity = row as WorldEntity
          if (existing >= 0) {
            const updated = [...prev]
            updated[existing] = entity
            return updated
          }
          return [...prev, entity]
        })
      } else if (table === 'chat_message') {
        setMessages(prev => {
          const msg = row as ChatMessage
          if (prev.find(m => m.id === msg.id)) return prev
          return [...prev, msg].sort((a, b) => Number(a.id) - Number(b.id))
        })
      }
    })

    client.on('rowDelete', (table: string, row: Record<string, unknown>) => {
      if (table === 'player') {
        setPlayers(prev => prev.filter(p => p.identity !== row.identity))
      } else if (table === 'world_entity') {
        setEntities(prev => prev.filter(e => e.id !== row.id))
      }
    })

    client.connect()
    setClient(client)

    return () => {
      client.disconnect()
    }
  }, [])

  const sendMessage = useCallback((message: string) => {
    if (client) {
      client.callReducer('send_chat', [message])
    }
  }, [client])

  const movePlayer = useCallback((x: number, y: number, z: number, rotationY: number) => {
    if (client) {
      client.callReducer('move_player', [x, y, z, rotationY])
    }
  }, [client])

  const setName = useCallback((name: string) => {
    if (client) {
      client.callReducer('set_player_name', [name])
    }
  }, [client])

  return {
    connected,
    identity,
    players,
    entities,
    messages,
    sendMessage,
    movePlayer,
    setName,
  }
}
