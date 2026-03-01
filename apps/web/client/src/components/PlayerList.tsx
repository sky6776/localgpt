import { useState } from 'react'
import './PlayerList.css'

interface Player {
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

interface PlayerListProps {
  players: Player[]
  identity: string | null
  setName: (name: string) => void
}

export function PlayerList({ players, identity, setName }: PlayerListProps) {
  const [isEditing, setIsEditing] = useState(false)
  const [editName, setEditName] = useState('')

  const currentPlayer = players.find(p => p.identity === identity)

  const handleStartEdit = () => {
    setEditName(currentPlayer?.name || '')
    setIsEditing(true)
  }

  const handleSave = () => {
    if (editName.trim()) {
      setName(editName.trim())
    }
    setIsEditing(false)
  }

  const getDeviceIcon = (device: string) => {
    switch (device) {
      case 'ios':
      case 'android':
      case 'mobile':
        return '📱'
      case 'web':
        return '🌐'
      default:
        return '👤'
    }
  }

  return (
    <div className="player-list">
      <h3>Players</h3>
      <ul>
        {players.filter(p => p.online).map(player => (
          <li key={player.identity} className={player.identity === identity ? 'self' : ''}>
            {player.identity === identity && isEditing ? (
              <div className="name-edit">
                <input
                  type="text"
                  value={editName}
                  onChange={e => setEditName(e.target.value)}
                  maxLength={32}
                  autoFocus
                  onKeyDown={e => {
                    if (e.key === 'Enter') handleSave()
                    if (e.key === 'Escape') setIsEditing(false)
                  }}
                />
                <button onClick={handleSave}>✓</button>
              </div>
            ) : (
              <>
                <span className="device-icon">{getDeviceIcon(player.device)}</span>
                <span className="status-dot" />
                <span className="name">{player.name}</span>
                {player.identity === identity && (
                  <button className="edit-btn" onClick={handleStartEdit}>✎</button>
                )}
              </>
            )}
          </li>
        ))}
      </ul>
    </div>
  )
}
