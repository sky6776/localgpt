import { useEffect, useState, useCallback } from 'react'
import { SpacetimeDBClient, Identity } from '@clockworklabs/spacetimedb-sdk'

// Types matching the server module (flat fields for SpacetimeDB 2.0)
export interface Player {
  identity: string
  device: string
  name: string
  x: number
  y: number
  z: number
  rotationY: number
  online: boolean
  lastSeen: bigint
}

export interface WorldEntityRow {
  id: number
  name: string
  entityType: string
  x: number
  y: number
  z: number
  rotPitch: number
  rotYaw: number
  rotRoll: number
  scale: number
  shapeJson: string
  materialJson: string | null
  lightJson: string | null
  behaviorsJson: string
  audioJson: string | null
  chunkX: number
  chunkY: number
  owner: string | null
  createdAt: bigint
}

export interface ChatMessage {
  id: number
  senderIdentity: string
  senderName: string
  message: string
  timestamp: bigint
  msgType: string
}

export interface WorldInfo {
  id: number
  name: string
  description: string
  seed: bigint
  width: number
  depth: number
  createdAt: bigint
  updatedAt: bigint
}

interface SpacetimeState {
  connected: boolean
  identity: string | null
  players: Player[]
  entities: WorldEntityRow[]
  messages: ChatMessage[]
  worldInfo: WorldInfo | null
  sendMessage: (message: string) => void
  movePlayer: (x: number, y: number, z: number, rotationY: number) => void
  setName: (name: string) => void
  setDevice: (device: string) => void
  spawnEntity: (entity: Partial<WorldEntityRow>) => void
  removeEntity: (id: number) => void
  subscribeChunk: (x: number, y: number) => void
}

const SPACETIMEDB_URL = import.meta.env.VITE_SPACETIMEDB_URL || 'ws://localhost:3000'
const MODULE_NAME = import.meta.env.VITE_MODULE_NAME || 'localgpt-world'

export function useSpacetime(): SpacetimeState {
  const [connected, setConnected] = useState(false)
  const [identity, setIdentity] = useState<string | null>(null)
  const [players, setPlayers] = useState<Player[]>([])
  const [entities, setEntities] = useState<WorldEntityRow[]>([])
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [worldInfo, setWorldInfo] = useState<WorldInfo | null>(null)
  const [client, setClient] = useState<SpacetimeDBClient | null>(null)

  useEffect(() => {
    const client = new SpacetimeDBClient(SPACETIMEDB_URL, MODULE_NAME)

    client.on('connected', (identity: Identity) => {
      setConnected(true)
      setIdentity(identity.toHexString())
    })

    client.on('disconnected', () => {
      setConnected(false)
    })

    client.on('error', (error: Error) => {
      console.error('SpacetimeDB error:', error)
    })

    // Subscribe to tables
    client.subscribe([
      'SELECT * FROM player',
      'SELECT * FROM world_entity',
      'SELECT * FROM chat_message',
      'SELECT * FROM world_info',
    ])

    // Handle initial state sync
    client.on('initialStateSync', (tables: Record<string, unknown[]>) => {
      if (tables.player) {
        setPlayers(tables.player as Player[])
      }
      if (tables.world_entity) {
        setEntities(tables.world_entity as WorldEntityRow[])
      }
      if (tables.chat_message) {
        setMessages((tables.chat_message as ChatMessage[]).sort((a, b) => Number(a.id) - Number(b.id)))
      }
      if (tables.world_info && tables.world_info.length > 0) {
        setWorldInfo(tables.world_info[0] as WorldInfo)
      }
    })

    // Handle row inserts
    client.on('rowInsert', (table: string, row: Record<string, unknown>) => {
      if (table === 'player') {
        setPlayers(prev => [...prev, row as Player])
      } else if (table === 'world_entity') {
        setEntities(prev => [...prev, row as WorldEntityRow])
      } else if (table === 'chat_message') {
        const msg = row as ChatMessage
        setMessages(prev => {
          if (prev.find(m => m.id === msg.id)) return prev
          return [...prev, msg].sort((a, b) => Number(a.id) - Number(b.id))
        })
      } else if (table === 'world_info') {
        setWorldInfo(row as WorldInfo)
      }
    })

    // Handle row updates
    client.on('rowUpdate', (table: string, row: Record<string, unknown>) => {
      if (table === 'player') {
        setPlayers(prev => {
          const idx = prev.findIndex(p => p.identity === (row as Player).identity)
          if (idx >= 0) {
            const updated = [...prev]
            updated[idx] = row as Player
            return updated
          }
          return prev
        })
      } else if (table === 'world_entity') {
        setEntities(prev => {
          const idx = prev.findIndex(e => e.id === (row as WorldEntityRow).id)
          if (idx >= 0) {
            const updated = [...prev]
            updated[idx] = row as WorldEntityRow
            return updated
          }
          return prev
        })
      } else if (table === 'world_info') {
        setWorldInfo(row as WorldInfo)
      }
    })

    // Handle row deletes
    client.on('rowDelete', (table: string, row: Record<string, unknown>) => {
      if (table === 'player') {
        setPlayers(prev => prev.filter(p => p.identity !== (row as Player).identity))
      } else if (table === 'world_entity') {
        setEntities(prev => prev.filter(e => e.id !== (row as WorldEntityRow).id))
      } else if (table === 'chat_message') {
        setMessages(prev => prev.filter(m => m.id !== (row as ChatMessage).id))
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

  const setDevice = useCallback((device: string) => {
    if (client) {
      client.callReducer('set_device', [device])
    }
  }, [client])

  const spawnEntity = useCallback((entity: Partial<WorldEntityRow>) => {
    if (client) {
      client.callReducer('spawn_entity', [
        entity.name || 'entity',
        entity.entityType || 'custom',
        entity.position?.x ?? 0,
        entity.position?.y ?? 0,
        entity.position?.z ?? 0,
        entity.rotation?.pitch ?? 0,
        entity.rotation?.yaw ?? 0,
        entity.rotation?.roll ?? 0,
        entity.scale ?? 1,
        entity.shapeJson || '{}',
        entity.materialJson || null,
        entity.lightJson || null,
        entity.behaviorsJson || '[]',
        entity.audioJson || null,
      ])
    }
  }, [client])

  const removeEntity = useCallback((id: number) => {
    if (client) {
      client.callReducer('remove_entity', [id])
    }
  }, [client])

  const subscribeChunk = useCallback((x: number, y: number) => {
    if (client) {
      client.callReducer('subscribe_chunk', [x, y])
    }
  }, [client])

  return {
    connected,
    identity,
    players,
    entities,
    messages,
    worldInfo,
    sendMessage,
    movePlayer,
    setName,
    setDevice,
    spawnEntity,
    removeEntity,
    subscribeChunk,
  }
}
